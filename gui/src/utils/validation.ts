import { CrypRQErrorCode } from '@/errors/catalog'

// Multiaddr regex: supports /ip4|ip6|dns|dns4|dns6/.../tcp|udp/.../quic|quic-v1?/p2p/...
const MA_RE = /^\/(ip4|ip6|dns|dns4|dns6)\/[^/]+\/(tcp|udp)\/\d+(\/(quic|quic-v1))?(\/p2p\/[1-9A-HJ-NP-Za-km-z]{46,59})?$/

export function isValidPort(n: number): boolean {
  return Number.isInteger(n) && n >= 1 && n <= 65535
}

export function isValidRotationMinutes(n: number): boolean {
  return Number.isInteger(n) && n >= 1
}

export function parseAndValidateMultiaddr(
  s: string
): { ok: true; multiaddr: string } | { ok: false; reason: CrypRQErrorCode } {
  if (!s || s.trim().length === 0) {
    return { ok: false, reason: 'BAD_MULTIADDR' }
  }

  const trimmed = s.trim()

  // Basic format check
  if (!MA_RE.test(trimmed)) {
    return { ok: false, reason: 'BAD_MULTIADDR' }
  }

  // Additional validation: ensure it's a valid multiaddr structure
  try {
    const parts = trimmed.split('/').filter(Boolean)
    
    // Must start with protocol (ip4, ip6, dns, etc.)
    if (!['ip4', 'ip6', 'dns', 'dns4', 'dns6'].includes(parts[0])) {
      return { ok: false, reason: 'BAD_MULTIADDR' }
    }

    // Must have transport (tcp, udp)
    const transportIndex = parts.findIndex(p => ['tcp', 'udp'].includes(p))
    if (transportIndex === -1) {
      return { ok: false, reason: 'BAD_MULTIADDR' }
    }

    // Port must be valid
    const portIndex = transportIndex + 1
    if (portIndex >= parts.length) {
      return { ok: false, reason: 'BAD_MULTIADDR' }
    }
    const port = parseInt(parts[portIndex], 10)
    if (!isValidPort(port)) {
      return { ok: false, reason: 'INVALID_PORT' }
    }

    return { ok: true, multiaddr: trimmed }
  } catch {
    return { ok: false, reason: 'BAD_MULTIADDR' }
  }
}

export const VALIDATION_HELP = {
  port: 'Port must be between 1 and 65535. Common VPN ports: 51820 (WireGuard), 1194 (OpenVPN), 9999 (custom).',
  rotationMinutes: 'Ephemeral keys reduce exposure window. Recommended: 5 minutes. Minimum: 1 minute.',
  multiaddr: 'Format: /ip4/127.0.0.1/udp/9999/quic-v1/p2p/12D3KooW...\n\nExamples:\n• /ip4/192.168.1.100/udp/9999/quic-v1\n• /dns/example.com/udp/9999/quic-v1/p2p/QmHash...',
}

