export interface Peer {
  id: string
  multiaddr: string
  status: 'connected' | 'disconnected' | 'connecting'
  lastSeen?: Date
  latency?: number
}

export interface ConnectionStatus {
  connected: boolean
  peerId?: string
  currentPeer?: Peer
  rotationTimer?: number // seconds until next rotation
  latency?: number // milliseconds
  throughput?: {
    bytesIn: number
    bytesOut: number
  }
}

export interface AppSettings {
  rotationInterval: number // seconds
  logLevel: 'error' | 'warn' | 'info' | 'debug'
  transport: {
    multiaddr: string
    udpPort: number
  }
  theme: 'light' | 'dark' | 'system'
  minimizeToTray?: boolean
  startMinimized?: boolean
  keepRunningInBackground?: boolean
  disconnectOnQuit?: boolean // Kill-switch: disconnect on app quit (default: true)
  remoteEndpointAllowlist?: string[] // Allowed domains for REMOTE profile
  chartSmoothing?: number // EMA alpha for charts (0-0.4, default: 0.2)
  telemetryEnabled?: boolean // Opt-in telemetry v0 (default: false)
  postQuantumEnabled?: boolean // Post-quantum encryption (ML-KEM + X25519 hybrid, default: true)
}

export interface MetricsPoint {
  ts: number // timestamp in ms
  bytesIn: number
  bytesOut: number
  latencyMs?: number
}

export interface AppState {
  connectionStatus: ConnectionStatus
  peers: Peer[]
  settings: AppSettings
  logs: LogEntry[]
  metricsSeries60s: MetricsPoint[] // 60s rolling window
}

export interface LogEntry {
  timestamp: Date
  level: 'error' | 'warn' | 'info' | 'debug'
  message: string
}

export interface LogLine {
  ts: string
  level: 'error' | 'warn' | 'info' | 'debug'
  source: 'cli' | 'ipc' | 'metrics' | 'app'
  msg: string
  meta?: Record<string, unknown>
}

