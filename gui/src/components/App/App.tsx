import React, { useState, useEffect } from 'react'
import { BrowserRouter, Routes, Route, useNavigate } from 'react-router-dom'
import { MainLayout } from '../Layout/MainLayout'
import { DebugConsole } from '../Debug/DebugConsole'
import { Dashboard } from '../Dashboard/Dashboard'
import { Peers } from '../Peers/Peers'
import { Settings } from '../Settings/Settings'
import { Toaster } from '../Toast/Toaster'
import { ErrorModal } from '../ErrorModal/ErrorModal'
import { useErrorBus, ErrorEvent } from '@/hooks/useErrorBus'
import { toastStore } from '@/store/toastStore'
import { CrypRQErrorCode, getErrorDescriptor } from '@/errors/catalog'
import { useAppStore } from '@/store/useAppStore'
import LogsPanel from '@/components/Logs/LogsPanel'

export const App: React.FC = () => {
  const [currentError, setCurrentError] = useState<ErrorEvent | null>(null)
  const { connect, disconnect, restartSession, connectionStatus, structuredLogs, openLogsPanel } = useAppStore()
  const navigate = useNavigate()

  useErrorBus((error: ErrorEvent) => {
    // Show blocking modal for critical errors
    const blockingErrors: CrypRQErrorCode[] = [
      'PORT_IN_USE',
      'CLI_NOT_FOUND',
      'CLI_EXITED',
      'PERMISSION_DENIED',
    ]

    if (blockingErrors.includes(error.code)) {
      setCurrentError(error)
    } else {
      // Show toast for non-blocking errors
      const descriptor = getErrorDescriptor(error.code)
      toastStore.getState().addToast({
        type: error.code === 'METRICS_TIMEOUT' ? 'warning' : 'error',
        title: descriptor.title,
        message: descriptor.description,
        duration: 6000,
      })
    }
  })

  const handleRestart = async () => {
    try {
      await restartSession()
    } catch (error) {
      console.error('Failed to restart session:', error)
    }
  }

  const handleViewLogs = () => {
    if (currentError) {
      const errorTime = new Date()
      const timeRange = {
        start: new Date(errorTime.getTime() - 5 * 60 * 1000), // 5 minutes before
        end: new Date(errorTime.getTime() + 1 * 60 * 1000), // 1 minute after
      }
      
      // Extract keywords from error
      const keywords = currentError.lastLogs?.join(' ') || currentError.message || ''
      const searchTerms = ['exitCode', 'EADDRINUSE', 'error', 'failed', 'crash'].filter(term =>
        keywords.toLowerCase().includes(term.toLowerCase())
      )
      
      navigate('/logs')
      if (openLogsPanel) {
        openLogsPanel(searchTerms.join(' '), timeRange)
      }
    } else {
      navigate('/logs')
    }
  }

  // Handle tray actions
  useEffect(() => {
    if (typeof window === 'undefined' || !window.electronAPI) return

    const handleTrayToggleConnect = async () => {
      try {
        if (connectionStatus.connected) {
          await disconnect()
        } else {
          await connect()
        }
      } catch (error) {
        console.error('Tray toggle connect failed:', error)
      }
    }

    const handleTraySwitchPeer = async (peerId: string) => {
      try {
        await restartSession(undefined, undefined, peerId)
      } catch (error) {
        console.error('Tray switch peer failed:', error)
      }
    }

    window.electronAPI.onTrayToggleConnect(handleTrayToggleConnect)
    window.electronAPI.onTraySwitchPeer(handleTraySwitchPeer)

    return () => {
      window.electronAPI?.removeAllListeners('tray:toggleConnect')
      window.electronAPI?.removeAllListeners('tray:switchPeer')
    }
  }, [connectionStatus.connected, connect, disconnect, restartSession])

  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<MainLayout />}>
            <Route index element={<Dashboard />} />
            <Route path="peers" element={<Peers />} />
            <Route path="settings" element={<Settings />} />
            <Route
              path="logs"
              element={
                <LogsPanel
                  lines={structuredLogs}
                  onClear={() => useAppStore.getState().clearLogs()}
                />
              }
            />
          </Route>
        </Routes>
      </BrowserRouter>
      <DebugConsole />
      <Toaster />
      <ErrorModal
        error={currentError}
        onClose={() => setCurrentError(null)}
        onRestart={handleRestart}
        onViewLogs={handleViewLogs}
      />
    </>
  )
}

