import React, { useState } from 'react'
import { Peer } from '@/types'
import { parseAndValidateMultiaddr, VALIDATION_HELP } from '@/utils/validation'
import { Tooltip } from '@/components/ui/Tooltip'

interface AddPeerModalProps {
  onAdd: (peer: Peer) => void
  onClose: () => void
  onTestReachability?: (multiaddr: string) => Promise<{ ok: boolean; latencyMs?: number; error?: string }>
}

export const AddPeerModal: React.FC<AddPeerModalProps> = ({ onAdd, onClose, onTestReachability }) => {
  const [peerId, setPeerId] = useState('')
  const [multiaddr, setMultiaddr] = useState('')
  const [multiaddrError, setMultiaddrError] = useState<string | null>(null)
  const [testing, setTesting] = useState(false)
  const [reachabilityResult, setReachabilityResult] = useState<{ ok: boolean; latencyMs?: number } | null>(null)

  const validation = parseAndValidateMultiaddr(multiaddr)
  const isValid = validation.ok && peerId.trim().length > 0

  const handleMultiaddrChange = (value: string) => {
    setMultiaddr(value)
    setReachabilityResult(null)
    
    if (value.trim().length === 0) {
      setMultiaddrError(null)
      return
    }

    const result = parseAndValidateMultiaddr(value)
    if (!result.ok) {
      setMultiaddrError('Invalid multiaddr format. ' + VALIDATION_HELP.multiaddr.split('\n')[0])
    } else {
      setMultiaddrError(null)
    }
  }

  const handleTestReachability = async () => {
    if (!onTestReachability || !validation.ok) return

    setTesting(true)
    setReachabilityResult(null)
    
    try {
      const result = await onTestReachability(validation.multiaddr)
      setReachabilityResult(result)
    } catch (error: any) {
      setReachabilityResult({ ok: false })
    } finally {
      setTesting(false)
    }
  }

  const handleAdd = () => {
    if (!isValid) return

    onAdd({
      id: peerId.trim(),
      multiaddr: validation.ok ? validation.multiaddr : multiaddr,
      status: 'disconnected',
    })
    
    setPeerId('')
    setMultiaddr('')
    setMultiaddrError(null)
    setReachabilityResult(null)
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
        backgroundColor: 'rgba(0, 0, 0, 0.5)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: '#1E1E1E',
          borderRadius: '12px',
          padding: '32px',
          minWidth: '400px',
          maxWidth: '600px',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h3 style={{ margin: '0 0 24px', fontSize: '20px', fontWeight: 600 }}>Add Peer</h3>
        
        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Peer ID
          </label>
          <input
            type="text"
            value={peerId}
            onChange={(e) => setPeerId(e.target.value)}
            placeholder="12D3KooW..."
            style={{
              width: '100%',
              padding: '10px',
              backgroundColor: '#121212',
              border: '1px solid #333',
              borderRadius: '6px',
              color: '#E0E0E0',
              fontSize: '14px',
              fontFamily: 'monospace',
            }}
          />
        </div>

        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Multiaddr
            <Tooltip content={VALIDATION_HELP.multiaddr}>
              <span style={{ fontSize: '12px', color: '#757575', cursor: 'help' }}>ℹ️</span>
            </Tooltip>
          </label>
          <input
            type="text"
            value={multiaddr}
            onChange={(e) => handleMultiaddrChange(e.target.value)}
            placeholder="/ip4/127.0.0.1/udp/9999/quic-v1"
            style={{
              width: '100%',
              padding: '10px',
              backgroundColor: '#121212',
              border: multiaddrError ? '1px solid #F44336' : '1px solid #333',
              borderRadius: '6px',
              color: '#E0E0E0',
              fontSize: '14px',
              fontFamily: 'monospace',
            }}
          />
          {multiaddrError && (
            <div style={{ fontSize: '12px', color: '#F44336', marginTop: '4px' }}>
              {multiaddrError}
            </div>
          )}
          {validation.ok && !multiaddrError && (
            <div style={{ fontSize: '12px', color: '#4CAF50', marginTop: '4px' }}>
              ✓ Valid multiaddr format
            </div>
          )}
        </div>

        {onTestReachability && validation.ok && (
          <div style={{ marginBottom: '16px' }}>
            <button
              onClick={handleTestReachability}
              disabled={testing || !validation.ok}
              style={{
                padding: '8px 16px',
                backgroundColor: testing ? '#666' : '#1DE9B6',
                color: testing ? '#999' : '#000',
                border: 'none',
                borderRadius: '6px',
                fontSize: '13px',
                fontWeight: 600,
                cursor: testing || !validation.ok ? 'not-allowed' : 'pointer',
              }}
            >
              {testing ? 'Testing...' : 'Test Reachability'}
            </button>
            {reachabilityResult && (
              <div style={{ marginTop: '8px', fontSize: '13px' }}>
                {reachabilityResult.ok ? (
                  <span style={{ color: '#4CAF50' }}>
                    ✓ Reachable in {reachabilityResult.latencyMs}ms
                  </span>
                ) : (
                  <span style={{ color: '#F44336' }}>
                    ✗ Unreachable
                  </span>
                )}
              </div>
            )}
          </div>
        )}

        <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
            style={{
              padding: '10px 20px',
              backgroundColor: 'transparent',
              border: '1px solid #333',
              borderRadius: '6px',
              color: '#E0E0E0',
              cursor: 'pointer',
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleAdd}
            disabled={!isValid}
            style={{
              padding: '10px 20px',
              backgroundColor: isValid ? '#1DE9B6' : '#666',
              border: 'none',
              borderRadius: '6px',
              color: isValid ? '#000' : '#999',
              fontWeight: 600,
              cursor: isValid ? 'pointer' : 'not-allowed',
            }}
          >
            Add
          </button>
        </div>
      </div>
    </div>
  )
}

