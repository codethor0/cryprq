#!/usr/bin/env node

/**
 * Fake CrypRQ backend for E2E testing
 * Exposes Prometheus metrics endpoint and emits JSONL events to stdout
 */

const http = require('http')
const port = 9464

let bytesIn = 0
let bytesOut = 0
let latencyMs = 25
let rotationTimerSeconds = 300
let peerId = 'QmFake1234567890123456789012345678901234567890123456789012'
let connected = false

// Metrics endpoint
const server = http.createServer((req, res) => {
  if (req.url === '/metrics' && req.method === 'GET') {
    const metrics = `# HELP cryprq_bytes_in_total Total bytes received
# TYPE cryprq_bytes_in_total counter
cryprq_bytes_in_total ${bytesIn}

# HELP cryprq_bytes_out_total Total bytes sent
# TYPE cryprq_bytes_out_total counter
cryprq_bytes_out_total ${bytesOut}

# HELP cryprq_latency_ms Current latency in milliseconds
# TYPE cryprq_latency_ms gauge
cryprq_latency_ms ${latencyMs}

# HELP cryprq_rotation_timer_seconds Seconds until next key rotation
# TYPE cryprq_rotation_timer_seconds gauge
cryprq_rotation_timer_seconds ${rotationTimerSeconds}

# HELP cryprq_peer_id Current peer ID
# TYPE cryprq_peer_id gauge
cryprq_peer_id "${peerId}"
`

    res.writeHead(200, { 'Content-Type': 'text/plain' })
    res.end(metrics)
  } else {
    res.writeHead(404)
    res.end('Not found')
  }
})

server.listen(port, () => {
  console.error(`Fake CrypRQ metrics server listening on port ${port}`)
})

// Emit JSONL events to stdout
let eventInterval = null

function emitEvent(type, data) {
  const event = { type, ...data, timestamp: new Date().toISOString() }
  console.log(JSON.stringify(event))
}

function startEventLoop() {
  // Initial status
  setTimeout(() => {
    emitEvent('status', { status: 'connected', peerId })
    connected = true
  }, 1000)

  // Periodic events
  eventInterval = setInterval(() => {
    // Simulate metrics updates
    bytesIn += Math.floor(Math.random() * 1000)
    bytesOut += Math.floor(Math.random() * 500)
    latencyMs = 20 + Math.floor(Math.random() * 20)
    rotationTimerSeconds = Math.max(0, rotationTimerSeconds - 1)

    emitEvent('metric', {
      latencyMs,
      bytesIn,
      bytesOut,
      rotationTimerSeconds,
    })

    // Rotation event when timer reaches 0
    if (rotationTimerSeconds === 0) {
      rotationTimerSeconds = 300
      emitEvent('rotation', { nextInSeconds: 300 })
    }
  }, 1000 + Math.floor(Math.random() * 1000)) // 1-2 seconds
}

// Handle shutdown
process.on('SIGTERM', () => {
  if (eventInterval) {
    clearInterval(eventInterval)
  }
  server.close()
  process.exit(0)
})

process.on('SIGINT', () => {
  if (eventInterval) {
    clearInterval(eventInterval)
  }
  server.close()
  process.exit(0)
})

startEventLoop()

