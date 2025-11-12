import React from 'react'
import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { useAppStore } from '@/store/useAppStore'

export const MainLayout: React.FC = () => {
  const theme = useAppStore(state => state.settings.theme)
  const isDark = theme === 'dark' || (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches)
  const bgColor = isDark ? '#121212' : '#FFFFFF'

  return (
    <div style={{
      display: 'flex',
      height: '100vh',
      backgroundColor: bgColor,
      fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    }}>
      <Sidebar />
      <main style={{
        flex: 1,
        overflow: 'auto',
        padding: '32px',
        paddingBottom: '232px',
      }}>
        <Outlet />
      </main>
    </div>
  )
}

