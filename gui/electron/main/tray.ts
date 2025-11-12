import { app, BrowserWindow, Menu, Tray, nativeImage, ipcMain } from 'electron'
import * as path from 'path'

let tray: Tray | null = null
let currentMenu: Electron.Menu | null = null
let currentTrayState: {
  status: 'connected' | 'rotating' | 'disconnected'
  currentPeer?: { alias?: string; peerId: string }
  recent: { alias?: string; peerId: string }[]
} = {
  status: 'disconnected',
  recent: [],
}

function getMainWindow(): BrowserWindow | undefined {
  return BrowserWindow.getAllWindows()[0]
}

function getTrayIcon(status: 'connected' | 'rotating' | 'disconnected'): Electron.NativeImage {
  // Try to load platform-specific icons
  const iconPath = (name: string) => {
    const basePath = path.join(__dirname, '../../assets/tray', name)
    // Fallback to empty if not found
    try {
      return nativeImage.createFromPath(basePath)
    } catch {
      return nativeImage.createEmpty()
    }
  }

  switch (status) {
    case 'connected':
      return iconPath('tray-connected.png')
    case 'rotating':
      return iconPath('tray-rotating.png')
    case 'disconnected':
    default:
      return iconPath('tray-disconnected.png')
  }
}

export function initTray() {
  if (tray) return tray

  // Create initial empty icon (will be updated by updateTray)
  const img = nativeImage.createEmpty()
  tray = new Tray(img)

  tray.setToolTip('CrypRQ')

  // Left-click: restore and focus window (Dashboard)
  tray.on('click', () => {
    const win = getMainWindow()
    if (win) {
      if (win.isMinimized()) win.restore()
      win.show()
      win.focus()
      // Navigate to dashboard if needed
      win.webContents.send('tray:openDashboard')
    }
  })

  updateTray({ status: 'disconnected', recent: [] })
  return tray
}

export function updateTray({
  status,
  currentPeer,
  recent,
}: {
  status: 'connected' | 'rotating' | 'disconnected'
  currentPeer?: { alias?: string; peerId: string }
  recent: { alias?: string; peerId: string }[]
}) {
  if (!tray) return

  // Update state for dev hooks
  currentTrayState = { status, currentPeer, recent }

  const statusDot = status === 'connected' ? '🟢' : status === 'rotating' ? '🟡' : '🔴'
  const statusLabel = status === 'connected' ? 'CONNECTED' : status === 'rotating' ? 'ROTATING' : 'DISCONNECTED'
  const peerLabel = currentPeer
    ? currentPeer.alias || currentPeer.peerId.slice(0, 8) + '…'
    : 'No peer'

  const win = getMainWindow()

  const menuItems: Electron.MenuItemConstructorOptions[] = [
    {
      label: `● ${statusLabel} — ${peerLabel}`,
      enabled: false,
    },
    { type: 'separator' },
    {
      label: status === 'connected' ? 'Disconnect' : 'Connect',
      click: () => {
        if (win) {
          win.webContents.send('tray:toggleConnect')
        }
      },
    },
  ]

  // Recent peers submenu
  if (recent.length > 0) {
    menuItems.push({
      label: 'Recent peers',
      submenu: recent.slice(0, 5).map((r) => ({
        label: r.alias ? `${r.alias} (${r.peerId.slice(0, 8)}…)` : r.peerId.slice(0, 16) + '…',
        click: () => {
          if (win) {
            win.webContents.send('tray:switchPeer', r.peerId)
          }
        },
      })),
    })
  }

  menuItems.push(
    { type: 'separator' },
    {
      label: 'Open Dashboard',
      click: () => {
        const w = getMainWindow()
        if (w) {
          if (w.isMinimized()) w.restore()
          w.show()
          w.focus()
        }
      },
    },
    {
      label: 'Quit',
      click: () => {
        app.quit()
      },
    }
  )

  const menu = Menu.buildFromTemplate(menuItems)
  tray.setContextMenu(menu)
  currentMenu = menu // Store menu for dev hooks
  tray.setToolTip(`CrypRQ — ${status}`)

  // Update icon
  const icon = getTrayIcon(status)
  if (!icon.isEmpty()) {
    tray.setImage(icon)
    if (process.platform === 'darwin') {
      // macOS: use template image for monochrome
      tray.setImage(icon)
    }
  }
}

export function getTray(): Tray | null {
  return tray
}

function getCurrentMenuLabels(): string[] {
  const menu = currentMenu
  const items: string[] = []
  
  if (menu) {
    menu.items.forEach((item: any) => {
      if (item.label && !item.enabled) {
        // Skip disabled items (status header)
        return
      }
      if (item.label) {
        items.push(item.label)
      }
      if (item.submenu) {
        item.submenu?.items.forEach((subItem: any) => {
          if (subItem.label) {
            items.push(subItem.label)
          }
        })
      }
    })
  }
  
  return items
}

// Dev hook for CI/testing
ipcMain.handle('dev:tray:snapshot', async () => {
  return {
    status: currentTrayState.status,
    currentPeer: currentTrayState.currentPeer || null,
    recentLabels: currentTrayState.recent.slice(0, 5).map(r => r.alias || r.peerId.slice(0, 8) + '…'),
    items: getCurrentMenuLabels(),
  }
})

