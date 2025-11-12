import React, { useState } from 'react'
import { isValidHostname, normalizeHostname } from '@/utils/host'
import { toastStore } from '@/store/toastStore'

interface AllowlistModalProps {
  allowlist: string[]
  onSave: (allowlist: string[]) => void
  onClose: () => void
}

export const AllowlistModal: React.FC<AllowlistModalProps> = ({ allowlist, onSave, onClose }) => {
  const [hostnames, setHostnames] = useState<string[]>(allowlist)
  const [newHostname, setNewHostname] = useState('')
  const [error, setError] = useState<string>('')

  const handleAdd = () => {
    const normalized = normalizeHostname(newHostname)
    
    if (!normalized) {
      setError('Hostname cannot be empty')
      return
    }
    
    if (!isValidHostname(normalized)) {
      setError('Invalid hostname format')
      return
    }
    
    if (hostnames.includes(normalized)) {
      setError('Hostname already in allowlist')
      return
    }
    
    setHostnames([...hostnames, normalized])
    setNewHostname('')
    setError('')
  }

  const handleRemove = (hostname: string) => {
    setHostnames(hostnames.filter(h => h !== hostname))
  }

  const handleSave = () => {
    // Dedupe and sort
    const deduped = Array.from(new Set(hostnames.map(normalizeHostname))).sort()
    onSave(deduped)
    toastStore.getState().addToast({
      type: 'success',
      title: 'Allowlist Updated',
      message: `${deduped.length} hostname(s) saved`,
      duration: 3000,
    })
    onClose()
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
        maxWidth: '500px',
        width: '90%',
        maxHeight: '80vh',
        overflow: 'auto',
      }}>
        <h2 style={{ margin: '0 0 16px', fontSize: '20px', fontWeight: 600 }}>Manage Endpoint Allowlist</h2>
        <p style={{ fontSize: '14px', color: '#B0B0B0', marginBottom: '16px' }}>
          Add hostnames allowed for REMOTE profile endpoints. Empty list = no restrictions.
        </p>

        {/* Add hostname */}
        <div style={{ marginBottom: '16px' }}>
          <div style={{ display: 'flex', gap: '8px' }}>
            <input
              type="text"
              value={newHostname}
              onChange={(e) => {
                setNewHostname(e.target.value)
                setError('')
              }}
              onKeyPress={(e) => {
                if (e.key === 'Enter') {
                  handleAdd()
                }
              }}
              placeholder="example.com"
              style={{
                flex: 1,
                padding: '10px',
                backgroundColor: '#121212',
                border: error ? '1px solid #F44336' : '1px solid #333',
                borderRadius: '6px',
                color: '#E0E0E0',
                fontSize: '14px',
              }}
            />
            <button
              onClick={handleAdd}
              style={{
                padding: '10px 20px',
                backgroundColor: '#1DE9B6',
                color: '#000',
                border: 'none',
                borderRadius: '6px',
                fontSize: '14px',
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Add
            </button>
          </div>
          {error && (
            <div style={{ fontSize: '12px', color: '#F44336', marginTop: '4px' }}>
              {error}
            </div>
          )}
        </div>

        {/* Hostname list */}
        <div style={{ marginBottom: '16px', maxHeight: '300px', overflowY: 'auto' }}>
          {hostnames.length === 0 ? (
            <div style={{ fontSize: '14px', color: '#757575', textAlign: 'center', padding: '20px' }}>
              No hostnames in allowlist (no restrictions)
            </div>
          ) : (
            hostnames.map((hostname, idx) => (
              <div
                key={idx}
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  padding: '8px 12px',
                  backgroundColor: '#121212',
                  borderRadius: '6px',
                  marginBottom: '8px',
                }}
              >
                <span style={{ fontSize: '14px', fontFamily: 'monospace', color: '#E0E0E0' }}>
                  {hostname}
                </span>
                <button
                  onClick={() => handleRemove(hostname)}
                  style={{
                    padding: '4px 12px',
                    backgroundColor: '#F44336',
                    color: '#FFF',
                    border: 'none',
                    borderRadius: '4px',
                    fontSize: '12px',
                    cursor: 'pointer',
                  }}
                >
                  Remove
                </button>
              </div>
            ))
          )}
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
            Cancel
          </button>
          <button
            onClick={handleSave}
            style={{
              padding: '10px 20px',
              backgroundColor: '#1DE9B6',
              color: '#000',
              border: 'none',
              borderRadius: '6px',
              fontSize: '14px',
              fontWeight: 600,
              cursor: 'pointer',
            }}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  )
}

