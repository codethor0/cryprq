import { ipcMain } from 'electron'

interface PrometheusMetrics {
  bytesIn?: number
  bytesOut?: number
  rotationTimer?: number
  latency?: number
  peerId?: string
}

let metricsCache: PrometheusMetrics = {}
let metricsPollInterval: NodeJS.Timeout | null = null

async function fetchMetrics(): Promise<PrometheusMetrics> {
  try {
    const response = await fetch('http://localhost:9464/metrics')
    const text = await response.text()
    
    const metrics: PrometheusMetrics = {}
    
    // Parse Prometheus format:
    // cryprq_bytes_in_total 1234567
    // cryprq_bytes_out_total 987654
    // cryprq_rotation_timer_seconds 245
    // cryprq_latency_ms 42
    // cryprq_peer_id "12D3KooW..."
    
    const bytesInMatch = text.match(/cryprq_bytes_in_total\s+(\d+)/)
    if (bytesInMatch) {
      metrics.bytesIn = parseInt(bytesInMatch[1], 10)
    }
    
    const bytesOutMatch = text.match(/cryprq_bytes_out_total\s+(\d+)/)
    if (bytesOutMatch) {
      metrics.bytesOut = parseInt(bytesOutMatch[1], 10)
    }
    
    const rotationTimerMatch = text.match(/cryprq_rotation_timer_seconds\s+(\d+)/)
    if (rotationTimerMatch) {
      metrics.rotationTimer = parseInt(rotationTimerMatch[1], 10)
    }
    
    const latencyMatch = text.match(/cryprq_latency_ms\s+(\d+)/)
    if (latencyMatch) {
      metrics.latency = parseInt(latencyMatch[1], 10)
    }
    
    const peerIdMatch = text.match(/cryprq_peer_id\s+"([^"]+)"/)
    if (peerIdMatch) {
      metrics.peerId = peerIdMatch[1]
    }
    
    return metrics
  } catch (error) {
    // Metrics endpoint not available or error
    return {}
  }
}

function startMetricsPolling(intervalMs: number = 2000) {
  if (metricsPollInterval) {
    clearInterval(metricsPollInterval)
  }

  metricsPollInterval = setInterval(async () => {
    const metrics = await fetchMetrics()
    metricsCache = { ...metricsCache, ...metrics }
  }, intervalMs)

  // Initial fetch
  fetchMetrics().then(metrics => {
    metricsCache = metrics
  })
}

function stopMetricsPolling() {
  if (metricsPollInterval) {
    clearInterval(metricsPollInterval)
    metricsPollInterval = null
  }
}

ipcMain.handle('metrics:get', async () => {
  return metricsCache
})

ipcMain.handle('metrics:start', async (_e, intervalMs?: number) => {
  startMetricsPolling(intervalMs)
  return { ok: true }
})

ipcMain.handle('metrics:stop', async () => {
  stopMetricsPolling()
  return { ok: true }
})

export { startMetricsPolling, stopMetricsPolling, fetchMetrics }

