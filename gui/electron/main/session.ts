import { spawn, ChildProcessWithoutNullStreams } from 'child_process'
import { BrowserWindow, ipcMain } from 'electron'
import * as path from 'path'
import * as fs from 'fs'
import { appendLog, appendStructuredLog } from './logging'
import { updateTrayFromSession } from './tray-updater'
import { emitTelemetry } from './telemetry'
import type { LogLine } from '../../src/types'

let child: ChildProcessWithoutNullStreams | null = null
let sessionState: 'idle' | 'starting' | 'running' | 'stopping' | 'errored' = 'idle'
let lastSessionState: 'idle' | 'starting' | 'running' | 'stopping' | 'errored' = 'idle'
let lastLogs: string[] = []
const MAX_LOG_BUFFER = 20

interface SessionStartArgs {
  binPath?: string
  binArgs: string[]
  listenMultiaddr?: string
  peerMultiaddr?: string
}

function findCryprqBinary(): string | null {
  // In development, try relative paths
  const devPaths = [
    path.join(__dirname, '../../../target/release/cryprq'),
    path.join(__dirname, '../../../target/debug/cryprq'),
    path.join(process.cwd(), 'target/release/cryprq'),
    path.join(process.cwd(), 'target/debug/cryprq'),
  ]

  // In production, bundled binary location
  const prodPaths = [
    path.join(process.resourcesPath || __dirname, 'cryprq'),
    path.join(__dirname, '../resources/cryprq'),
  ]

  const allPaths = process.env.NODE_ENV === 'development' ? devPaths : prodPaths

  for (const binPath of allPaths) {
    if (fs.existsSync(binPath)) {
      return binPath
    }
  }

  // Fallback: assume in PATH
  return process.platform === 'win32' ? 'cryprq.exe' : 'cryprq'
}

function addLogLine(line: string) {
  lastLogs.push(line)
  if (lastLogs.length > MAX_LOG_BUFFER) {
    lastLogs.shift()
  }
  
  // Write to file log (legacy format, will be converted)
  appendLog({
    ts: new Date().toISOString(),
    level: 'info',
    source: 'cli',
    msg: line,
  })
}

function logSessionState(state: string, peerId?: string, exitCode?: number) {
  appendStructuredLog({
    ts: new Date().toISOString(),
    lvl: 'info',
    src: 'app',
    event: 'session.state',
    msg: `state=${state}`,
    data: {
      state,
      ...(peerId && { peerId }),
      ...(exitCode !== undefined && { exitCode }),
    },
  })
  
  // Emit telemetry events
  const newState = state as 'idle' | 'starting' | 'running' | 'stopping' | 'errored'
  
  // Connect: idle/starting -> running
  if (newState === 'running' && lastSessionState !== 'running') {
    emitTelemetry('connect', { peerId })
  }
  
  // Disconnect: running -> idle
  if (newState === 'idle' && lastSessionState === 'running') {
    emitTelemetry('disconnect', {})
  }
  
  // Error: any -> errored
  if (newState === 'errored') {
    emitTelemetry('error', { 
      code: exitCode !== undefined ? `EXIT_${exitCode}` : 'UNKNOWN',
      exitCode 
    })
  }
  
  lastSessionState = newState
  
  // Update tray
  updateTrayFromSession(
    state as 'idle' | 'starting' | 'running' | 'stopping' | 'errored' | 'rotating',
    peerId
  )
}

