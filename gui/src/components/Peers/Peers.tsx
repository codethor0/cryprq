import React, { useState } from 'react'
import { useAppStore } from '@/store/useAppStore'
import { Peer } from '@/types'
import { AddPeerModal } from './AddPeerModal'
import { backend } from '@/services/backend'

export const Peers: React.FC = () => {
  const { peers, addPeer, removePeer, connect } = useAppStore()
  const [showAddDialog, setShowAddDialog] = useState(false)

  const handleAdd = (peer: Peer) => {
    addPeer(peer)
  }

  const handleRemove = (peerId: string) => {
    if (window.confirm(`Are you sure you want to remove peer ${peerId}?`)) {
      removePeer(peerId)
    }
  }

  return (
    <div style={{ maxWidth: '1200px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '32px', fontWeight: 600 }}>Peers</h1>
        <button
          onClick={() => setShowAddDialog(true)}
          style={{
            padding: '12px 24px',
            backgroundColor: '#1DE9B6',
            color: '#000',
            border: 'none',
            borderRadius: '8px',
            fontSize: '14px',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          + Add Peer
        </button>
      </div>

      {showAddDialog && (
        <AddPeerModal
          onAdd={handleAdd}
          onClose={() => setShowAddDialog(false)}
          onTestReachability={async (multiaddr) => {
            if (typeof window !== 'undefined' && window.electronAPI) {
              try {
                return await window.electronAPI.peerTestReachability(multiaddr)
              } catch (error) {
                return { ok: false, error: 'NET_UNREACHABLE' }
              }
            }
            return { ok: false, error: 'NOT_AVAILABLE' }
          }}
        />
      )}

      {peers.length === 0 ? (
        <div style={{
          backgroundColor: '#1E1E1E',
          borderRadius: '12px',
          padding: '48px',
          textAlign: 'center',
          color: '#B0B0B0',
        }}>
          <div style={{ fontSize: '48px', marginBottom: '16px' }}>🔗</div>
          <div style={{ fontSize: '18px', marginBottom: '8px' }}>No peers configured</div>
          <div style={{ fontSize: '14px' }}>Add a peer to get started</div>
        </div>
      ) : (
        <div style={{ display: 'grid', gap: '16px' }}>
          {peers.map(peer => (
            <div
              key={peer.id}
              style={{
                backgroundColor: '#1E1E1E',
                borderRadius: '12px',
                padding: '24px',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
              }}
            >
              <div style={{ flex: 1 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '8px' }}>
                  <div style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    backgroundColor: peer.status === 'connected' ? '#4CAF50' : '#757575',
                  }} />
                  <span style={{ fontSize: '16px', fontWeight: 600, fontFamily: 'monospace' }}>
                    {peer.id.substring(0, 20)}...
                  </span>
                </div>
                <div style={{ fontSize: '13px', color: '#B0B0B0', fontFamily: 'monospace' }}>
                  {peer.multiaddr}
                </div>
                {peer.lastSeen && (
                  <div style={{ fontSize: '12px', color: '#757575', marginTop: '4px' }}>
                    Last seen: {peer.lastSeen.toLocaleString()}
                  </div>
                )}
              </div>
              <div style={{ display: 'flex', gap: '8px' }}>
                <button
                  onClick={() => connect(peer.multiaddr)}
                  disabled={peer.status === 'connected'}
                  style={{
                    padding: '8px 16px',
                    backgroundColor: peer.status === 'connected' ? '#333' : '#1DE9B6',
                    color: peer.status === 'connected' ? '#757575' : '#000',
                    border: 'none',
                    borderRadius: '6px',
                    fontSize: '13px',
                    fontWeight: 600,
                    cursor: peer.status === 'connected' ? 'not-allowed' : 'pointer',
                  }}
                >
                  Connect
                </button>
                <button
                  onClick={() => handleRemove(peer.id)}
                  style={{
                    padding: '8px 16px',
                    backgroundColor: 'transparent',
                    border: '1px solid #F44336',
                    borderRadius: '6px',
                    color: '#F44336',
                    fontSize: '13px',
                    cursor: 'pointer',
                  }}
                >
                  Remove
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

