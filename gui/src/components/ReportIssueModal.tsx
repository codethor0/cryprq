import React, { useState, useEffect } from 'react'
import { supportToken } from '@/utils/supportToken'
import { toastStore } from '@/store/toastStore'

interface ReportIssueModalProps {
  onClose: () => void
  appVersion: string
}

export const ReportIssueModal: React.FC<ReportIssueModalProps> = ({ onClose, appVersion }) => {
  const [token, setToken] = useState<string>('')
  const [exporting, setExporting] = useState(false)
  const [exportPath, setExportPath] = useState<string | null>(null)

  useEffect(() => {
    // Generate support token
    setToken(supportToken(appVersion))
  }, [appVersion])

  const handleExportDiagnostics = async () => {
    if (!window.electronAPI) return
    
    setExporting(true)
    try {
      const result = await window.electronAPI.diagnosticsExport()
      if (result.ok && result.path) {
        setExportPath(result.path)
        toastStore.getState().addToast({
          type: 'success',
          title: 'Diagnostics Exported',
          message: 'Diagnostics file exported successfully',
          duration: 3000,
        })
      } else {
        toastStore.getState().addToast({
          type: 'error',
          title: 'Export Failed',
          message: 'Failed to export diagnostics',
          duration: 4000,
        })
      }
    } catch (error: any) {
      toastStore.getState().addToast({
        type: 'error',
        title: 'Export Failed',
        message: error?.message || 'Failed to export diagnostics',
        duration: 4000,
      })
    } finally {
      setExporting(false)
    }
  }

  const handleCopyToken = async () => {
    try {
      await navigator.clipboard.writeText(token)
      toastStore.getState().addToast({
        type: 'success',
        title: 'Token Copied',
        message: 'Support token copied to clipboard',
        duration: 2000,
      })
    } catch (error) {
      toastStore.getState().addToast({
        type: 'error',
        title: 'Copy Failed',
        message: 'Failed to copy token to clipboard',
        duration: 3000,
      })
    }
  }

  const handleCopyPath = async () => {
    if (!exportPath) return
    
    try {
      await navigator.clipboard.writeText(exportPath)
      toastStore.getState().addToast({
        type: 'success',
        title: 'Path Copied',
        message: 'Export path copied to clipboard',
        duration: 2000,
      })
    } catch (error) {
      toastStore.getState().addToast({
        type: 'error',
        title: 'Copy Failed',
        message: 'Failed to copy path to clipboard',
        duration: 3000,
      })
    }
  }

  const handleOpenFolder = async () => {
    if (!exportPath || !window.electronAPI) return
    
    // Use shell API if available, otherwise just show path
    if (window.electronAPI.shellShowItemInFolder) {
      await window.electronAPI.shellShowItemInFolder(exportPath)
    } else {
      toastStore.getState().addToast({
        type: 'info',
        title: 'Export Path',
        message: exportPath,
        duration: 5000,
      })
    }
  }

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.7)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
    }}>
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        maxWidth: '600px',
        width: '90%',
      }}>
        <h2 style={{ margin: '0 0 16px', fontSize: '20px', fontWeight: 600 }}>Report an Issue</h2>
        <p style={{ fontSize: '14px', color: '#B0B0B0', marginBottom: '24px' }}>
          To help us diagnose the issue, please export diagnostics and include your support token when contacting support.
        </p>

        {/* Support Token */}
        <div style={{ marginBottom: '24px' }}>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Support Token
          </label>
          <div style={{ display: 'flex', gap: '8px' }}>
            <input
              type="text"
              value={token}
              readOnly
              style={{
                flex: 1,
                padding: '10px',
                backgroundColor: '#121212',
                border: '1px solid #333',
                borderRadius: '6px',
                color: '#E0E0E0',
                fontSize: '14px',
                fontFamily: 'monospace',
              }}
            />
            <button
              onClick={handleCopyToken}
              style={{
                padding: '10px 20px',
                backgroundColor: '#333',
                color: '#E0E0E0',
                border: 'none',
                borderRadius: '6px',
                fontSize: '14px',
                cursor: 'pointer',
              }}
            >
              Copy
            </button>
          </div>
        </div>

        {/* Export Diagnostics */}
        <div style={{ marginBottom: '24px' }}>
          <button
            onClick={handleExportDiagnostics}
            disabled={exporting}
            style={{
              width: '100%',
              padding: '12px 24px',
              backgroundColor: exporting ? '#666' : '#1DE9B6',
              color: exporting ? '#999' : '#000',
              border: 'none',
              borderRadius: '6px',
              fontSize: '14px',
              fontWeight: 600,
              cursor: exporting ? 'not-allowed' : 'pointer',
            }}
          >
            {exporting ? 'Exporting...' : 'Export Diagnostics…'}
          </button>
          {exportPath && (
            <div style={{ marginTop: '12px', fontSize: '12px', color: '#757575' }}>
              <div style={{ marginBottom: '8px', wordBreak: 'break-all' }}>
                Exported to: {exportPath}
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={handleCopyPath}
                  style={{
                    padding: '6px 12px',
                    backgroundColor: '#333',
                    color: '#E0E0E0',
                    border: 'none',
                    borderRadius: '4px',
                    fontSize: '12px',
                    cursor: 'pointer',
                  }}
                >
                  Copy Path
                </button>
                <button
                  onClick={handleOpenFolder}
                  style={{
                    padding: '6px 12px',
                    backgroundColor: '#333',
                    color: '#E0E0E0',
                    border: 'none',
                    borderRadius: '4px',
                    fontSize: '12px',
                    cursor: 'pointer',
                  }}
                >
                  Open Folder
                </button>
              </div>
            </div>
          )}
        </div>

        {/* Warning */}
        <div style={{
          padding: '12px',
          backgroundColor: '#2A2A2A',
          borderRadius: '6px',
          fontSize: '12px',
          color: '#B0B0B0',
          marginBottom: '24px',
        }}>
          <strong>Note:</strong> All diagnostics files are automatically redacted of secrets (bearer tokens, private keys, etc.). However, please review the exported files before sharing.
        </div>

        {/* Actions */}
        <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
            style={{
              padding: '10px 20px',
              backgroundColor: '#333',
              color: '#E0E0E0',
              border: 'none',
              borderRadius: '6px',
              fontSize: '14px',
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

