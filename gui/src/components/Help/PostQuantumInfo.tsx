import React from 'react'

interface PostQuantumInfoProps {
  isOpen: boolean
  onClose: () => void
}

export const PostQuantumInfo: React.FC<PostQuantumInfoProps> = ({ isOpen, onClose }) => {
  if (!isOpen) return null

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
    }} onClick={onClose}>
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        maxWidth: '600px',
        width: '90%',
        maxHeight: '80vh',
        overflow: 'auto',
      }} onClick={(e) => e.stopPropagation()}>
        <h2 style={{ fontSize: '24px', fontWeight: 600, marginBottom: '16px' }}>
          What is Post-Quantum Encryption?
        </h2>
        
        <p style={{ fontSize: '16px', lineHeight: '1.6', marginBottom: '16px', color: '#E0E0E0' }}>
          Post-quantum encryption protects your data against future quantum computer attacks.
          CrypRQ uses a hybrid approach combining ML-KEM (Kyber768) with X25519 for maximum security.
        </p>

        <h3 style={{ fontSize: '18px', fontWeight: 600, marginTop: '24px', marginBottom: '12px' }}>
          ✅ Benefits
        </h3>
        <ul style={{ listStyle: 'none', padding: 0, marginBottom: '24px' }}>
          <li style={{ marginBottom: '8px', paddingLeft: '24px', position: 'relative' }}>
            <span style={{ position: 'absolute', left: 0 }}>🔒</span>
            <strong>Future-proof security:</strong> Protects against store-now-decrypt-later attacks
          </li>
          <li style={{ marginBottom: '8px', paddingLeft: '24px', position: 'relative' }}>
            <span style={{ position: 'absolute', left: 0 }}>🛡️</span>
            <strong>Defense-in-depth:</strong> Hybrid ML-KEM + X25519 provides multiple layers
          </li>
          <li style={{ marginBottom: '8px', paddingLeft: '24px', position: 'relative' }}>
            <span style={{ position: 'absolute', left: 0 }}>🔄</span>
            <strong>Automatic rotation:</strong> Keys rotate every 5 minutes by default
          </li>
          <li style={{ marginBottom: '8px', paddingLeft: '24px', position: 'relative' }}>
            <span style={{ position: 'absolute', left: 0 }}>⚡</span>
            <strong>Performance:</strong> Minimal overhead, optimized implementation
          </li>
        </ul>

        <h3 style={{ fontSize: '18px', fontWeight: 600, marginTop: '24px', marginBottom: '12px' }}>
          ⚠️ Important Notes
        </h3>
        <div style={{ backgroundColor: '#2A2A2A', padding: '16px', borderRadius: '8px', marginBottom: '24px' }}>
          <p style={{ fontSize: '14px', lineHeight: '1.6', color: '#B0B0B0', margin: 0 }}>
            <strong>Recommended:</strong> Keep post-quantum encryption enabled for maximum security.
            Disabling it falls back to X25519-only encryption, which is not recommended for long-term security.
          </p>
        </div>

        <h3 style={{ fontSize: '18px', fontWeight: 600, marginTop: '24px', marginBottom: '12px' }}>
          📚 Learn More
        </h3>
        <p style={{ fontSize: '14px', lineHeight: '1.6', color: '#B0B0B0' }}>
          For technical details, see the{' '}
          <a
            href="https://github.com/codethor0/cryprq/blob/main/docs/security.md"
            target="_blank"
            rel="noopener noreferrer"
            style={{ color: '#1DE9B6', textDecoration: 'underline' }}
          >
            Security Model documentation
          </a>
          {' '}or{' '}
          <a
            href="https://openquantumsafe.org/"
            target="_blank"
            rel="noopener noreferrer"
            style={{ color: '#1DE9B6', textDecoration: 'underline' }}
          >
            Open Quantum Safe project
          </a>
          .
        </p>

        <div style={{ marginTop: '32px', display: 'flex', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
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
            Got it
          </button>
        </div>
      </div>
    </div>
  )
}

