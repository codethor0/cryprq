/**
 * Telemetry v0: Opt-in event counters (no PII)
 * Writes to ~/.cryprq/telemetry/events-YYYY-MM-DD.jsonl
 */

import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import { app } from 'electron'

const TELEMETRY_DIR = path.join(os.homedir(), '.cryprq', 'telemetry')

// Global flag (set via IPC)
declare global {
  var __TELEMETRY_ENABLED__: boolean
  var __APP_VERSION__: string
}

// Initialize telemetry enabled state (default: false)
globalThis.__TELEMETRY_ENABLED__ = false
globalThis.__APP_VERSION__ = app.getVersion()

function sanitize(obj: any): any {
  try {
    const s = JSON.stringify(obj)
    // Redact secrets
    const redacted = s.replace(
      /(bearer\s+\S+|privKey=\S+|authorization:\s*\S+)/gi,
      '***REDACTED***'
    )
    return JSON.parse(redacted)
  } catch {
    return {}
  }
}

export function emitTelemetry(event: string, data: Record<string, unknown> = {}): void {
  try {
    if (!globalThis.__TELEMETRY_ENABLED__) {
      return // Telemetry disabled
    }

    // Ensure directory exists
    fs.mkdirSync(TELEMETRY_DIR, { recursive: true })

    // Daily file rotation
    const today = new Date().toISOString().slice(0, 10)
    const file = path.join(TELEMETRY_DIR, `events-${today}.jsonl`)

    // Create telemetry record
    const record = {
      v: 1,
      ts: new Date().toISOString(),
      event,
      appVersion: globalThis.__APP_VERSION__,
      platform: process.platform,
      data: sanitize(data),
    }

    // Append to JSONL file
    fs.appendFileSync(file, JSON.stringify(record) + '\n', { encoding: 'utf8' })
  } catch (error) {
    // Never throw - telemetry failures should not affect app
    console.warn('Telemetry emit failed:', error)
  }
}

export function setTelemetryEnabled(enabled: boolean): void {
  globalThis.__TELEMETRY_ENABLED__ = enabled
}

export function isTelemetryEnabled(): boolean {
  return globalThis.__TELEMETRY_ENABLED__ === true
}

