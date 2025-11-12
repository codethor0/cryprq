import { contextBridge, ipcRenderer } from 'electron'

export interface ElectronAPI {
  // Session management
  sessionStart: (args: { binPath?: string; binArgs: string[]; listenMultiaddr?: string; peerMultiaddr?: string }) => Promise<{ ok: boolean; error?: string; message?: string; lastLogs?: string[] }>
  sessionStop: () => Promise<{ ok: boolean; error?: string; force?: boolean }>
  sessionGet: () => Promise<{ state: string; pid: number | null; lastLogs: string[] }>
  sessionRestart: (args: { binPath?: string; binArgs: string[]; listenMultiaddr?: string; peerMultiaddr?: string }) => Promise<any>
  
  // Events
  onSessionEvent: (callback: (event: any) => void) => void
  onSessionLog: (callback: (log: { level: string; message: string }) => void) => void
  onSessionEnded: (callback: (data: { code: number | null; signal: string | null; state: string; lastLogs: string[] }) => void) => void
  onSessionError: (callback: (error: { error: string; code: string; lastLogs: string[] }) => void) => void
  
  // Metrics
  metricsGet: () => Promise<{ bytesIn?: number; bytesOut?: number; rotationTimer?: number; latency?: number; peerId?: string }>
  metricsStart: (intervalMs?: number) => Promise<{ ok: boolean }>
  metricsStop: () => Promise<{ ok: boolean }>
  
  // Heartbeat
  heartbeatPing: () => Promise<{ ok: boolean; timestamp: number }>
  onHeartbeatStalled: (callback: (data: { missedBeats: number }) => void) => void
  
  // Peer operations
  peerTestReachability: (multiaddr: string) => Promise<{ ok: boolean; latencyMs?: number; error?: string }>
  
  // Settings
  settingsLoad: () => Promise<any>
  settingsSave: (settings: any) => Promise<void>
  
  // Telemetry
  telemetrySet: (enabled: boolean) => Promise<{ ok: boolean }>
  
  // Tray actions (from renderer to main)
  onTrayToggleConnect: (callback: () => void) => void
  onTraySwitchPeer: (callback: (peerId: string) => void) => void
  
  // Remove listeners
  removeAllListeners: (channel: string) => void
  
  // Menu events
  onMenuReportIssue: (callback: () => void) => void
  
  // Shell operations
  shellShowItemInFolder: (path: string) => Promise<void>
  
  // Dev hooks
  devTraySnapshot: () => Promise<{ status: string; currentPeer: { peerId: string } | null; recentLabels: string[]; items: string[] }>
  devSessionSimulateExit: (args: { code?: number; signal?: string }) => Promise<{ ok: boolean; error?: string }>
}

contextBridge.exposeInMainWorld('electronAPI', {
  // Session management
  sessionStart: (args: any) => ipcRenderer.invoke('session:start', args),
  sessionStop: () => ipcRenderer.invoke('session:stop'),
  sessionGet: () => ipcRenderer.invoke('session:get'),
  sessionRestart: (args: any) => ipcRenderer.invoke('session:restart', args),
  
  // Events
  onSessionEvent: (callback: (event: any) => void) => {
    ipcRenderer.on('session:event', (_event, data) => callback(data))
  },
  onSessionLog: (callback: (log: { level: string; message: string }) => void) => {
    ipcRenderer.on('session:log', (_event, data) => callback(data))
  },
  onSessionEnded: (callback: (data: any) => void) => {
    ipcRenderer.on('session:ended', (_event, data) => callback(data))
  },
  onSessionError: (callback: (error: any) => void) => {
    ipcRenderer.on('session:error', (_event, data) => callback(data))
  },
  
  // Metrics
  metricsGet: () => ipcRenderer.invoke('metrics:get'),
  metricsStart: (intervalMs?: number) => ipcRenderer.invoke('metrics:start', intervalMs),
  metricsStop: () => ipcRenderer.invoke('metrics:stop'),
  
  // Heartbeat
  heartbeatPing: () => ipcRenderer.invoke('heartbeat:ping'),
  onHeartbeatStalled: (callback: (data: any) => void) => {
    ipcRenderer.on('heartbeat:stalled', (_event, data) => callback(data))
  },
  
  // Peer operations
  peerTestReachability: (multiaddr: string) => ipcRenderer.invoke('peer:testReachability', { multiaddr }),
  
  // Settings
  settingsLoad: () => ipcRenderer.invoke('settings:load'),
  settingsSave: (settings: any) => ipcRenderer.invoke('settings:save', settings),
  
  // Telemetry
  telemetrySet: (enabled: boolean) => ipcRenderer.invoke('telemetry:set', enabled),
  
  // Tray actions
  onTrayToggleConnect: (callback: () => void) => {
    ipcRenderer.on('tray:toggleConnect', () => callback())
  },
  onTraySwitchPeer: (callback: (peerId: string) => void) => {
    ipcRenderer.on('tray:switchPeer', (_event, peerId) => callback(peerId))
  },
  
  // Session state changes
  onSessionStateChanged: (callback: (data: { status: string; peerId?: string }) => void) => {
    ipcRenderer.on('session:state-changed', (_event, data) => callback(data))
  },
  
  // Tray actions
  onTrayOpenDashboard: (callback: () => void) => {
    ipcRenderer.on('tray:openDashboard', () => callback())
  },
  
  // Logs
  logsTail: (options: { lines?: number }) => ipcRenderer.invoke('logs:tail', options),
  logsList: () => ipcRenderer.invoke('logs:list'),
  
  // Diagnostics
  diagnosticsExport: () => ipcRenderer.invoke('diagnostics:export'),
  
  // Dev hooks
  devTraySnapshot: () => ipcRenderer.invoke('dev:tray:snapshot'),
  devSessionSimulateExit: (args: { code?: number; signal?: string }) => ipcRenderer.invoke('dev:session:simulateExit', args),
  
  // Remove listeners
  removeAllListeners: (channel: string) => ipcRenderer.removeAllListeners(channel),
  
  // Menu events
  onMenuReportIssue: (callback: () => void) => {
    ipcRenderer.on('menu:report-issue', () => callback())
  },
  
  // Shell operations
  shellShowItemInFolder: (path: string) => ipcRenderer.invoke('shell:showItemInFolder', path),
} as ElectronAPI)

