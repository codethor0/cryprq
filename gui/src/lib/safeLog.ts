// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

/**
 * GUI debug console privacy guard
 * Ensures logs shown in the Debug Console never display secrets or raw key material
 */

export function safeLog(msg: string): string {
  if (!msg || typeof msg !== 'string') {
    return String(msg || '')
  }

  let safe = msg

  // Redact common secret patterns
  safe = safe.replace(/(priv(key|ate)?|secret|token|bearer|api[_-]?key|apikey)[^ \n\t:]*/gi, '$1[REDACTED]')
  
  // Redact cryptographic keys and seeds
  safe = safe.replace(/(sk|pk|seed|nonce|key|password|passwd|pwd)[=:\s]+([A-Za-z0-9+/=_-]{8,})/gi, '$1=[REDACTED]')
  
  // Redact multiaddr with embedded keys
  safe = safe.replace(/\/ip4\/[0-9.]+(\/[^/]+)*\/[A-Za-z0-9+/=_-]{32,}/g, '/ip4/[REDACTED]/...')
  
  // Redact hex strings that look like keys (32+ chars)
  safe = safe.replace(/\b([0-9a-fA-F]{32,})\b/g, (match) => {
    // Keep short hex strings (likely not keys)
    if (match.length < 32) return match
    return '[REDACTED]'
  })
  
  // Redact base64-looking strings (long ones)
  safe = safe.replace(/\b([A-Za-z0-9+/=_-]{32,})\b/g, (match) => {
    // Keep if it's clearly not a key (e.g., file paths, URLs)
    if (match.includes('/') || match.includes(':') || match.includes('.')) return match
    return '[REDACTED]'
  })
  
  // Redact email addresses (optional, but safer)
  safe = safe.replace(/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g, '[REDACTED]')

  return safe
}

/**
 * Redact an object recursively
 */
export function safeLogObject(obj: unknown): unknown {
  if (typeof obj === 'string') {
    return safeLog(obj)
  }
  
  if (Array.isArray(obj)) {
    return obj.map(safeLogObject)
  }
  
  if (obj && typeof obj === 'object') {
    const safe: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(obj)) {
      const safeKey = safeLog(key)
      safe[safeKey] = safeLogObject(value)
    }
    return safe
  }
  
  return obj
}

