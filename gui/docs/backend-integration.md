# Backend Integration Guide

This document describes how to integrate the GUI with the CrypRQ CLI backend.

## Architecture Overview

```
┌─────────────────┐
│  Electron GUI   │
│  (React/TS)     │
└────────┬────────┘
         │ IPC / Process Spawn
         ▼
┌─────────────────┐
│  CrypRQ CLI     │
│  (Rust)         │
└─────────────────┘
```

## Integration Points

### 1. Process Management

The GUI spawns and manages the `cryprq` process:

```typescript
import { spawn, ChildProcess } from 'child_process'
import * as path from 'path'

class BackendService {
  private process: ChildProcess | null = null
  private cryprqPath: string

  constructor() {
    // In production, resolve cryprq binary path
    // - macOS: ../target/release/cryprq or bundled binary
    // - Windows: ../target/release/cryprq.exe
    // - Development: assume in PATH or ../target/release/cryprq
    this.cryprqPath = process.platform === 'win32' 
      ? 'cryprq.exe' 
      : 'cryprq'
  }

  async connect(peerMultiaddr?: string): Promise<void> {
    const args = peerMultiaddr
      ? ['--peer', peerMultiaddr]
      : ['--listen', '/ip4/0.0.0.0/udp/9999/quic-v1']

    this.process = spawn(this.cryprqPath, args, {
      cwd: process.cwd(),
      stdio: ['ignore', 'pipe', 'pipe'],
    })

    this.process.stdout?.on('data', (data) => {
      this.parseLogOutput(data.toString())
    })

    this.process.stderr?.on('data', (data) => {
      this.parseLogOutput(data.toString(), 'error')
    })

    this.process.on('exit', (code) => {
      this.emit('status', { connected: false })
    })
  }

  async disconnect(): Promise<void> {
    if (this.process) {
      this.process.kill('SIGTERM')
      this.process = null
    }
  }
}
```

### 2. Log Parsing

Parse CLI output to extract status updates:

```typescript
private parseLogOutput(output: string, level: 'info' | 'error' = 'info') {
  const lines = output.split('\n').filter(Boolean)
  
  for (const line of lines) {
    // Example log formats:
    // "INFO: Connected to peer 12D3KooW..."
    // "INFO: Key rotation completed"
    // "ERROR: Failed to bind to port 9999"
    
    if (line.includes('Connected to peer')) {
      const peerIdMatch = line.match(/peer (\w+)/)
      if (peerIdMatch) {
        this.emit('status', {
          connected: true,
          peerId: peerIdMatch[1],
        })
      }
    }
    
    if (line.includes('Key rotation')) {
      // Reset rotation timer
      this.emit('status', {
        rotationTimer: 300, // 5 minutes default
      })
    }
    
    // Emit log entry
    this.emit('log', {
      timestamp: new Date(),
      level,
      message: line,
    })
  }
}
```

### 3. Metrics Endpoint

Query Prometheus metrics endpoint for real-time stats:

```typescript
async getMetrics(): Promise<Metrics> {
  try {
    const response = await fetch('http://localhost:9464/metrics')
    const text = await response.text()
    
    // Parse Prometheus format:
    // cryprq_bytes_in_total 1234567
    // cryprq_bytes_out_total 987654
    // cryprq_rotation_timer_seconds 245
    
    const bytesIn = this.parseMetric(text, 'cryprq_bytes_in_total')
    const bytesOut = this.parseMetric(text, 'cryprq_bytes_out_total')
    const rotationTimer = this.parseMetric(text, 'cryprq_rotation_timer_seconds')
    
    return {
      throughput: {
        bytesIn,
        bytesOut,
      },
      rotationTimer: Math.floor(rotationTimer),
    }
  } catch (error) {
    console.error('Failed to fetch metrics:', error)
    return {}
  }
}

private parseMetric(text: string, name: string): number {
  const regex = new RegExp(`${name}\\s+(\\d+)`)
  const match = text.match(regex)
  return match ? parseInt(match[1], 10) : 0
}
```

### 4. Configuration Management

Read/write peer configuration:

