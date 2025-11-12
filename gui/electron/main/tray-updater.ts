import { BrowserWindow, ipcMain } from 'electron'
import { updateTray } from './tray'
import { getRecentPeers, addRecentPeer, setLastPeerId } from './settings'

// Unified function to update tray from session state
export function updateTrayFromSession(
  state: 'idle' | 'starting' | 'running' | 'stopping' | 'errored' | 'rotating',
  peerId?: string
) {
  const status = state === 'running' ? 'connected' : state === 'rotating' ? 'rotating' : 'disconnected'
  const currentPeer = peerId
    ? {
        peerId,
      }
    : undefined

  if (peerId) {
    setLastPeerId(peerId)
    addRecentPeer({ peerId })
  }

  updateTray({
    status,
    currentPeer,
    recent: getRecentPeers(),
  })
}

// Subscribe to session state changes and update tray
export function setupTrayUpdater() {
  // Listen for session state changes via IPC
  ipcMain.on('session:state-changed', (_event, data: { status: string; peerId?: string }) => {
    const state = data.status as 'idle' | 'starting' | 'running' | 'stopping' | 'errored' | 'rotating'
    updateTrayFromSession(state, data.peerId)
  })
}

