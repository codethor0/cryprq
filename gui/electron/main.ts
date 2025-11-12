import { app, BrowserWindow, Tray, Menu, nativeImage, ipcMain, shell } from 'electron'
import * as path from 'path'
import './main/session'
import './main/metrics'
import './main/peers'
import './main/settings-ipc'
import './main/logging-ipc'
import './main/diagnostics'
import { startHeartbeatMonitor, stopHeartbeatMonitor } from './main/heartbeat'
import { initTray, updateTray } from './main/tray'
import {
  loadSettings,
  saveSettings,
  addRecentPeer,
  setLastPeerId,
  getLastPeerId,
  getRecentPeers,
} from './main/settings'
import { stopSession } from './main/session'
import { emitTelemetry, setTelemetryEnabled } from './main/telemetry'

// IPC handlers
ipcMain.handle('shell:showItemInFolder', async (_event, path: string) => {
  shell.showItemInFolder(path)
})

// Telemetry toggle IPC handler
ipcMain.handle('telemetry:set', async (_event, enabled: boolean) => {
  setTelemetryEnabled(enabled)
  return { ok: true }
})

let mainWindow: BrowserWindow | null = null
let tray: Tray | null = null
let isQuitting = false

// Single instance lock
const gotTheLock = app.requestSingleInstanceLock()

if (!gotTheLock) {
  app.quit()
} else {
  app.on('second-instance', () => {
    // Focus existing window
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore()
      mainWindow.focus()
    }
  })
}

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 800,
    minHeight: 600,
    backgroundColor: '#121212',
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, 'preload.js'),
    },
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'default',
  })

  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadURL('http://localhost:5173')
    mainWindow.webContents.openDevTools()
  } else {
    mainWindow.loadFile(path.join(__dirname, '../dist/index.html'))
  }

  mainWindow.on('closed', () => {
    mainWindow = null
  })

  // Setup close handler (must be after mainWindow is assigned)
  setupWindowCloseHandler()
}

function setupWindowCloseHandler() {
  if (mainWindow) {
    mainWindow.on('close', (event) => {
      const settings = loadSettings()
      const keepRunning = settings.keepRunningInBackground !== false // Default true

      if (!isQuitting && keepRunning) {
        event.preventDefault()
        mainWindow?.hide()
      }
    })
  }
}

// Tray is now handled by tray.ts module

// Setup application menu
function setupApplicationMenu() {
  if (process.platform === 'darwin') {
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'CrypRQ',
        submenu: [
          { role: 'about' },
          { type: 'separator' },
          { role: 'services' },
          { type: 'separator' },
          { role: 'hide' },
          { role: 'hideOthers' },
          { role: 'unhide' },
          { type: 'separator' },
          { role: 'quit' },
        ],
      },
      {
        label: 'Edit',
        submenu: [
          { role: 'undo' },
          { role: 'redo' },
          { type: 'separator' },
          { role: 'cut' },
          { role: 'copy' },
          { role: 'paste' },
          { role: 'selectAll' },
        ],
      },
      {
        label: 'View',
        submenu: [
          { role: 'reload' },
          { role: 'forceReload' },
          { role: 'toggleDevTools' },
          { type: 'separator' },
          { role: 'resetZoom' },
          { role: 'zoomIn' },
          { role: 'zoomOut' },
          { type: 'separator' },
          { role: 'togglefullscreen' },
        ],
      },
      {
        label: 'Help',
        submenu: [
          {
            label: 'Report Issue…',
            click: () => {
              const window = getMainWindow()
              if (window) {
                window.webContents.send('menu:report-issue')
              }
            },
          },
          { type: 'separator' },
          {
            label: 'Export Diagnostics…',
            click: async () => {
              const result = await ipcMain.invoke('diagnostics:export')
              if (result.ok) {
                dialog.showMessageBox(getMainWindow()!, {
                  type: 'info',
                  title: 'Diagnostics Exported',
                  message: `Diagnostics exported to:\n${result.path}`,
                  buttons: ['Open Folder', 'OK'],
                }).then((response) => {
                  if (response.response === 0) {
                    shell.showItemInFolder(result.path)
                  }
                })
              }
            },
          },
        ],
      },
    ]
    Menu.setApplicationMenu(Menu.buildFromTemplate(template))
  } else {
    // Windows/Linux menu
    const template: Electron.MenuItemConstructorOptions[] = [
      {
        label: 'File',
        submenu: [{ role: 'quit' }],
      },
      {
        label: 'Edit',
        submenu: [
          { role: 'undo' },
          { role: 'redo' },
          { type: 'separator' },
          { role: 'cut' },
          { role: 'copy' },
          { role: 'paste' },
        ],
      },
      {
        label: 'View',
        submenu: [
          { role: 'reload' },
          { role: 'forceReload' },
          { role: 'toggleDevTools' },
          { type: 'separator' },
          { role: 'resetZoom' },
          { role: 'zoomIn' },
          { role: 'zoomOut' },
          { type: 'separator' },
          { role: 'togglefullscreen' },
        ],
      },
      {
        label: 'Help',
        submenu: [
          {
            label: 'Report Issue…',
            click: () => {
              const window = getMainWindow()
              if (window) {
                window.webContents.send('menu:report-issue')
              }
            },
          },
          {
            label: 'Export Diagnostics…',
            click: async () => {
              const result = await ipcMain.invoke('diagnostics:export')
              if (result.ok) {
                dialog.showMessageBox(getMainWindow()!, {
                  type: 'info',
                  title: 'Diagnostics Exported',
                  message: `Diagnostics exported to:\n${result.path}`,
                  buttons: ['Open Folder', 'OK'],
                }).then((response) => {
                  if (response.response === 0) {
                    shell.showItemInFolder(result.path)
                  }
                })
              }
            },
          },
        ],
      },
    ]
    Menu.setApplicationMenu(Menu.buildFromTemplate(template))
  }
}

function getMainWindow(): BrowserWindow | undefined {
  return BrowserWindow.getAllWindows()[0]
}

app.whenReady().then(() => {
  // Load settings
  const settings = loadSettings()

  // Initialize telemetry state from settings
  setTelemetryEnabled(settings.telemetryEnabled === true)

  // Emit app.open telemetry event
  emitTelemetry('app.open')

  // Setup application menu
  setupApplicationMenu()

  // Initialize tray
  initTray()
  
  // Setup tray updater to listen for session state changes
  setupTrayUpdater()

  // Create window (or start minimized if configured)
  if (settings.startMinimized) {
    createWindow()
    if (mainWindow) {
      mainWindow.hide()
    }
  } else {
    createWindow()
  }

  startHeartbeatMonitor()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    } else if (mainWindow) {
      mainWindow.focus()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    isQuitting = true
    app.quit()
  }
})

app.on('before-quit', async (event) => {
  isQuitting = true
  stopHeartbeatMonitor()
  
  // Emit app.quit telemetry event
  emitTelemetry('app.quit')
  
  // Kill-switch: disconnect on app quit if enabled
  const settings = loadSettings()
  if (settings.disconnectOnQuit !== false) { // Default: true
    // Check if session is running and stop it
    const result = await stopSession()
    
    // If session was running, wait a moment for cleanup
    if (result.ok) {
      event.preventDefault() // Prevent quit until disconnect completes
      await new Promise(resolve => setTimeout(resolve, 200))
      app.quit()
    }
  }
})

