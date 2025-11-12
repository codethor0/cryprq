import { create } from 'zustand'

export interface Toast {
  id: string
  type: 'info' | 'success' | 'warning' | 'error'
  title?: string
  message: string
  duration?: number
}

interface ToastStore {
  toasts: Toast[]
  addToast: (toast: Omit<Toast, 'id'>) => void
  removeToast: (id: string) => void
  lastErrorAt: number
  rateLimitEnabled: boolean
  setRateLimitEnabled: (enabled: boolean) => void
}

let lastErrorAt = 0

export const toastStore = create<ToastStore>((set, get) => ({
  toasts: [],
  lastErrorAt: 0,
  rateLimitEnabled: true,
  addToast: (toast) => {
    // Rate-limit error toasts (max 1 per 10s)
    if (toast.type === 'error' && get().rateLimitEnabled) {
      const now = Date.now()
      if (now - lastErrorAt < 10_000) {
        return // Drop error toast if within 10s window
      }
      lastErrorAt = now
      set({ lastErrorAt: now })
    }
    
    const id = `toast-${Date.now()}-${Math.random()}`
    set((state) => ({
      toasts: [...state.toasts, { ...toast, id }],
    }))
  },
  removeToast: (id: string) => {
    set((state) => ({
      toasts: state.toasts.filter((t) => t.id !== id),
    }))
  },
  setRateLimitEnabled: (enabled: boolean) => {
    set({ rateLimitEnabled: enabled })
  },
}))
