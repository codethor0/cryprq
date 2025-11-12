import React, { useEffect, useState } from 'react'
import { toastStore, Toast } from '@/store/toastStore'

export const Toaster: React.FC = () => {
  const toasts = toastStore((state) => state.toasts)
  const removeToast = toastStore((state) => state.removeToast)

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '24px',
        right: '24px',
        zIndex: 9999,
        display: 'flex',
        flexDirection: 'column',
        gap: '12px',
        maxWidth: '400px',
      }}
    >
      {toasts.slice(0, 3).map((toast) => (
        <ToastItem key={toast.id} toast={toast} onRemove={removeToast} />
      ))}
    </div>
  )
}

interface ToastItemProps {
  toast: Toast
  onRemove: (id: string) => void
}

const ToastItem: React.FC<ToastItemProps> = ({ toast, onRemove }) => {
  const [isVisible, setIsVisible] = useState(false)

  useEffect(() => {
    setIsVisible(true)
    const timer = setTimeout(() => {
      setIsVisible(false)
      setTimeout(() => onRemove(toast.id), 300)
    }, toast.duration || 6000)

    return () => clearTimeout(timer)
  }, [toast.id, toast.duration, onRemove])

  const bgColor = {
    info: '#1E1E1E',
    success: '#1E1E1E',
    warning: '#1E1E1E',
    error: '#1E1E1E',
  }[toast.type] || '#1E1E1E'

  const borderColor = {
    info: '#1DE9B6',
    success: '#4CAF50',
    warning: '#FF9800',
    error: '#F44336',
  }[toast.type] || '#1DE9B6'

  return (
    <div
      style={{
        backgroundColor: bgColor,
        border: `1px solid ${borderColor}`,
        borderRadius: '8px',
        padding: '16px',
        minWidth: '300px',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
        opacity: isVisible ? 1 : 0,
        transform: isVisible ? 'translateX(0)' : 'translateX(100%)',
        transition: 'all 0.3s ease',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
        <div style={{ flex: 1 }}>
          {toast.title && (
            <div style={{ fontSize: '14px', fontWeight: 600, color: '#E0E0E0', marginBottom: '4px' }}>
              {toast.title}
            </div>
          )}
          <div style={{ fontSize: '13px', color: '#B0B0B0' }}>
            {toast.message}
          </div>
        </div>
        <button
          onClick={() => onRemove(toast.id)}
          style={{
            background: 'none',
            border: 'none',
            color: '#757575',
            cursor: 'pointer',
            fontSize: '18px',
            padding: '0',
            marginLeft: '12px',
            lineHeight: '1',
          }}
        >
          ×
        </button>
      </div>
    </div>
  )
}

