import { app } from 'electron'
import * as fs from 'fs'
import * as path from 'path'
import * as os from 'os'

const SETTINGS_DIR = path.join(os.homedir(), '.cryprq')
const SETTINGS_FILE = path.join(SETTINGS_DIR, 'settings.json')

interface AppSettings {
  minimizeToTray?: boolean
  startMinimized?: boolean
  keepRunningInBackground?: boolean
  disconnectOnQuit?: boolean // Kill-switch: disconnect on app quit
  remoteEndpointAllowlist?: string[] // Allowed domains for REMOTE profile
  telemetryEnabled?: boolean // Opt-in telemetry v0 (default: false)
  lastPeerId?: string
  recentPeers?: Array<{ alias?: string; peerId: string; lastUsed: number }>
}

const defaultSettings: AppSettings = {
  minimizeToTray: true,
  startMinimized: false,
  keepRunningInBackground: true,
  disconnectOnQuit: true, // Default: disconnect on quit
  remoteEndpointAllowlist: [], // Empty = no restrictions (for now)
  recentPeers: [],
}

function ensureDir(p: string) {
  fs.mkdirSync(p, { recursive: true })
}

export function loadSettings(): AppSettings {
  ensureDir(SETTINGS_DIR)

  if (!fs.existsSync(SETTINGS_FILE)) {
    return defaultSettings
  }

  try {
    const data = fs.readFileSync(SETTINGS_FILE, 'utf-8')
    const settings = JSON.parse(data)
    return { ...defaultSettings, ...settings }
  } catch (error) {
    console.error('Failed to load settings:', error)
    return defaultSettings
  }
}

export function saveSettings(settings: Partial<AppSettings>): void {
  ensureDir(SETTINGS_DIR)

  const current = loadSettings()
  const updated = { ...current, ...settings }

  // Cap recent peers at 5
  if (updated.recentPeers && updated.recentPeers.length > 5) {
    updated.recentPeers = updated.recentPeers
      .sort((a, b) => b.lastUsed - a.lastUsed)
      .slice(0, 5)
  }

  try {
    fs.writeFileSync(SETTINGS_FILE, JSON.stringify(updated, null, 2), 'utf-8')
  } catch (error) {
    console.error('Failed to save settings:', error)
  }
}

export function addRecentPeer(peer: { alias?: string; peerId: string }): void {
  const settings = loadSettings()
  const recent = settings.recentPeers || []

  // Remove if already exists
  const filtered = recent.filter((p) => p.peerId !== peer.peerId)

  // Add to front
  filtered.unshift({
    ...peer,
    lastUsed: Date.now(),
  })

  // Cap at 5
  const capped = filtered.slice(0, 5)

  saveSettings({ recentPeers: capped })
}

export function setLastPeerId(peerId: string): void {
  saveSettings({ lastPeerId: peerId })
}

export function getLastPeerId(): string | undefined {
  return loadSettings().lastPeerId
}

export function getRecentPeers(): Array<{ alias?: string; peerId: string }> {
  const settings = loadSettings()
  return (settings.recentPeers || []).map((p) => ({
    alias: p.alias,
    peerId: p.peerId,
  }))
}

