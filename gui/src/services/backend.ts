import { ConnectionStatus, Peer, AppSettings, LogEntry } from '@/types'
import { mapErrorToCode, CrypRQErrorCode } from '@/errors/catalog'
import { errorBus } from '@/hooks/useErrorBus'

declare global {
  interface Window {
    electronAPI: {
      sessionStart: (args: { binPath?: string; binArgs: string[]; listenMultiaddr?: string; peerMultiaddr?: string }) => Promise<{ ok: boolean; error?: string; message?: string; lastLogs?: string[] }>
      sessionStop: () => Promise<{ ok: boolean; error?: string; force?: boolean }>
      sessionGet: () => Promise<{ state: string; pid: number | null; lastLogs: string[] }>
      sessionRestart: (args: { binPath?: string; binArgs: string[]; listenMultiaddr?: string; peerMultiaddr?: string }) => Promise<any>
      onSessionEvent: (callback: (event: any) => void) => void
      onSessionLog: (callback: (log: { level: string; message: string }) => void) => void
      onSessionEnded: (callback: (data: { code: number | null; signal: string | null; state: string; lastLogs: string[] }) => void) => void
      onSessionError: (callback: (error: { error: string; code: string; lastLogs: string[] }) => void) => void
      metricsGet: () => Promise<{ bytesIn?: number; bytesOut?: number; rotationTimer?: number; latency?: number; peerId?: string }>
      metricsStart: (intervalMs?: number) => Promise<{ ok: boolean }>
      metricsStop: () => Promise<{ ok: boolean }>
      heartbeatPing: () => Promise<{ ok: boolean; timestamp: number }>
      onHeartbeatStalled: (callback: (data: { missedBeats: number }) => void) => void
      peerTestReachability: (multiaddr: string) => Promise<{ ok: boolean; latencyMs?: number; error?: string }>
      settingsLoad: () => Promise<any>
      settingsSave: (settings: any) => Promise<void>
      onTrayToggleConnect: (callback: () => void) => void
      onTraySwitchPeer: (callback: (peerId: string) => void) => void
      onSessionStateChanged: (callback: (data: { status: string; peerId?: string }) => void) => void
      logsTail: (options: { lines?: number }) => Promise<string[]>
      logsList: () => Promise<string[]>
      diagnosticsExport: () => Promise<{ ok: boolean; path?: string }>
      removeAllListeners: (channel: string) => void
    }
  }
}

/**
 * Backend service for communicating with CrypRQ CLI via Electron IPC
 */
class BackendService {
  private listeners: Map<string, Function[]> = new Map()
  private metricsPollInterval: NodeJS.Timeout | null = null
  private heartbeatInterval: NodeJS.Timeout | null = null
  private currentStatus: ConnectionStatus = { connected: false }

  constructor() {
    if (typeof window !== 'undefined' && window.electronAPI) {
      this.setupEventListeners()
      this.startHeartbeat()
      this.startMetricsPolling()
    }
  }