```typescript
import * as fs from 'fs'
import * as path from 'path'

const CONFIG_PATH = path.join(
  app.getPath('userData'),
  'cryprq',
  'peers.json'
)

async getPeers(): Promise<Peer[]> {
  try {
    const data = await fs.promises.readFile(CONFIG_PATH, 'utf-8')
    return JSON.parse(data)
  } catch (error) {
    // File doesn't exist, return empty array
    return []
  }
}

async addPeer(peer: Peer): Promise<void> {
  const peers = await this.getPeers()
  peers.push(peer)
  await fs.promises.mkdir(path.dirname(CONFIG_PATH), { recursive: true })
  await fs.promises.writeFile(CONFIG_PATH, JSON.stringify(peers, null, 2))
}
```

### 5. Settings Persistence

Store user settings:

```typescript
import { app } from 'electron'

const SETTINGS_PATH = path.join(
  app.getPath('userData'),
  'cryprq',
  'settings.json'
)

async loadSettings(): Promise<AppSettings> {
  try {
    const data = await fs.promises.readFile(SETTINGS_PATH, 'utf-8')
    return JSON.parse(data)
  } catch (error) {
    return defaultSettings
  }
}

async saveSettings(settings: AppSettings): Promise<void> {
  await fs.promises.mkdir(path.dirname(SETTINGS_PATH), { recursive: true })
  await fs.promises.writeFile(
    SETTINGS_PATH,
    JSON.stringify(settings, null, 2)
  )
}
```

## Error Handling

### Port Already in Use

```typescript
private parseLogOutput(output: string) {
  if (output.includes('Address already in use') || 
      output.includes('bind: address already in use')) {
    this.emit('error', {
      type: 'PORT_BLOCKED',
      message: 'Port 9999/UDP is already in use',
      details: 'Another application is using this port. Please change the UDP port in Settings.',
    })
  }
}
```

### Network Unreachable

```typescript
if (output.includes('Network unreachable')) {
  this.emit('error', {
    type: 'NETWORK_ERROR',
    message: 'Network unreachable',
    details: 'Check your network connection and firewall settings.',
  })
}
```

## Testing

### Mock Backend for Development

During development, use a mock backend that simulates CLI behavior:

```typescript
class MockBackendService extends BackendService {
  async connect(peerMultiaddr?: string): Promise<void> {
    // Simulate connection delay
    setTimeout(() => {
      this.emit('status', {
        connected: true,
        peerId: '12D3KooWMockPeerId',
        rotationTimer: 300,
      })
    }, 1000)
  }
  
  // Override other methods with mock implementations
}
```

### Integration Tests

Test the full integration:

```typescript
describe('Backend Integration', () => {
  it('should spawn cryprq process on connect', async () => {
    const backend = new BackendService()
    await backend.connect()
    expect(backend.process).not.toBeNull()
  })
  
  it('should parse connection status from logs', async () => {
    const backend = new BackendService()
    const statusPromise = new Promise(resolve => {
      backend.on('status', resolve)
    })
    
    await backend.connect()
    backend.parseLogOutput('INFO: Connected to peer 12D3KooW...')
    
    const status = await statusPromise
    expect(status.connected).toBe(true)
  })
})
```

## Production Considerations

### Binary Bundling

Bundle the `cryprq` binary with the Electron app:

1. Copy binary to `resources/` directory during build
2. Resolve path relative to `app.getAppPath()` or `process.resourcesPath`
3. Handle platform-specific paths (`.exe` on Windows)

### Auto-Updates

Consider auto-updating the CLI binary separately from the GUI:

- Check for CLI updates on startup
- Download and replace binary if newer version available
- Verify checksums before replacing

### Permissions

On macOS, may need to request network permissions:

```typescript
import { app } from 'electron'

app.on('ready', () => {
  // Request network permissions if needed
  // macOS will prompt user automatically
})
```

## Next Steps

1. ✅ Implement process spawning
2. ✅ Add log parsing
3. ⏳ Integrate metrics polling
4. ⏳ Add configuration file management
5. ⏳ Handle errors gracefully
6. ⏳ Test on Windows and macOS

