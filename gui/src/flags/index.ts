/**
 * Feature Flags System
 * Runtime feature toggles via JSON file or ENV variable
 */

import * as fs from 'fs'
import * as path from 'path'

export interface Flags {
  enableCharts: boolean
  enableTrayEnhancements: boolean
  enableNewToasts: boolean
}

const defaults: Flags = {
  enableCharts: true,
  enableTrayEnhancements: true,
  enableNewToasts: true,
}

// In Electron main process, use app.getAppPath(); in renderer, use relative path
function getFlagsPath(): string {
  if (!path) {
    // Browser context - return default path (won't be used)
    return 'config/flags.json'
  }
  if (typeof window !== 'undefined' && window.electronAPI) {
    // Renderer process - flags.json should be in app root
    return path.resolve(process.cwd(), 'config/flags.json')
  }
  // Main process or Node.js
  const appPath = process.env.APP_PATH || process.cwd()
  return path.resolve(appPath, 'config/flags.json')
}

export function loadFlags(): Flags {
  try {
    // ENV override (highest priority)
    const envFlags = typeof process !== 'undefined' && process.env.CRYPRQ_FLAGS
      ? JSON.parse(process.env.CRYPRQ_FLAGS)
      : {}

    // File-based flags (only if fs is available - main process only)
    if (fs && path) {
      const flagsPath = getFlagsPath()
      const fileFlags = fs.existsSync(flagsPath)
        ? JSON.parse(fs.readFileSync(flagsPath, 'utf8'))
        : {}
      // Merge: defaults < file < env
      return { ...defaults, ...fileFlags, ...envFlags }
    }

    // In browser/renderer, return defaults + env only
    return { ...defaults, ...envFlags }
  } catch (error) {
    console.warn('Failed to load flags:', error)
    return defaults
  }
}

export function watchFlags(onChange: (f: Flags) => void): () => void {
  if (!fs || !path) {
    // Browser context - return no-op
    return () => {}
  }

  const flagsPath = getFlagsPath()
  if (!fs.existsSync(flagsPath)) {
    return () => {} // No-op cleanup
  }

  try {
    const watcher = fs.watch(flagsPath, { persistent: false }, () => {
      try {
        const newFlags = loadFlags()
        onChange(newFlags)
      } catch (error) {
        console.warn('Failed to reload flags:', error)
      }
    })

    return () => {
      watcher.close()
    }
  } catch (error) {
    console.warn('Failed to watch flags file:', error)
    return () => {} // No-op cleanup
  }
}

