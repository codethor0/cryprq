import { useEffect, useCallback } from 'react'
import { CrypRQErrorCode, mapErrorToCode, getErrorDescriptor } from '@/errors/catalog'
import { useAppStore } from '@/store/useAppStore'

export interface ErrorEvent {
  code: CrypRQErrorCode
  message: string
  lastLogs?: string[]
  metadata?: Record<string, unknown>
}

type ErrorHandler = (error: ErrorEvent) => void

class ErrorBus {
  private handlers: Set<ErrorHandler> = new Set()

  subscribe(handler: ErrorHandler): () => void {
    this.handlers.add(handler)
    return () => {
      this.handlers.delete(handler)
    }
  }

  emit(error: ErrorEvent) {
    this.handlers.forEach(handler => handler(error))
  }
}

export const errorBus = new ErrorBus()

export function useErrorBus(handler?: ErrorHandler) {
  const addLog = useAppStore(state => state.addLog)

  useEffect(() => {
    if (!handler) return

    const unsubscribe = errorBus.subscribe(handler)
    return unsubscribe
  }, [handler])

  // Also subscribe to backend errors
  useEffect(() => {
    if (typeof window === 'undefined' || !window.electronAPI) return

    const handleSessionError = (error: { error: string; code: string; lastLogs: string[] }) => {
      const code = mapErrorToCode(error.error, error.lastLogs)
      const descriptor = getErrorDescriptor(code)
      
      errorBus.emit({
        code,
        message: error.error,
        lastLogs: error.lastLogs,
      })

      addLog({
        timestamp: new Date(),
        level: 'error',
        message: `${descriptor.title}: ${error.error}`,
      })
    }

    const handleSessionEnded = (data: { code: number | null; signal: string | null; state: string; lastLogs: string[] }) => {
      if (data.state === 'errored') {
        const code = mapErrorToCode(`Process exited with code ${data.code}`, data.lastLogs)
        const descriptor = getErrorDescriptor(code)
        
        errorBus.emit({
          code,
          message: `Session ended unexpectedly (code: ${data.code}, signal: ${data.signal})`,
          lastLogs: data.lastLogs,
        })

        addLog({
          timestamp: new Date(),
          level: 'error',
          message: `${descriptor.title}: Process exited with code ${data.code}`,
        })
      }
    }

    window.electronAPI.onSessionError(handleSessionError)
    window.electronAPI.onSessionEnded(handleSessionEnded)

    return () => {
      window.electronAPI?.removeAllListeners('session:error')
      window.electronAPI?.removeAllListeners('session:ended')
    }
  }, [addLog])
}

