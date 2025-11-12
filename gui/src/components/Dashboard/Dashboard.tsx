import React, { useEffect, useState } from 'react'
import { useAppStore } from '@/store/useAppStore'
import { Charts } from './Charts'
import { useFlagsHook } from '@/store/useFlags'

export const Dashboard: React.FC = () => {
  const { connectionStatus, connect, disconnect, logs } = useAppStore()
  const flags = useFlagsHook()
  const [rotationTimer, setRotationTimer] = useState<number | undefined>(connectionStatus.rotationTimer)
  const [isRotating, setIsRotating] = useState(false)
  const [lastRotationToast, setLastRotationToast] = useState<number>(0)

  useEffect(() => {
    setRotationTimer(connectionStatus.rotationTimer)
  }, [connectionStatus.rotationTimer])

  useEffect(() => {
    if (rotationTimer !== undefined && rotationTimer > 0 && !isRotating) {
      const interval = setInterval(() => {
        setRotationTimer(prev => (prev !== undefined && prev > 0 ? prev - 1 : 0))
      }, 1000)
      return () => clearInterval(interval)
    }
  }, [rotationTimer, isRotating])

  // Listen to rotation events from backend
  useEffect(() => {
    if (typeof window === 'undefined' || !window.electronAPI) return

    const handleRotation = (event: { type: string; nextInSeconds?: number }) => {
      if (event.type === 'rotation.started') {
        setIsRotating(true)
        // Pulse icon animation would go here
      } else if (event.type === 'rotation.completed') {
        setIsRotating(false)
        // Resync countdown
        if (event.nextInSeconds !== undefined) {
          setRotationTimer(event.nextInSeconds)
        }
        // Show toast (prevent duplicates)
        const now = Date.now()
        if (now - lastRotationToast > 2000) {
          const timeStr = new Date().toLocaleTimeString()
          // Toast would be shown here via toastStore
          setLastRotationToast(now)
        }
      } else if (event.type === 'rotation.scheduled' && event.nextInSeconds !== undefined) {
        // Resync countdown from metrics
        setRotationTimer(event.nextInSeconds)
      }
    }

    // Subscribe to rotation events (would need to be exposed from backend service)
    // For now, we'll listen to metrics updates for rotation timer changes
    const metricsInterval = setInterval(async () => {
      if (window.electronAPI && connectionStatus.connected) {
        try {
          const metrics = await window.electronAPI.metricsGet()
          if (metrics.rotationTimer !== undefined) {
            setRotationTimer(metrics.rotationTimer)
          }
        } catch (error) {
          // Ignore metrics errors
        }
      }
    }, 2000) // Resync every 2s

    return () => {
      clearInterval(metricsInterval)
    }
  }, [connectionStatus.connected, lastRotationToast])

  const formatTime = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const statusColor = isRotating ? '#FF9800' : connectionStatus.connected ? '#4CAF50' : '#F44336'
  const statusText = isRotating ? 'Rotating Keys' : connectionStatus.connected ? 'Connected' : 'Disconnected'

  return (
    <div style={{ maxWidth: '1200px' }}>
      <h1 style={{ marginTop: 0, fontSize: '32px', fontWeight: 600 }}>Dashboard</h1>

      {/* Connection Status Card */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '32px',
        marginBottom: '24px',
        border: `2px solid ${statusColor}`,
      }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
          <div>
            <h2 style={{ margin: '0 0 8px', fontSize: '20px', fontWeight: 600 }}>Connection Status</h2>
            <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
              <div style={{
                width: '12px',
                height: '12px',
                borderRadius: '50%',
                backgroundColor: statusColor,
                boxShadow: `0 0 8px ${statusColor}`,
              }} />
              <span style={{ fontSize: '18px', fontWeight: 500 }}>{statusText}</span>
            </div>
          </div>
          <button
            onClick={async () => {
              try {
                if (connectionStatus.connected) {
                  await disconnect()
                } else {
                  await connect()
                }
              } catch (error: any) {
                // Error already logged by store
                alert(error?.message || 'Connection failed')
              }
            }}
            style={{
              padding: '12px 24px',
              backgroundColor: connectionStatus.connected ? '#F44336' : '#1DE9B6',
              color: '#000',
              border: 'none',
              borderRadius: '8px',
              fontSize: '14px',
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            {connectionStatus.connected ? 'Disconnect' : 'Connect'}
          </button>
        </div>

        {connectionStatus.connected && (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '24px' }}>
            {connectionStatus.peerId && (
              <div>
                <div style={{ fontSize: '12px', color: '#B0B0B0', marginBottom: '4px' }}>Peer ID</div>
                <div style={{ fontSize: '14px', fontFamily: 'monospace', wordBreak: 'break-all' }}>
                  {connectionStatus.peerId}
                </div>
              </div>
            )}
            {rotationTimer !== undefined && (
              <div>
                <div style={{ fontSize: '12px', color: '#B0B0B0', marginBottom: '4px' }}>Next Rotation</div>
                <div style={{ fontSize: '20px', fontWeight: 600, color: '#1DE9B6' }}>
                  {formatTime(rotationTimer)}
                </div>
              </div>
            )}
            {connectionStatus.throughput && (
              <>
                <div>
                  <div style={{ fontSize: '12px', color: '#B0B0B0', marginBottom: '4px' }}>Bytes In</div>
                  <div style={{ fontSize: '14px' }}>
                    {(connectionStatus.throughput.bytesIn / 1024).toFixed(2)} KB
                  </div>
                </div>
                <div>
                  <div style={{ fontSize: '12px', color: '#B0B0B0', marginBottom: '4px' }}>Bytes Out</div>
                  <div style={{ fontSize: '14px' }}>
                    {(connectionStatus.throughput.bytesOut / 1024).toFixed(2)} KB
                  </div>
                </div>
              </>
            )}
          </div>
        )}
      </div>

      {/* Charts */}
      {connectionStatus.connected && flags.enableCharts && <Charts />}

      {/* Recent Activity */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Recent Activity</h3>
        <div style={{ maxHeight: '300px', overflowY: 'auto' }}>
          {logs.length === 0 ? (
            <div style={{ color: '#B0B0B0', fontSize: '14px' }}>No activity yet</div>
          ) : (
            logs.slice(-10).reverse().map((log, idx) => (
              <div key={idx} style={{
                padding: '8px 0',
                borderBottom: idx < logs.length - 1 ? '1px solid #333' : 'none',
                fontSize: '13px',
                fontFamily: 'monospace',
              }}>
                <span style={{ color: '#757575', marginRight: '12px' }}>
                  {log.timestamp.toLocaleTimeString()}
                </span>
                <span style={{ color: log.level === 'error' ? '#EF5350' : log.level === 'warn' ? '#FFB74D' : '#B0B0B0' }}>
                  [{log.level.toUpperCase()}]
                </span>
                <span style={{ color: '#E0E0E0', marginLeft: '8px' }}>{log.message}</span>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  )
}

