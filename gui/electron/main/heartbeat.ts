import { BrowserWindow, ipcMain } from 'electron'

let heartbeatInterval: NodeJS.Timeout | null = null
let lastHeartbeat: number = Date.now()
let missedBeats: number = 0
const HEARTBEAT_TIMEOUT_MS = 15000 // 3 beats * 5s
const MAX_MISSED_BEATS = 3

function startHeartbeatMonitor() {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval)
  }

  heartbeatInterval = setInterval(() => {
    const timeSinceLastBeat = Date.now() - lastHeartbeat
    
    if (timeSinceLastBeat > HEARTBEAT_TIMEOUT_MS) {
      missedBeats++
      
      if (missedBeats >= MAX_MISSED_BEATS) {
        console.warn('Renderer appears stalled, missed', missedBeats, 'heartbeats')
        
        // Notify all windows
        BrowserWindow.getAllWindows().forEach(w => {
          w.webContents.send('heartbeat:stalled', { missedBeats })
        })
        
        // Optionally attempt soft reload (commented out for now)
        // const mainWindow = BrowserWindow.getAllWindows()[0]
        // if (mainWindow) {
        //   mainWindow.reload()
        // }
      }
    } else {
      missedBeats = 0
    }
  }, 5000) // Check every 5 seconds
}

function stopHeartbeatMonitor() {
  if (heartbeatInterval) {
    clearInterval(heartbeatInterval)
    heartbeatInterval = null
  }
  missedBeats = 0
}

ipcMain.handle('heartbeat:ping', async () => {
  lastHeartbeat = Date.now()
  missedBeats = 0
  return { ok: true, timestamp: lastHeartbeat }
})

ipcMain.on('heartbeat:ping', () => {
  lastHeartbeat = Date.now()
  missedBeats = 0
})

export { startHeartbeatMonitor, stopHeartbeatMonitor }