  private setupEventListeners() {
    if (!window.electronAPI) return

    // Session events
    window.electronAPI.onSessionEvent((event: any) => {
      this.handleSessionEvent(event)
      
      // Handle rotation events
      if (event.type === 'rotation') {
        this.emit('rotation', {
          type: 'rotation.started',
          nextInSeconds: event.nextInSeconds,
        })
        // Rotation completed will be handled after delay
        setTimeout(() => {
          this.emit('rotation', {
            type: 'rotation.completed',
            nextInSeconds: event.nextInSeconds,
          })
        }, 1000)
      }
    })
    
    // Listen to session state changes for rotation status
    window.electronAPI.onSessionStateChanged?.((data: { status: string; peerId?: string }) => {
      if (data.status === 'rotating') {
        this.emit('rotation', { type: 'rotation.started' })
      } else if (data.status === 'running' && this.currentStatus.connected) {
        // Check if we just completed a rotation
        this.emit('rotation', { type: 'rotation.completed' })
      }
    })

    window.electronAPI.onSessionLog((log: { level: string; message: string }) => {
      const logEntry = {
        timestamp: new Date(),
        level: log.level as any,
        message: log.message,
      }
      this.emit('log', logEntry)
      
      // Also emit structured log for LogsPanel
      this.emit('structuredLog', {
        ts: new Date().toISOString(),
        level: log.level as any,
        source: 'cli' as const,
        msg: log.message,
      })
    })

    window.electronAPI.onSessionEnded((data: { code: number | null; signal: string | null; state: string; lastLogs: string[] }) => {
      this.currentStatus = { connected: false }
      this.emit('status', this.currentStatus)
      
      if (data.state === 'errored') {
        const errorCode = mapErrorToCode(`Process exited with code ${data.code}`, data.lastLogs)
        errorBus.emit({
          code: errorCode,
          message: `Session ended with code ${data.code}`,
          lastLogs: data.lastLogs,
        })
      }
    })

    window.electronAPI.onSessionError((error: { error: string; code: string; lastLogs: string[] }) => {
      this.currentStatus = { connected: false }
      this.emit('status', this.currentStatus)
      const errorCode = mapErrorToCode(error.error, error.lastLogs)
      errorBus.emit({
        code: errorCode,
        message: error.error,
        lastLogs: error.lastLogs,
      })
    })

    // Heartbeat stalled
    window.electronAPI.onHeartbeatStalled((data: { missedBeats: number }) => {
      console.warn('Renderer heartbeat stalled, missed beats:', data.missedBeats)
    })

    // Session state changes (for tray updates)
    window.electronAPI.onSessionStateChanged((data: { status: string; peerId?: string }) => {
      if (data.status === 'running') {
        this.currentStatus = {
          ...this.currentStatus,
          connected: true,
          peerId: data.peerId,
        }
      } else {
        this.currentStatus = { connected: false }
      }
      this.emit('status', this.currentStatus)
    })

    // Tray actions
    window.electronAPI.onTrayToggleConnect(async () => {
      if (this.currentStatus.connected) {
        await this.disconnect()
      } else {
        await this.connect()
      }
    })

    window.electronAPI.onTraySwitchPeer(async (peerId: string) => {
      // Restart session with new peer
      // Would need to look up peer multiaddr from stored peers
      await this.restartSession(peerId)
    })
  }

  private handleSessionEvent(event: any) {
    // Parse structured events from CLI
    if (event.type === 'connected') {
      this.currentStatus = {
        connected: true,
        peerId: event.peerId,
        rotationTimer: event.rotationTimer,
      }
      this.emit('status', this.currentStatus)
    } else if (event.type === 'disconnected') {
      this.currentStatus = { connected: false }
      this.emit('status', this.currentStatus)
    } else if (event.type === 'rotation') {
      this.currentStatus.rotationTimer = event.timer
      this.emit('status', { ...this.currentStatus })
    }
  }

  private startMetricsPolling() {
    if (!window.electronAPI) return

    // Start metrics polling in main process
    window.electronAPI.metricsStart(2000).then(() => {
      // Also poll from renderer to update UI
      this.metricsPollInterval = setInterval(async () => {
        try {
          const metrics = await window.electronAPI.metricsGet()
          
          if (metrics.bytesIn !== undefined || metrics.bytesOut !== undefined) {
            this.currentStatus.throughput = {
              bytesIn: metrics.bytesIn || 0,
              bytesOut: metrics.bytesOut || 0,
            }
          }
          
          if (metrics.rotationTimer !== undefined) {
            this.currentStatus.rotationTimer = metrics.rotationTimer
          }
          
          if (metrics.peerId) {
            this.currentStatus.peerId = metrics.peerId
          }
          
          if (metrics.latency !== undefined) {
            this.currentStatus.latency = metrics.latency
          }
          
          this.emit('status', { ...this.currentStatus })
        } catch (error) {
          // Metrics endpoint not available, ignore
        }
      }, 2000)
    })
  }

  private startHeartbeat() {
    if (!window.electronAPI) return

    this.heartbeatInterval = setInterval(async () => {
      try {
        await window.electronAPI.heartbeatPing()
      } catch (error) {
        console.error('Heartbeat ping failed:', error)
      }
    }, 5000) // Every 5 seconds
  }

