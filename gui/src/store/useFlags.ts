import { create } from 'zustand'
import { loadFlags, watchFlags, type Flags } from '@/flags'
import { useEffect } from 'react'

interface FlagsStore {
  flags: Flags
  set: (flags: Flags) => void
}

export const useFlags = create<FlagsStore>((set) => {
  const initialFlags = loadFlags()

  // Watch for file changes (only in main process or Node.js)
  if (typeof window === 'undefined' || !window.electronAPI) {
    const unwatch = watchFlags((newFlags) => {
      set({ flags: newFlags })
    })

    // Cleanup on store destruction (if possible)
    return {
      flags: initialFlags,
      set: (newFlags: Flags) => set({ flags: newFlags }),
      unwatch, // Expose for cleanup if needed
    }
  }

  return {
    flags: initialFlags,
    set: (newFlags: Flags) => set({ flags: newFlags }),
  }
})

// React hook for components
export function useFlagsHook() {
  const flags = useFlags((state) => state.flags)
  const setFlags = useFlags((state) => state.set)

  // In renderer, poll for changes (since we can't watch files directly)
  useEffect(() => {
    if (typeof window !== 'undefined' && window.electronAPI) {
      const interval = setInterval(() => {
        const newFlags = loadFlags()
        if (JSON.stringify(newFlags) !== JSON.stringify(flags)) {
          setFlags(newFlags)
        }
      }, 2000) // Poll every 2s

      return () => clearInterval(interval)
    }
  }, [flags, setFlags])

  return flags
}

