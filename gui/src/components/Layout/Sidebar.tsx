import React from 'react'
import { useNavigate, useLocation } from 'react-router-dom'
import { useAppStore } from '@/store/useAppStore'

const menuItems = [
  { path: '/', label: 'Dashboard', icon: '📊' },
  { path: '/peers', label: 'Peers', icon: '🔗' },
  { path: '/settings', label: 'Settings', icon: '⚙️' },
  { path: '/logs', label: 'Logs', icon: '📋' },
]

export const Sidebar: React.FC = () => {
  const navigate = useNavigate()
  const location = useLocation()
  const theme = useAppStore(state => state.settings.theme)
  const isDark = theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  const colors = isDark ? {
    bg: '#1A1A1A',
    hover: '#252525',
    active: '#1DE9B6',
    text: '#E0E0E0',
  } : {
    bg: '#FAFAFA',
    hover: '#F0F0F0',
    active: '#1DE9B6',
    text: '#212121',
  }

  return (
    <div style={{
      width: '240px',
      height: '100vh',
      backgroundColor: colors.bg,
      borderRight: `1px solid ${isDark ? '#333' : '#E0E0E0'}`,
      padding: '24px 0',
      display: 'flex',
      flexDirection: 'column',
    }}>
      <div style={{ padding: '0 24px 32px', borderBottom: `1px solid ${isDark ? '#333' : '#E0E0E0'}` }}>
        <h1 style={{ margin: 0, fontSize: '24px', fontWeight: 600, color: colors.text }}>
          CrypRQ
        </h1>
        <p style={{ margin: '4px 0 0', fontSize: '12px', color: isDark ? '#B0B0B0' : '#757575' }}>
          Post-quantum VPN
        </p>
      </div>
      
      <nav style={{ flex: 1, padding: '16px 0', marginTop: '16px' }}>
        {menuItems.map(item => {
          const isActive = location.pathname === item.path
          return (
            <button
              key={item.path}
              onClick={() => navigate(item.path)}
              style={{
                width: '100%',
                padding: '12px 24px',
                border: 'none',
                background: isActive ? (isDark ? '#252525' : '#F0F0F0') : 'transparent',
                color: isActive ? colors.active : colors.text,
                textAlign: 'left',
                cursor: 'pointer',
                fontSize: '14px',
                fontWeight: isActive ? 600 : 400,
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                transition: 'all 0.2s',
              }}
              onMouseEnter={(e) => {
                if (!isActive) {
                  e.currentTarget.style.backgroundColor = colors.hover
                }
              }}
              onMouseLeave={(e) => {
                if (!isActive) {
                  e.currentTarget.style.backgroundColor = 'transparent'
                }
              }}
            >
              <span style={{ fontSize: '18px' }}>{item.icon}</span>
              {item.label}
            </button>
          )
        })}
      </nav>
    </div>
  )
}