  async connect(peerMultiaddr?: string, listenMultiaddr?: string): Promise<void> {
    if (!window.electronAPI) {
      throw new Error('Electron API not available')
    }

    const args: any = {
      binArgs: [],
    }

    if (peerMultiaddr) {
      args.peerMultiaddr = peerMultiaddr
    } else if (listenMultiaddr) {
      args.listenMultiaddr = listenMultiaddr
    } else {
      args.listenMultiaddr = '/ip4/0.0.0.0/udp/9999/quic-v1'
    }

    const result = await window.electronAPI.sessionStart(args)
    
    if (!result.ok) {
      const errorCode = mapErrorToCode(result.error || result.message || 'Unknown error', result.lastLogs)
      const error = new Error(result.message || result.error || 'Failed to start session')
      
      // Emit error via error bus
      errorBus.emit({
        code: errorCode,
        message: error.message,
        lastLogs: result.lastLogs,
      })
      
      throw error
    }

    // Status will be updated via events
  }

  async disconnect(): Promise<void> {
    if (!window.electronAPI) {
      throw new Error('Electron API not available')
    }

    const result = await window.electronAPI.sessionStop()
    
    if (!result.ok) {
      throw new Error(result.error || 'Failed to stop session')
    }

    this.currentStatus = { connected: false }
    this.emit('status', this.currentStatus)
  }

  async getStatus(): Promise<ConnectionStatus> {
    if (!window.electronAPI) {
      return this.currentStatus
    }

    try {
      const session = await window.electronAPI.sessionGet()
      const metrics = await window.electronAPI.metricsGet()
      
      return {
        connected: session.state === 'running',
        peerId: metrics.peerId || this.currentStatus.peerId,
        rotationTimer: metrics.rotationTimer || this.currentStatus.rotationTimer,
        throughput: metrics.bytesIn !== undefined || metrics.bytesOut !== undefined ? {
          bytesIn: metrics.bytesIn || 0,
          bytesOut: metrics.bytesOut || 0,
        } : this.currentStatus.throughput,
        latency: metrics.latency,
      }
    } catch (error) {
      return this.currentStatus
    }
  }

  async getPeers(): Promise<Peer[]> {
    // TODO: Read from configuration file
    return []
  }

  async addPeer(peer: Peer): Promise<void> {
    // TODO: Update configuration file
    console.log('Adding peer:', peer)
  }

  async removePeer(peerId: string): Promise<void> {
    // TODO: Update configuration file
    console.log('Removing peer:', peerId)
  }

  async updateSettings(settings: Partial<AppSettings>): Promise<void> {
    // TODO: Write to configuration file
    console.log('Updating settings:', settings)
  }

  async restartSession(peerMultiaddr?: string, listenMultiaddr?: string, peerId?: string): Promise<void> {
    if (!window.electronAPI) {
      throw new Error('Electron API not available')
    }

    const args: any = {
      binArgs: [],
    }

    if (peerMultiaddr) {
      args.peerMultiaddr = peerMultiaddr
    } else if (listenMultiaddr) {
      args.listenMultiaddr = listenMultiaddr
    } else if (peerId) {
      // Look up peer by ID (would need peer storage)
      // For now, just use peerId as multiaddr placeholder
      args.peerMultiaddr = peerId
    }

    await window.electronAPI.sessionRestart(args)
  }

  on(event: string, callback: Function) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, [])
    }
    this.listeners.get(event)!.push(callback)
  }

  off(event: string, callback: Function) {
    const callbacks = this.listeners.get(event)
    if (callbacks) {
      const index = callbacks.indexOf(callback)
      if (index > -1) {
        callbacks.splice(index, 1)
      }
    }
  }

  private emit(event: string, data: any) {
    const callbacks = this.listeners.get(event) || []
    callbacks.forEach(cb => cb(data))
  }

  cleanup() {
    if (this.metricsPollInterval) {
      clearInterval(this.metricsPollInterval)
      this.metricsPollInterval = null
    }
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval)
      this.heartbeatInterval = null
    }
    if (window.electronAPI) {
      window.electronAPI.removeAllListeners('session:event')
      window.electronAPI.removeAllListeners('session:log')
      window.electronAPI.removeAllListeners('session:ended')
      window.electronAPI.removeAllListeners('session:error')
    }
  }
}

export const backend = new BackendService()

export async function peerTestReachability(multiaddr: string): Promise<{ ok: boolean; latencyMs?: number; error?: string }> {
  if (typeof window === 'undefined' || !window.electronAPI) {
    return { ok: false, error: 'NOT_AVAILABLE' }
  }
  
  try {
    return await window.electronAPI.peerTestReachability(multiaddr)
  } catch (error: any) {
    return { ok: false, error: error.message || 'NET_UNREACHABLE' }
  }
}
