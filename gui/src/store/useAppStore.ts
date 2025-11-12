import { create } from 'zustand'
import { AppState, ConnectionStatus, Peer, AppSettings, LogEntry, MetricsPoint } from '@/types'
import { backend } from '@/services/backend'

interface AppStore extends AppState {
  // Actions
  connect: (peerMultiaddr?: string) => Promise<void>
  disconnect: () => Promise<void>
  addPeer: (peer: Peer) => Promise<void>
  removePeer: (peerId: string) => Promise<void>
  updateSettings: (settings: Partial<AppSettings>) => Promise<void>
  addLog: (entry: LogEntry) => void
  clearLogs: () => void
  openLogsPanel?: () => void
  addMetricsPoint: (point: MetricsPoint) => void
}

const defaultSettings: AppSettings = {
  rotationInterval: 300, // 5 minutes
  logLevel: 'info',
  transport: {
    multiaddr: '/ip4/0.0.0.0/udp/9999/quic-v1',
    udpPort: 9999,
  },
  theme: 'system',
  disconnectOnQuit: true, // Kill-switch: default ON
  remoteEndpointAllowlist: [], // Empty = no restrictions
  chartSmoothing: 0.2, // EMA alpha for charts (default: 0.2)
  telemetryEnabled: false, // Opt-in telemetry v0 (default: OFF)
}

export const useAppStore = create<AppStore>((set, get) => ({
  connectionStatus: {
    connected: false,
  },
  peers: [],
  settings: defaultSettings,
  logs: [],
  structuredLogs: [],
  metricsSeries60s: [],

  connect: async (peerMultiaddr?: string, listenMultiaddr?: string) => {
    try {
      await backend.connect(peerMultiaddr, listenMultiaddr)
      // Status updates will come via backend events
    } catch (error: any) {
      get().addLog({
        timestamp: new Date(),
        level: 'error',
        message: `Failed to connect: ${error?.message || error}`,
      })
      throw error
    }
  },

  disconnect: async () => {
    try {
      await backend.disconnect()
    } catch (error: any) {
      get().addLog({
        timestamp: new Date(),
        level: 'error',
        message: `Failed to disconnect: ${error?.message || error}`,
      })
      throw error
    }
  },

  addPeer: async (peer: Peer) => {
    try {
      await backend.addPeer(peer)
      set(state => ({
        peers: [...state.peers, peer],
      }))
      get().addLog({
        timestamp: new Date(),
        level: 'info',
        message: `Added peer: ${peer.id}`,
      })
    } catch (error) {
      get().addLog({
        timestamp: new Date(),
        level: 'error',
        message: `Failed to add peer: ${error}`,
      })
    }
  },

  removePeer: async (peerId: string) => {
    try {
      await backend.removePeer(peerId)
      set(state => ({
        peers: state.peers.filter(p => p.id !== peerId),
      }))
      get().addLog({
        timestamp: new Date(),
        level: 'info',
        message: `Removed peer: ${peerId}`,
      })
    } catch (error) {
      get().addLog({
        timestamp: new Date(),
        level: 'error',
        message: `Failed to remove peer: ${error}`,
      })
    }
  },

  updateSettings: async (settings: Partial<AppSettings>) => {
    try {
      await backend.updateSettings(settings)
      set(state => ({
        settings: { ...state.settings, ...settings },
      }))
      get().addLog({
        timestamp: new Date(),
        level: 'info',
        message: 'Settings updated',
      })
    } catch (error) {
      get().addLog({
        timestamp: new Date(),
        level: 'error',
        message: `Failed to update settings: ${error}`,
      })
    }
  },

  addLog: (entry: LogEntry) => {
    set(state => {
      const structured: LogLine = {
        ts: entry.timestamp.toISOString(),
        level: entry.level,
        source: 'app',
        msg: entry.message,
      }
      return {
        logs: [...state.logs.slice(-99), entry], // Keep last 100 logs
        structuredLogs: [...state.structuredLogs.slice(-4999), structured], // Keep last 5000 structured logs
      }
    })
  },

  addStructuredLog: (log: LogLine) => {
    set(state => ({
      structuredLogs: [...state.structuredLogs.slice(-4999), log], // Ring buffer of 5000
    }))
  },

  clearLogs: () => {
    set({ logs: [] })
  },

  restartSession: async (peerMultiaddr?: string, listenMultiaddr?: string, peerId?: string) => {
    try {
      await backend.restartSession(peerMultiaddr, listenMultiaddr, peerId)
    } catch (error: any) {
      get().addLog({
        timestamp: new Date(),
        level: 'error',
        message: `Failed to restart session: ${error?.message || error}`,
      })
      throw error
    }
  },

  openLogsPanel: (search?: string, timeRange?: { start: Date; end: Date }) => {
    // This will be handled by the LogsPanel component via props
    // Navigation is handled in App.tsx
  },

  addMetricsPoint: (point: MetricsPoint) => {
    const now = Date.now()
    const cutoff = now - 60000 // 60 seconds ago
    
    set(state => {
      // Keep only points within 60s window
      const filtered = state.metricsSeries60s.filter(p => p.ts > cutoff)
      return {
        metricsSeries60s: [...filtered, point],
      }
    })
  },
}))

// Throttle metrics ingestion to 1 Hz max
let lastMetricsUpdate = 0
const METRICS_THROTTLE_MS = 1000 // 1 second

// Subscribe to backend events
backend.on('status', (status: ConnectionStatus) => {
  const state = useAppStore.getState()
  useAppStore.setState({ connectionStatus: status })
  
  // Update metrics series when throughput/latency changes (throttled to 1 Hz)
  const now = Date.now()
  if ((status.throughput || status.latency !== undefined) && (now - lastMetricsUpdate >= METRICS_THROTTLE_MS)) {
    lastMetricsUpdate = now
    
    // Use requestIdleCallback for non-blocking updates if available
    const updateMetrics = () => {
      state.addMetricsPoint({
        ts: now,
        bytesIn: status.throughput?.bytesIn || 0,
        bytesOut: status.throughput?.bytesOut || 0,
        latencyMs: status.latency,
      })
    }
    
    if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
      requestIdleCallback(updateMetrics, { timeout: 100 })
    } else {
      updateMetrics()
    }
  }
})

backend.on('log', (entry: LogEntry) => {
  useAppStore.getState().addLog(entry)
})

backend.on('structuredLog', (log: LogLine) => {
  useAppStore.getState().addStructuredLog(log)
})

