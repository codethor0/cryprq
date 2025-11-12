import axios from 'axios';
import EventEmitter from 'eventemitter3';
import type {Metrics, ConnectionState, ProfileType} from '@/types';
import {useAppStore} from '@/store/appStore';
import {validateRemoteEndpoint, NETWORK_TIMEOUTS, redactSecrets} from './security';

const METRICS_POLL_INTERVAL = 2000; // 2 seconds

class BackendService extends EventEmitter {
  private pollInterval?: NodeJS.Timeout;
  private currentProfile: ProfileType = 'LOCAL';
  private currentEndpoint?: string;

  getMetricsEndpoint(): string {
    const {settings} = useAppStore.getState();
    const endpoint = settings.endpoint || 'http://127.0.0.1:9464';
    return `${endpoint}/metrics`;
  }

  async getMetrics(): Promise<Metrics | null> {
    const {settings} = useAppStore.getState();
    
    // Validate REMOTE endpoint
    if (settings.profile === 'REMOTE' && settings.endpoint) {
      const validation = validateRemoteEndpoint(settings.endpoint);
      if (!validation.valid) {
        this.emit('error', {type: 'INVALID_ENDPOINT', error: validation.error});
        return null;
      }
    }
    
    try {
      const response = await axios.get(this.getMetricsEndpoint(), {
        timeout: NETWORK_TIMEOUTS.read,
        headers: {
          'Accept': 'text/plain',
        },
      });

      return this.parsePrometheusMetrics(response.data);
    } catch (error: any) {
      const redactedError = redactSecrets(error.message);
      console.error('Failed to fetch metrics:', redactedError);
      this.emit('error', {type: 'METRICS_FETCH_FAILED', error: redactedError});
      return null;
    }
  }

  private parsePrometheusMetrics(text: string): Metrics {
    const lines = text.split('\n');
    const metrics: Partial<Metrics> = {};

    for (const line of lines) {
      if (line.startsWith('#') || !line.trim()) continue;

      // Parse Prometheus format: metric_name value or metric_name{label="value"} value
      const match = line.match(/^(\w+)(?:\{([^}]+)\})?\s+([\d.]+)$/);
      if (!match) continue;

      const [, name, labels, value] = match;
      const numValue = parseFloat(value);

      switch (name) {
        case 'cr_bytesIn':
          metrics.bytesIn = numValue;
          break;
        case 'cr_bytesOut':
          metrics.bytesOut = numValue;
          break;
        case 'cr_latency_ms':
          metrics.latency = numValue;
          break;
        case 'cr_rotation_timer_s':
          metrics.rotationTimer = numValue;
          break;
        case 'cr_peer_id':
          // Extract peer_id from labels: cr_peer_id{peer_id="Qm..."} 1
          if (labels) {
            const peerIdMatch = labels.match(/peer_id="([^"]+)"/);
            if (peerIdMatch) {
              metrics.peerId = peerIdMatch[1];
            }
          }
          break;
      }
    }

    return {
      bytesIn: metrics.bytesIn || 0,
      bytesOut: metrics.bytesOut || 0,
      latency: metrics.latency || 0,
      rotationTimer: metrics.rotationTimer || 0,
      peerId: metrics.peerId,
    };
  }

  startPolling() {
    if (this.pollInterval) return;

    this.pollInterval = setInterval(async () => {
      const metrics = await this.getMetrics();
      if (metrics) {
        const {updateMetrics, addThroughputPoint, connectionStatus} = useAppStore.getState();
        
        // Update connection state
        const updates: Partial<ConnectionState> = {
          latency: metrics.latency,
          rotationTimer: metrics.rotationTimer,
          bytesIn: metrics.bytesIn,
          bytesOut: metrics.bytesOut,
        };
        
        if (metrics.peerId) {
          updates.peerId = metrics.peerId;
          if (connectionStatus.status === 'disconnected' || connectionStatus.status === 'connecting') {
            updates.status = 'connected';
          }
        }

        updateMetrics(updates);

        // Add throughput point
        addThroughputPoint({
          timestamp: Date.now(),
          bytesIn: metrics.bytesIn,
          bytesOut: metrics.bytesOut,
        });

        // Emit metrics event
        this.emit('metrics', metrics);

        // Check for rotation
        if (metrics.rotationTimer === 0 && connectionStatus.status === 'connected') {
          this.emit('rotation', {type: 'rotation.started'});
          updateMetrics({status: 'rotating'});
          setTimeout(() => {
            updateMetrics({status: 'connected'});
            this.emit('rotation', {type: 'rotation.completed', nextInSeconds: 300});
          }, 1000);
        }
        
        // Emit rotation.scheduled when timer updates
        if (metrics.rotationTimer > 0 && connectionStatus.status === 'connected') {
          this.emit('rotation', {type: 'rotation.scheduled', nextInSeconds: metrics.rotationTimer});
        }
      }
    }, METRICS_POLL_INTERVAL);
  }

  stopPolling() {
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = undefined;
    }
  }

  async testReachability(endpoint: string): Promise<{ok: boolean; latency?: number; error?: string}> {
    try {
      const start = Date.now();
      await axios.get(`${endpoint}/metrics`, {
        timeout: NETWORK_TIMEOUTS.connect,
        validateStatus: () => true, // Accept any status for reachability test
      });
      const latency = Date.now() - start;
      return {ok: true, latency};
    } catch (error: any) {
      const redactedError = redactSecrets(error.message);
      return {ok: false, error: redactedError};
    }
  }

  setProfile(profile: ProfileType, endpoint?: string) {
    this.currentProfile = profile;
    this.currentEndpoint = endpoint;
    // Restart polling with new endpoint
    this.stopPolling();
    this.startPolling();
  }
}

export const backendService = new BackendService();

