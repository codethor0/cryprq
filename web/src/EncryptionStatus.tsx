type Status = {
  connected: boolean;
  encryption: 'ML-KEM (Kyber768) + X25519 hybrid' | 'None';
  peerId?: string;
  connectedPeer?: string;
  keyEpoch?: number;
  rotationInterval?: number;
  mode?: 'listener' | 'dialer';
};

export function EncryptionStatus({ status }: { status: Status }) {
  return (
    <div style={{
      background: '#1a1a1a',
      border: '1px solid #333',
      borderRadius: '8px',
      padding: '16px',
      marginTop: '16px',
      fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
      fontSize: '12px'
    }}>
      <h3 style={{ marginTop: 0, marginBottom: '12px', color: '#fff' }}>
        Encryption Status
      </h3>
      
      <div style={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: '8px 16px', color: '#ddd' }}>
        <div style={{ color: '#888' }}>Connection:</div>
        <div style={{ color: status.connected ? '#4f4' : '#f55' }}>
          {status.connected ? '✓ Encrypted Tunnel Active' : '✗ Disconnected'}
        </div>

        <div style={{ color: '#888' }}>Encryption:</div>
        <div style={{ color: '#59f' }}>
          {status.encryption}
        </div>

        {status.peerId && (
          <>
            <div style={{ color: '#888' }}>Local Peer ID:</div>
            <div style={{ color: '#8f8', wordBreak: 'break-all' }}>{status.peerId}</div>
          </>
        )}

        {status.connectedPeer && (
          <>
            <div style={{ color: '#888' }}>Connected Peer:</div>
            <div style={{ color: '#4f4', wordBreak: 'break-all' }}>{status.connectedPeer}</div>
          </>
        )}

        {status.keyEpoch && (
          <>
            <div style={{ color: '#888' }}>Key Epoch:</div>
            <div style={{ color: '#f90' }}>Epoch {status.keyEpoch}</div>
          </>
        )}

        {status.rotationInterval && (
          <>
            <div style={{ color: '#888' }}>Key Rotation:</div>
            <div style={{ color: '#f90' }}>Every {status.rotationInterval}s</div>
          </>
        )}

        {status.mode && (
          <>
            <div style={{ color: '#888' }}>Mode:</div>
            <div style={{ color: '#59f', textTransform: 'capitalize' }}>{status.mode}</div>
          </>
        )}
      </div>

      <div style={{ 
        marginTop: '12px', 
        padding: '8px', 
        background: '#0a0a0a', 
        borderRadius: '4px',
        fontSize: '11px',
        color: '#888',
        lineHeight: '1.5'
      }}>
        <strong style={{ color: '#fff' }}>Note:</strong> CrypRQ establishes encrypted peer-to-peer connections using post-quantum cryptography. 
        This creates an encrypted tunnel between peers, but does not route system/browser traffic yet. 
        The data-plane (packet forwarding) is experimental.
      </div>
    </div>
  );
}

