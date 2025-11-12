import type {CrypRQErrorCode} from '@/types/errors';

const MA_RE = /^\/(ip4|ip6|dns|dns4|dns6)\/[^/]+\/(tcp|udp)\/\d+(\/(quic|quic-v1))?(\/p2p\/[1-9A-HJ-NP-Za-km-z]{46,59})?$/;

export function isValidPort(n: number): boolean {
  return n >= 1 && n <= 65535;
}

export function isValidRotationMinutes(n: number): boolean {
  return n >= 1;
}

export function parseAndValidateMultiaddr(s: string): {ok: true} | {ok: false; reason: CrypRQErrorCode}> {
  if (!s || s.trim() === '') {
    return {ok: false, reason: 'BAD_MULTIADDR'};
  }
  if (!MA_RE.test(s)) {
    return {ok: false, reason: 'BAD_MULTIADDR'};
  }
  return {ok: true};
}

export function isValidEndpoint(url: string): boolean {
  try {
    const parsed = new URL(url);
    return ['http:', 'https:'].includes(parsed.protocol);
  } catch {
    return false;
  }
}

