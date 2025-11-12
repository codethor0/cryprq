import React, { useState } from 'react'
import { useAppStore } from '@/store/useAppStore'
import { isValidPort, isValidRotationMinutes, VALIDATION_HELP } from '@/utils/validation'
import { Tooltip } from '@/components/ui/Tooltip'
import { toastStore } from '@/store/toastStore'
import { hostnameFromUrl, isValidHostname } from '@/utils/host'
import { AllowlistModal } from './AllowlistModal'

export const Settings: React.FC = () => {
  const { settings, updateSettings } = useAppStore()
  const [errors, setErrors] = useState<{ port?: string; rotation?: string; endpoint?: string }>({})
  const [touched, setTouched] = useState<{ port?: boolean; rotation?: boolean; endpoint?: boolean }>({})
  const [showAllowlistModal, setShowAllowlistModal] = useState(false)
  const [showPostQuantumInfo, setShowPostQuantumInfo] = useState(false)
  const [localEndpoint, setLocalEndpoint] = useState('') // For REMOTE endpoint validation

  const validate = () => {
    const e: typeof errors = {}
    
    if (!isValidPort(settings.transport.udpPort)) {
      e.port = 'Port must be between 1 and 65535.'
    }
    
    const rotationMinutes = Math.floor(settings.rotationInterval / 60)
    if (!isValidRotationMinutes(rotationMinutes)) {
      e.rotation = 'Rotation interval must be at least 1 minute.'
    }
    
    // REMOTE endpoint allowlist validation (when REMOTE profile is implemented)
    // This is a placeholder for future REMOTE profile support
    // if (profile === 'REMOTE' && localEndpoint) {
    //   const host = hostnameFromUrl(localEndpoint).toLowerCase()
    //   const allowlist = settings.remoteEndpointAllowlist || []
    //   if (allowlist.length > 0 && !allowlist.includes(host)) {
    //     e.endpoint = `Host "${host}" isn't in the allowlist. Add it via "Manage Allowlist" first.`
    //   }
    // }
    
    setErrors(e)
    return Object.keys(e).length === 0
  }

  const handleSave = async () => {
    if (!validate()) {
      toastStore.getState().addToast({
        type: 'error',
        title: 'Validation Error',
        message: 'Please fix the errors before saving.',
        duration: 4000,
      })
      return
    }

    try {
      await updateSettings(settings)
      toastStore.getState().addToast({
        type: 'success',
        title: 'Settings Saved',
        message: 'Your settings have been updated successfully.',
        duration: 3000,
      })
    } catch (error: any) {
      toastStore.getState().addToast({
        type: 'error',
        title: 'Save Failed',
        message: error?.message || 'Failed to save settings.',
        duration: 6000,
      })
    }
  }

  const handlePortChange = (value: string) => {
    const port = parseInt(value, 10) || 0
    updateSettings({ transport: { ...settings.transport, udpPort: port } })
    
    if (touched.port) {
      if (!isValidPort(port)) {
        setErrors(prev => ({ ...prev, port: 'Port must be between 1 and 65535.' }))
      } else {
        setErrors(prev => {
          const { port: _, ...rest } = prev
          return rest
        })
      }
    }
  }

  const handleRotationChange = (value: string) => {
    const seconds = parseInt(value, 10) || 0
    updateSettings({ rotationInterval: seconds })
    
    const minutes = Math.floor(seconds / 60)
    if (touched.rotation) {
      if (!isValidRotationMinutes(minutes)) {
        setErrors(prev => ({ ...prev, rotation: 'Rotation interval must be at least 1 minute.' }))
      } else {
        setErrors(prev => {
          const { rotation: _, ...rest } = prev
          return rest
        })
      }
    }
  }

  const hasErrors = Object.keys(errors).length > 0

  return (
    <div style={{ maxWidth: '800px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h1 style={{ margin: 0, fontSize: '32px', fontWeight: 600 }}>Settings</h1>
        <button
          onClick={handleSave}
          disabled={hasErrors}
          style={{
            padding: '12px 24px',
            backgroundColor: hasErrors ? '#666' : '#1DE9B6',
            color: hasErrors ? '#999' : '#000',
            border: 'none',
            borderRadius: '8px',
            fontSize: '14px',
            fontWeight: 600,
            cursor: hasErrors ? 'not-allowed' : 'pointer',
          }}
        >
          Save Settings
        </button>
      </div>

      {/* Rotation Interval */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Key Rotation</h3>
        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Rotation Interval (seconds)
            <Tooltip content={VALIDATION_HELP.rotationMinutes}>
              <span style={{ fontSize: '12px', color: '#757575', cursor: 'help' }}>ℹ️</span>
            </Tooltip>
          </label>
          <input
            type="number"
            value={settings.rotationInterval}
            onChange={(e) => handleRotationChange(e.target.value)}
            onBlur={() => {
              setTouched(prev => ({ ...prev, rotation: true }))
              validate()
            }}
            min="60"
            max="3600"
            data-field="rotation"
            style={{
              width: '200px',
              padding: '10px',
              backgroundColor: '#121212',
              border: errors.rotation ? '1px solid #F44336' : '1px solid #333',
              borderRadius: '6px',
              color: '#E0E0E0',
              fontSize: '14px',
            }}
          />
          {errors.rotation && (
            <div style={{ fontSize: '12px', color: '#F44336', marginTop: '4px' }}>
              {errors.rotation}
            </div>
          )}
          <div style={{ fontSize: '12px', color: '#757575', marginTop: '4px' }}>
            Current: {Math.floor(settings.rotationInterval / 60)} minutes
          </div>
        </div>
      </div>

      {/* Logging */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Logging</h3>
        <div>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Log Level
          </label>
          <select
            value={settings.logLevel}
            onChange={(e) => updateSettings({ logLevel: e.target.value as any })}
            style={{
              width: '200px',
              padding: '10px',
              backgroundColor: '#121212',
              border: '1px solid #333',
              borderRadius: '6px',
              color: '#E0E0E0',
              fontSize: '14px',
            }}
          >
            <option value="error">Error</option>
            <option value="warn">Warning</option>
            <option value="info">Info</option>
            <option value="debug">Debug</option>
          </select>
        </div>
      </div>

      {/* Transport */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Transport</h3>
        <div style={{ marginBottom: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Multiaddr
            <Tooltip content={VALIDATION_HELP.multiaddr}>
              <span style={{ fontSize: '12px', color: '#757575', cursor: 'help' }}>ℹ️</span>
            </Tooltip>
          </label>
          <input
            type="text"
            value={settings.transport.multiaddr}
            onChange={(e) => updateSettings({ transport: { ...settings.transport, multiaddr: e.target.value } })}
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
        <div>
          <label style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            UDP Port
            <Tooltip content={VALIDATION_HELP.port}>
              <span style={{ fontSize: '12px', color: '#757575', cursor: 'help' }}>ℹ️</span>
            </Tooltip>
          </label>
          <input
            type="number"
            value={settings.transport.udpPort}
            onChange={(e) => handlePortChange(e.target.value)}
            onBlur={() => {
              setTouched(prev => ({ ...prev, port: true }))
              validate()
            }}
            min="1"
            max="65535"
            data-field="port"
            style={{
              width: '200px',
              padding: '10px',
              backgroundColor: '#121212',
              border: errors.port ? '1px solid #F44336' : '1px solid #333',
              borderRadius: '6px',
              color: '#E0E0E0',
              fontSize: '14px',
            }}
          />
          {errors.port && (
            <div style={{ fontSize: '12px', color: '#F44336', marginTop: '4px' }}>
              {errors.port}
            </div>
          )}
        </div>
      </div>

      {/* Theme */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Appearance</h3>
        <div>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Theme
          </label>
          <select
            value={settings.theme}
            onChange={(e) => updateSettings({ theme: e.target.value as any })}
            style={{
              width: '200px',
              padding: '10px',
              backgroundColor: '#121212',
              border: '1px solid #333',
              borderRadius: '6px',
              color: '#E0E0E0',
              fontSize: '14px',
            }}
          >
            <option value="light">Light</option>
            <option value="dark">Dark</option>
            <option value="system">System</option>
          </select>
        </div>
      </div>

      {/* Window Behavior */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Window Behavior</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '12px', fontSize: '14px', color: '#B0B0B0', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.minimizeToTray !== false}
              onChange={(e) => updateSettings({ minimizeToTray: e.target.checked })}
              style={{ width: '18px', height: '18px', cursor: 'pointer' }}
            />
            <span>Minimize to tray</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: '12px', fontSize: '14px', color: '#B0B0B0', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.startMinimized === true}
              onChange={(e) => updateSettings({ startMinimized: e.target.checked })}
              style={{ width: '18px', height: '18px', cursor: 'pointer' }}
            />
            <span>Start minimized</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: '12px', fontSize: '14px', color: '#B0B0B0', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.keepRunningInBackground !== false}
              onChange={(e) => updateSettings({ keepRunningInBackground: e.target.checked })}
              style={{ width: '18px', height: '18px', cursor: 'pointer' }}
            />
            <span>Keep running in background when window is closed</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: '12px', fontSize: '14px', color: '#B0B0B0', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.disconnectOnQuit !== false}
              onChange={(e) => updateSettings({ disconnectOnQuit: e.target.checked })}
              style={{ width: '18px', height: '18px', cursor: 'pointer' }}
            />
            <span>Disconnect on app quit (kill-switch)</span>
          </label>
        </div>
      </div>

      {/* Security */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Security</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          {/* Post-Quantum Encryption Toggle */}
          <div>
            <label style={{ display: 'flex', alignItems: 'flex-start', gap: '12px', fontSize: '14px', color: '#B0B0B0', cursor: 'pointer' }}>
              <input
                type="checkbox"
                checked={settings.postQuantumEnabled !== false}
                onChange={(e) => {
                  const enabled = e.target.checked
                  updateSettings({ postQuantumEnabled: enabled })
                  if (!enabled) {
                    toastStore.getState().addToast({
                      type: 'warning',
                      title: 'Post-Quantum Encryption Disabled',
                      message: 'You are using X25519-only encryption. This is not recommended for future-proof security.',
                      duration: 8000,
                    })
                  }
                }}
                style={{ width: '18px', height: '18px', cursor: 'pointer', marginTop: '2px', flexShrink: 0 }}
              />
              <div style={{ flex: 1 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
                  <span style={{ fontWeight: 500 }}>Post-Quantum Encryption</span>
                  <Tooltip content="Enable ML-KEM (Kyber768) + X25519 hybrid handshake for future-proof security. Disabling falls back to X25519-only (not recommended).">
                    <span style={{ fontSize: '12px', color: '#757575', cursor: 'help' }}>ℹ️</span>
                  </Tooltip>
                  <button
                    onClick={() => setShowPostQuantumInfo(true)}
                    style={{
                      fontSize: '12px',
                      color: '#1DE9B6',
                      background: 'none',
                      border: 'none',
                      cursor: 'pointer',
                      textDecoration: 'underline',
                      padding: 0,
                      marginLeft: '4px',
                    }}
                  >
                    Learn more
                  </button>
                </div>
                <p style={{ fontSize: '12px', color: '#757575', marginTop: '4px', marginLeft: '0' }}>
                  {settings.postQuantumEnabled !== false
                    ? '✅ ML-KEM (Kyber768) + X25519 hybrid encryption enabled. Protects against future quantum computer attacks.'
                    : '⚠️ X25519-only encryption. Not recommended for long-term security.'}
                </p>
              </div>
            </label>
          </div>

          <div style={{ height: '1px', backgroundColor: '#333', margin: '8px 0' }} />

          <div>
            <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
              Remote Endpoint Allowlist
            </label>
            <p style={{ fontSize: '12px', color: '#757575', marginBottom: '12px' }}>
              Manage allowed hostnames for REMOTE profile endpoints. Empty list = no restrictions.
            </p>
            <button
              onClick={() => setShowAllowlistModal(true)}
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
              Manage Allowlist
            </button>
            {settings.remoteEndpointAllowlist && settings.remoteEndpointAllowlist.length > 0 && (
              <div style={{ marginTop: '12px', fontSize: '12px', color: '#B0B0B0' }}>
                {settings.remoteEndpointAllowlist.length} hostname(s) configured
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Privacy */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Privacy</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '12px', fontSize: '14px', color: '#B0B0B0', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={settings.telemetryEnabled === true}
              onChange={async (e) => {
                const enabled = e.target.checked
                await updateSettings({ telemetryEnabled: enabled })
                // Notify main process
                if (window.electronAPI?.telemetrySet) {
                  await window.electronAPI.telemetrySet(enabled)
                }
              }}
              style={{ width: '18px', height: '18px', cursor: 'pointer' }}
            />
            <div>
              <span>Enable telemetry (opt-in)</span>
              <p style={{ fontSize: '12px', color: '#757575', marginTop: '4px', marginLeft: '0' }}>
                Event counters only (connect/disconnect/rotation). No PII. Stored locally.
              </p>
            </div>
          </label>
        </div>
      </div>

      {/* Charts */}
      <div style={{
        backgroundColor: '#1E1E1E',
        borderRadius: '12px',
        padding: '24px',
        marginBottom: '24px',
      }}>
        <h3 style={{ margin: '0 0 16px', fontSize: '18px', fontWeight: 600 }}>Charts</h3>
        <div>
          <label style={{ display: 'block', marginBottom: '8px', fontSize: '14px', color: '#B0B0B0' }}>
            Smoothing (EMA Alpha)
          </label>
          <input
            type="range"
            min="0"
            max="0.4"
            step="0.05"
            value={settings.chartSmoothing || 0.2}
            onChange={(e) => updateSettings({ chartSmoothing: parseFloat(e.target.value) })}
            style={{ width: '100%', cursor: 'pointer' }}
          />
          <div style={{ fontSize: '12px', color: '#757575', marginTop: '4px' }}>
            Current: {(settings.chartSmoothing || 0.2).toFixed(2)} (0 = no smoothing, 0.4 = max smoothing)
          </div>
        </div>
      </div>

      {showAllowlistModal && (
        <AllowlistModal
          allowlist={settings.remoteEndpointAllowlist || []}
          onSave={(allowlist) => {
            updateSettings({ remoteEndpointAllowlist: allowlist })
            setShowAllowlistModal(false)
          }}
          onClose={() => setShowAllowlistModal(false)}
        />
      )}

      <PostQuantumInfo
        isOpen={showPostQuantumInfo}
        onClose={() => setShowPostQuantumInfo(false)}
      />
    </div>
  )
}