ipcMain.handle('session:start', async (_e, args: SessionStartArgs) => {
  if (child || sessionState !== 'idle') {
    return { ok: false, error: 'ALREADY_RUNNING' }
  }

  const binPath = args.binPath || findCryprqBinary()
  if (!binPath) {
    return { ok: false, error: 'BINARY_NOT_FOUND', message: 'CrypRQ binary not found' }
  }

  // Build command args
  const cmdArgs: string[] = []
  
  if (args.peerMultiaddr) {
    cmdArgs.push('--peer', args.peerMultiaddr)
  } else if (args.listenMultiaddr) {
    cmdArgs.push('--listen', args.listenMultiaddr)
  } else {
    cmdArgs.push('--listen', '/ip4/0.0.0.0/udp/9999/quic-v1')
  }

  // Add metrics endpoint flag if supported
  cmdArgs.push('--metrics-addr', '127.0.0.1:9464')

  try {
    sessionState = 'starting'
    lastLogs = []

    child = spawn(binPath, cmdArgs, {
      env: { ...process.env },
      stdio: ['ignore', 'pipe', 'pipe'],
    }) as ChildProcessWithoutNullStreams

    child.stdout.setEncoding('utf8')
    child.stderr.setEncoding('utf8')

    child.stdout.on('data', (chunk: string) => {
      const lines = chunk.split('\n').filter(Boolean)
      for (const line of lines) {
        // Try to parse as JSON event
        try {
          const evt = JSON.parse(line)
          BrowserWindow.getAllWindows().forEach(w =>
            w.webContents.send('session:event', evt)
          )
          
          // Emit telemetry for rotation events
          if (evt.type === 'rotation' || evt.event === 'rotation.completed') {
            emitTelemetry('rotation.completed', {})
          }
          
          // Log structured events
          appendLog({
            ts: new Date().toISOString(),
            level: 'info',
            source: 'cli',
            msg: JSON.stringify(evt),
            meta: evt,
          })
        } catch {
          // Not JSON, send as raw log
          addLogLine(line)
          BrowserWindow.getAllWindows().forEach(w =>
            w.webContents.send('session:log', { level: 'info', message: line })
          )
        }
      }
    })

    child.stderr.on('data', (chunk: string) => {
      const lines = chunk.split('\n').filter(Boolean)
      for (const line of lines) {
        addLogLine(line)
        
        // Also write error-level log
        appendLog({
          ts: new Date().toISOString(),
          level: 'error',
          source: 'cli',
          msg: line,
        })
        
        BrowserWindow.getAllWindows().forEach(w =>
          w.webContents.send('session:log', { level: 'error', message: line })
        )
      }
    })

    child.on('exit', (code, signal) => {
      sessionState = code === 0 ? 'idle' : 'errored'
      
      BrowserWindow.getAllWindows().forEach(w => {
        w.webContents.send('session:ended', {
          code,
          signal,
          state: sessionState,
          lastLogs: lastLogs.slice(-MAX_LOG_BUFFER),
        })
        w.webContents.send('session:state-changed', {
          status: sessionState,
        })
      })
      
      child = null
    })

    child.on('error', (error) => {
      sessionState = 'errored'
      addLogLine(`Process error: ${error.message}`)
      
      BrowserWindow.getAllWindows().forEach(w =>
        w.webContents.send('session:error', {
          error: error.message,
          code: 'PROCESS_ERROR',
          lastLogs: lastLogs.slice(-MAX_LOG_BUFFER),
        })
      )
      
      child = null
    })

    // Wait a moment to see if process starts successfully
    await new Promise(resolve => setTimeout(resolve, 500))
    
    if (child && !child.killed) {
      sessionState = 'running'
      
      // Notify tray of state change
      BrowserWindow.getAllWindows().forEach(w =>
        w.webContents.send('session:state-changed', {
          status: 'running',
        })
      )
      
      return { ok: true }
    } else {
      sessionState = 'errored'
      BrowserWindow.getAllWindows().forEach(w =>
        w.webContents.send('session:state-changed', {
          status: 'errored',
        })
      )
      return { ok: false, error: 'PROCESS_EXITED', lastLogs }
    }
  } catch (error: any) {
    sessionState = 'errored'
    return { ok: false, error: 'SPAWN_FAILED', message: error.message }
  }
})

// Export stopSession for use in main.ts kill-switch
export async function stopSession(): Promise<{ ok: boolean; error?: string }> {
  if (!child || sessionState === 'idle') {
    return { ok: false, error: 'NOT_RUNNING' }
  }

  sessionState = 'stopping'
  
  return new Promise((resolve) => {
    const timeout = setTimeout(() => {
      if (child) {
        child.kill('SIGKILL')
      }
      sessionState = 'idle'
      BrowserWindow.getAllWindows().forEach(w =>
        w.webContents.send('session:state-changed', {
          status: 'idle',
        })
      )
      resolve({ ok: true, force: true })
    }, 1000)

    child?.on('exit', () => {
      clearTimeout(timeout)
      sessionState = 'idle'
      child = null
      BrowserWindow.getAllWindows().forEach(w =>
        w.webContents.send('session:state-changed', {
          status: 'idle',
        })
      )
      resolve({ ok: true })
    })

    child?.kill('SIGTERM')
  })
}

ipcMain.handle('session:stop', async () => {
  return stopSession()
})

ipcMain.handle('session:get', async () => {
  return {
    state: sessionState,
    pid: child?.pid || null,
    lastLogs: lastLogs.slice(-MAX_LOG_BUFFER),
  }
})

