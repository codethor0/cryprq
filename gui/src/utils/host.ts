/**
 * Hostname validation and extraction utilities
 */

export const isValidHostname = (h: string): boolean => {
  if (!h || typeof h !== 'string') return false
  const trimmed = h.trim().toLowerCase()
  if (!trimmed) return false
  
  // Basic hostname regex: alphanumeric, dots, hyphens
  if (!/^[a-z0-9.-]+$/i.test(trimmed)) return false
  
  // Cannot start or end with dot or hyphen
  if (trimmed.startsWith('.') || trimmed.endsWith('.') || 
      trimmed.startsWith('-') || trimmed.endsWith('-')) {
    return false
  }
  
  // Cannot have consecutive dots
  if (trimmed.includes('..')) return false
  
  // Must have at least one dot (for domain) or be localhost
  if (trimmed !== 'localhost' && !trimmed.includes('.')) return false
  
  return true
}

export const hostnameFromUrl = (u: string): string => {
  if (!u || typeof u !== 'string') return ''
  
  try {
    // Add protocol if missing for URL parsing
    const urlStr = u.includes('://') ? u : `https://${u}`
    const url = new URL(urlStr)
    return url.hostname.toLowerCase().trim()
  } catch {
    // If URL parsing fails, try to extract hostname manually
    const match = u.match(/^(?:https?:\/\/)?([^\/:]+)/i)
    return match ? match[1].toLowerCase().trim() : ''
  }
}

export const normalizeHostname = (h: string): string => {
  return h.trim().toLowerCase()
}

