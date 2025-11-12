import React from 'react'
import { ErrorEvent, getErrorDescriptor } from '@/errors/catalog'
import { useNavigate } from 'react-router-dom'
import { useAppStore } from '@/store/useAppStore'

interface ErrorModalProps {
  error: ErrorEvent | null
  onClose: () => void
  onRestart?: () => void
  onViewLogs?: () => void
}

export const ErrorModal: React.FC<ErrorModalProps> = ({ error, onClose, onRestart, onViewLogs }) => {

  if (!error) return null

  const descriptor = getErrorDescriptor(error.code)

  const handleRemediation = () => {
    switch (error.code) {
      case 'PORT_IN_USE':
        navigate('/settings')
        // Focus port field (would need ref or ID)
        setTimeout(() => {
          const portInput = document.querySelector('[data-field="port"]') as HTMLInputElement
          portInput?.focus()
        }, 100)
        onClose()
        break
      case 'CLI_NOT_FOUND':
        // Open file picker (would need IPC handler)
        console.log('Open file picker to locate binary')
        onClose()
        break
      case 'CLI_EXITED':
        if (onViewLogs) {
          onViewLogs()
        }
        onClose()
        break
      default:
        onClose()
    }
  }

  const handleViewLogs = () => {
    if (openLogs) {
      openLogs()
    }
    onClose()
  }

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.7)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 10000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: '#1E1E1E',
          borderRadius: '12px',
          padding: '32px',
          minWidth: '500px',
          maxWidth: '600px',
          border: '2px solid #F44336',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ margin: '0 0 16px', fontSize: '24px', fontWeight: 600, color: '#E0E0E0' }}>
          {descriptor.title}
        </h2>
        
        <p style={{ margin: '0 0 16px', fontSize: '16px', color: '#B0B0B0', lineHeight: '1.5' }}>
          {descriptor.description}
        </p>

        {descriptor.help && (
          <div style={{
            margin: '16px 0',
            padding: '12px',
            backgroundColor: '#121212',
            borderRadius: '6px',
            fontSize: '14px',
            color: '#B0B0B0',
          }}>
            {descriptor.help}
          </div>
        )}

        {error.lastLogs && error.lastLogs.length > 0 && (
          <details style={{ margin: '16px 0' }}>
            <summary style={{ cursor: 'pointer', color: '#1DE9B6', fontSize: '14px' }}>
              Show error details
            </summary>
            <pre style={{
              marginTop: '8px',
              padding: '12px',
              backgroundColor: '#121212',
              borderRadius: '6px',
              fontSize: '12px',
              fontFamily: 'monospace',
              color: '#E0E0E0',
              maxHeight: '200px',
              overflow: 'auto',
            }}>
              {error.lastLogs.join('\n')}
            </pre>
          </details>
        )}

        <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '24px' }}>
          {(error.code === 'CLI_EXITED' || error.code === 'CLI_NOT_FOUND') && onRestart && (
            <button
              onClick={() => {
                onRestart()
                onClose()
              }}
              style={{
                padding: '10px 20px',
                backgroundColor: '#1DE9B6',
                border: 'none',
                borderRadius: '6px',
                color: '#000',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Restart Session
            </button>
          )}
          
          {error.code === 'CLI_EXITED' && onViewLogs && (
            <button
              onClick={() => {
                onViewLogs()
                onClose()
              }}
              style={{
                padding: '10px 20px',
                backgroundColor: 'transparent',
                border: '1px solid #1DE9B6',
                borderRadius: '6px',
                color: '#1DE9B6',
                cursor: 'pointer',
              }}
            >
              View Logs
            </button>
          )}

          {(error.code === 'PORT_IN_USE' || error.code === 'CLI_NOT_FOUND') && (
            <button
              onClick={handleRemediation}
              style={{
                padding: '10px 20px',
                backgroundColor: '#1DE9B6',
                border: 'none',
                borderRadius: '6px',
                color: '#000',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              {error.code === 'PORT_IN_USE' ? 'Pick Another Port' : 'Locate Binary...'}
            </button>
          )}

          <button
            onClick={onClose}
            style={{
              padding: '10px 20px',
              backgroundColor: 'transparent',
              border: '1px solid #666',
              borderRadius: '6px',
              color: '#E0E0E0',
              cursor: 'pointer',
            }}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  )
}

