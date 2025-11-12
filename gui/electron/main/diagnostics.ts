import { app, dialog, ipcMain } from 'electron'
import * as fs from 'fs'
import * as os from 'os'
import * as path from 'path'
import AdmZip from 'adm-zip'
import { getLogFiles, readTail, readStructuredLogs, redactSecrets } from './logging'
import { loadSettings } from './settings'

export async function exportDiagnostics(): Promise<{ ok: boolean; path?: string }> {
  const zip = new AdmZip()
  const now = new Date()
  const ts = now.toISOString().replace(/[-:T]/g, '').slice(0, 13)
  const defaultName = `cryprq-diagnostics-${ts}.zip`

  // System info
  const systemInfo = {
    os: `${os.type()} ${os.release()} (${os.arch()})`,
    appVersion: app.getVersion(),
    electron: process.versions.electron,
    chrome: process.versions.chrome,
    node: process.versions.node,
    platform: process.platform,
  }
  zip.addFile('system-info.json', Buffer.from(JSON.stringify(systemInfo, null, 2)))

  // Settings (redacted)
  try {
    const settings = loadSettings()
    const settingsStr = JSON.stringify(settings, null, 2)
    const redacted = redactSecrets(settingsStr)
    zip.addFile('settings.json', Buffer.from(redacted))
  } catch (error) {
    zip.addFile('settings.json', Buffer.from(JSON.stringify({ error: 'Failed to load settings' }, null, 2)))
  }

  // Last 24h log files
  const logFiles = getLogFiles()
  const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000

  for (const logFile of logFiles) {
    try {
      const stats = fs.statSync(logFile)
      if (stats.mtimeMs >= oneDayAgo) {
        const content = fs.readFileSync(logFile, 'utf8')
        const redacted = redactSecrets(content)
        const fileName = path.basename(logFile)
        zip.addFile(`logs/${fileName}`, Buffer.from(redacted))
      }
    } catch (error) {
      // Skip files that can't be read
    }
  }

  // Session summary from structured logs
  try {
    const now = new Date()
    const oneDayAgo = new Date(now.getTime() - 24 * 60 * 60 * 1000)
    const structuredLogs = await readStructuredLogs({
      since: oneDayAgo,
      until: now,
      limit: 50,
    })
    
    // Compute session statistics
    const stateTransitions: Array<{ event: string; ts: string; data?: any }> = []
    const sessions: Array<{ start: string; end?: string; state: string; exitCode?: number }> = []
    let currentSession: { start: string; state: string } | null = null
    let rotations = 0
    let totalBytesIn = 0
    let totalBytesOut = 0
    let latencySum = 0
    let latencyCount = 0
    
    for (const entry of structuredLogs) {
      stateTransitions.push({
        event: entry.event,
        ts: entry.ts,
        data: entry.data,
      })
      
      if (entry.event === 'session.state') {
        const state = entry.data?.state as string
        if (state === 'starting' && !currentSession) {
          currentSession = { start: entry.ts, state }
        } else if (currentSession && (state === 'idle' || state === 'errored')) {
          sessions.push({
            ...currentSession,
            end: entry.ts,
            state,
            exitCode: entry.data?.exitCode as number | undefined,
          })
          currentSession = null
        } else if (currentSession) {
          currentSession.state = state
        }
      }
      
      if (entry.event === 'rotation.completed') {
        rotations++
      }
      
      if (entry.event === 'metrics.tick' && entry.data) {
        totalBytesIn += (entry.data.bytesIn as number) || 0
        totalBytesOut += (entry.data.bytesOut as number) || 0
        if (entry.data.latencyMs) {
          latencySum += entry.data.latencyMs as number
          latencyCount++
        }
      }
    }
    
    // Compute durations
    const durations: Record<string, number> = {}
    for (let i = 0; i < stateTransitions.length - 1; i++) {
      const curr = stateTransitions[i]
      const next = stateTransitions[i + 1]
      if (curr.event === 'session.state' && next.event === 'session.state') {
        const duration = new Date(next.ts).getTime() - new Date(curr.ts).getTime()
        const key = `${curr.data?.state}->${next.data?.state}`
        if (duration > 0) {
          durations[key] = (durations[key] || 0) + duration
        }
      }
    }
    
    const sessionSummary = {
      timestamp: new Date().toISOString(),
      last50Events: stateTransitions.slice(-50),
      sessions: {
        total: sessions.length,
        failures: sessions.filter(s => s.state === 'errored').length,
        meanTimeToConnect: sessions.length > 0
          ? sessions.reduce((sum, s) => {
              const duration = s.end ? new Date(s.end).getTime() - new Date(s.start).getTime() : 0
              return sum + duration
            }, 0) / sessions.length
          : 0,
      },
      rotations: {
        count: rotations,
        averageInterval: rotations > 0 ? 300 : 0, // Default 5min, could compute from logs
      },
      metrics: {
        totalBytesIn,
        totalBytesOut,
        averageLatency: latencyCount > 0 ? latencySum / latencyCount : 0,
      },
      stateDurations: durations,
    }
    
    zip.addFile('session-summary.json', Buffer.from(JSON.stringify(sessionSummary, null, 2)))
  } catch (error: any) {
    zip.addFile('session-summary.json', Buffer.from(JSON.stringify({ error: 'Failed to generate session summary', message: error.message }, null, 2)))
  }
  
  // Metrics snapshot (last observed metrics)
  try {
    const recentMetrics = await readStructuredLogs({
      event: 'metrics.tick',
      limit: 1,
    })
    
    const metricsSnapshot = recentMetrics.length > 0
      ? {
          timestamp: recentMetrics[0].ts,
          ...recentMetrics[0].data,
        }
      : {
          timestamp: new Date().toISOString(),
          message: 'No metrics available',
        }
    
    zip.addFile('metrics-snapshot.json', Buffer.from(JSON.stringify(metricsSnapshot, null, 2)))
  } catch (error: any) {
    zip.addFile('metrics-snapshot.json', Buffer.from(JSON.stringify({ error: 'Failed to generate metrics snapshot', message: error.message }, null, 2)))
  }
  
  // README
  const readme = `CrypRQ Diagnostics Export
Generated: ${new Date().toISOString()}

Contents:
- logs/: Last 24 hours of log files (JSONL format, secrets redacted)
- system-info.json: System and application version information
- settings.json: Application settings (secrets redacted)
- session-summary.json: Session statistics, state transitions, and metrics
- metrics-snapshot.json: Last observed metrics

How to share with support:
1. Review settings.json and logs/ to ensure no sensitive data remains
2. All secrets (bearer tokens, private keys, etc.) are automatically redacted
3. Share this zip file via secure channel
4. Include a description of the issue you're experiencing

Note: All files are already redacted of secrets. However, review logs/ for any
application-specific sensitive information before sharing.
`
  zip.addFile('README.txt', Buffer.from(readme))

  // Show save dialog
  const { filePath, canceled } = await dialog.showSaveDialog({
    defaultPath: path.join(os.homedir(), defaultName),
    filters: [{ name: 'ZIP Archive', extensions: ['zip'] }],
  })

  if (canceled || !filePath) {
    return { ok: false }
  }

  zip.writeZip(filePath)

  return { ok: true, path: filePath }
}

// Register IPC handler
ipcMain.handle('diagnostics:export', exportDiagnostics)