ipcMain.handle('session:restart', async (_e, args: SessionStartArgs) => {
  // Stop current session if running
  if (child && sessionState !== 'idle') {
    sessionState = 'stopping'
    await new Promise<void>((resolve) => {
      const timeout = setTimeout(() => {
        if (child) {
          child.kill('SIGKILL')
        }
        resolve()
      }, 1000)
      child?.on('exit', () => {
        clearTimeout(timeout)
        sessionState = 'idle'
        child = null
        resolve()
      })
      child?.kill('SIGTERM')
    })
    await new Promise(r => setTimeout(r, 500))
  }
  
  // Start new session (reuse start handler logic)
  if (sessionState !== 'idle') {
    return { ok: false, error: 'ALREADY_RUNNING' }
  }

  const binPath = args.binPath || findCryprqBinary()
  if (!binPath) {
    return { ok: false, error: 'BINARY_NOT_FOUND', message: 'CrypRQ binary not found' }
  }

  const cmdArgs: string[] = []
  if (args.peerMultiaddr) {
    cmdArgs.push('--peer', args.peerMultiaddr)
  } else if (args.listenMultiaddr) {
    cmdArgs.push('--listen', args.listenMultiaddr)
  } else {
    cmdArgs.push('--listen', '/ip4/0.0.0.0/udp/9999/quic-v1')
  }
  cmdArgs.push('--metrics-addr', '127.0.0.1:9464')

  try {
    sessionState = 'starting'
    lastLogs = []

    child = spawn(binPath, cmdArgs, {
      env: { ...process.env },
      stdio: ['ignore', 'pipe', 'pipe'],
    }) as ChildProcessWithoutNullStreams

    child.stdout.setEncoding('utf8')
    child.stderr.setEncoding('utf8')

    child.stdout.on('data', (chunk: string) => {
      const lines = chunk.split('\n').filter(Boolean)
      for (const line of lines) {
        // Try to parse as JSON event
        try {
          const evt = JSON.parse(line)
          BrowserWindow.getAllWindows().forEach(w =>
            w.webContents.send('session:event', evt)
          )
          
          // Emit telemetry for rotation events
          if (evt.type === 'rotation' || evt.event === 'rotation.completed') {
            emitTelemetry('rotation.completed', {})
          }
          
          // Log structured events
          appendLog({
            ts: new Date().toISOString(),
            level: 'info',
            source: 'cli',
            msg: JSON.stringify(evt),
            meta: evt,
          })
        } catch {
          // Not JSON, send as raw log
          addLogLine(line)
          BrowserWindow.getAllWindows().forEach(w =>
            w.webContents.send('session:log', { level: 'info', message: line })
          )
        }
      }
    })

    child.stderr.on('data', (chunk: string) => {
      const lines = chunk.split('\n').filter(Boolean)
      for (const line of lines) {
        addLogLine(line)
        
        // Also write error-level log
        appendLog({
          ts: new Date().toISOString(),
          level: 'error',
          source: 'cli',
          msg: line,
        })
        
        BrowserWindow.getAllWindows().forEach(w =>
          w.webContents.send('session:log', { level: 'error', message: line })
        )
      }
    })

    child.on('exit', (code, signal) => {
      sessionState = code === 0 ? 'idle' : 'errored'
      
      logSessionState(sessionState, undefined, code || undefined)
      
      appendStructuredLog({
        ts: new Date().toISOString(),
        lvl: code === 0 ? 'info' : 'error',
        src: 'app',
        event: 'session.error',
        msg: code === 0 ? 'Session ended normally' : `Session ended with error (code ${code})`,
        data: { exitCode: code, signal: signal?.toString() },
      })
      
      BrowserWindow.getAllWindows().forEach(w => {
        w.webContents.send('session:ended', {
          code,
          signal,
          state: sessionState,
          lastLogs: lastLogs.slice(-MAX_LOG_BUFFER),
        })
        w.webContents.send('session:state-changed', {
          status: sessionState,
        })
      })
      child = null
    })

    child.on('error', (error) => {
      sessionState = 'errored'
      addLogLine(`Process error: ${error.message}`)
      
      logSessionState('errored')
      
      appendStructuredLog({
        ts: new Date().toISOString(),
        lvl: 'error',
        src: 'app',
        event: 'session.error',
        msg: `Process error: ${error.message}`,
        data: { error: error.message, code: 'PROCESS_ERROR' },
      })
      
      BrowserWindow.getAllWindows().forEach(w => {
        w.webContents.send('session:error', {
          error: error.message,
          code: 'PROCESS_ERROR',
          lastLogs: lastLogs.slice(-MAX_LOG_BUFFER),
        })
        w.webContents.send('session:state-changed', {
          status: 'errored',
        })
      })
      child = null
    })

    await new Promise(resolve => setTimeout(resolve, 500))
    
    if (child && !child.killed) {
      sessionState = 'running'
      
      // Notify tray of state change
      BrowserWindow.getAllWindows().forEach(w =>
        w.webContents.send('session:state-changed', {
          status: 'running',
        })
      )
      
      return { ok: true }
    } else {
      sessionState = 'errored'
      BrowserWindow.getAllWindows().forEach(w =>
        w.webContents.send('session:state-changed', {
          status: 'errored',
        })
      )
      return { ok: false, error: 'PROCESS_EXITED', lastLogs }
    }
  } catch (error: any) {
    sessionState = 'errored'
    return { ok: false, error: 'SPAWN_FAILED', message: error.message }
  }
})


// Dev hook for fault injection testing
ipcMain.handle('dev:session:simulateExit', async (_e, args: { code?: number; signal?: string }) => {
  if (!child) {
    return { ok: false, error: 'NO_SESSION' }
  }
  
  const code = args.code ?? 0
  const signal = args.signal ?? null
  
  // Simulate exit by killing the process
  if (signal) {
    child.kill(signal as NodeJS.Signals)
  } else {
    child.kill()
  }
  
  return { ok: true }
})
