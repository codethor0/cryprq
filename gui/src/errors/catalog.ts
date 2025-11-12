export type CrypRQErrorCode =
  | 'PORT_IN_USE'
  | 'CLI_NOT_FOUND'
  | 'CLI_EXITED'
  | 'BAD_MULTIADDR'
  | 'INVALID_PORT'
  | 'INVALID_ROTATION_MINUTES'
  | 'NET_UNREACHABLE'
  | 'PERMISSION_DENIED'
  | 'METRICS_TIMEOUT'
  | 'UNKNOWN'

export interface ErrorDescriptor {
  title: string
  description: string
  help?: string
  docSlug?: string
}

export const ERROR_CATALOG: Record<CrypRQErrorCode, ErrorDescriptor> = {
  PORT_IN_USE: {
    title: 'Port Already in Use',
    description: 'The selected port is already being used by another application.',
    help: 'Try selecting a different port in Settings, or close the application using this port.',
    docSlug: 'troubleshooting#ports',
  },
  CLI_NOT_FOUND: {
    title: 'CrypRQ Binary Not Found',
    description: 'The CrypRQ CLI binary could not be located.',
    help: 'Please ensure CrypRQ is built and available, or locate the binary manually.',
    docSlug: 'installation#binary-location',
  },
  CLI_EXITED: {
    title: 'Session Ended Unexpectedly',
    description: 'The CrypRQ process exited unexpectedly.',
    help: 'Check the logs for details about what went wrong.',
    docSlug: 'troubleshooting#session-errors',
  },
  BAD_MULTIADDR: {
    title: 'Invalid Multiaddr',
    description: 'The provided multiaddr format is invalid.',
    help: 'Multiaddr should follow the format: /ip4/127.0.0.1/udp/9999/quic-v1/p2p/...',
    docSlug: 'configuration#multiaddr',
  },
  INVALID_PORT: {
    title: 'Invalid Port Number',
    description: 'Port must be between 1 and 65535.',
    help: 'Please enter a valid port number.',
    docSlug: 'configuration#ports',
  },
  INVALID_ROTATION_MINUTES: {
    title: 'Invalid Rotation Interval',
    description: 'Rotation interval must be at least 1 minute.',
    help: 'For security, keys should rotate at least every minute.',
    docSlug: 'configuration#rotation',
  },
  NET_UNREACHABLE: {
    title: 'Network Unreachable',
    description: 'Unable to reach the specified peer or network.',
    help: 'Check your network connection and firewall settings.',
    docSlug: 'troubleshooting#network',
  },
  PERMISSION_DENIED: {
    title: 'Permission Denied',
    description: 'Insufficient permissions to perform this operation.',
    help: 'You may need administrator/root privileges for this action.',
    docSlug: 'troubleshooting#permissions',
  },
  METRICS_TIMEOUT: {
    title: 'Metrics Temporarily Unavailable',
    description: 'Unable to fetch metrics from the CrypRQ process.',
    help: 'This is usually temporary. Metrics will resume once the connection stabilizes.',
    docSlug: 'troubleshooting#metrics',
  },
  UNKNOWN: {
    title: 'Unknown Error',
    description: 'An unexpected error occurred.',
    help: 'Please check the logs for more details.',
    docSlug: 'troubleshooting',
  },
}

export function mapErrorToCode(error: string | Error, lastLogs?: string[]): CrypRQErrorCode {
  const errorStr = typeof error === 'string' ? error : error.message
  const lowerError = errorStr.toLowerCase()
  const logsStr = lastLogs?.join('\n').toLowerCase() || ''

  if (lowerError.includes('address already in use') || lowerError.includes('port') && lowerError.includes('busy')) {
    return 'PORT_IN_USE'
  }
  if (lowerError.includes('binary not found') || lowerError.includes('not found') && lowerError.includes('cryprq')) {
    return 'CLI_NOT_FOUND'
  }
  if (lowerError.includes('exited') || lowerError.includes('process exited')) {
    return 'CLI_EXITED'
  }
  if (lowerError.includes('multiaddr') || lowerError.includes('invalid address')) {
    return 'BAD_MULTIADDR'
  }
  if (lowerError.includes('network unreachable') || lowerError.includes('unreachable')) {
    return 'NET_UNREACHABLE'
  }
  if (lowerError.includes('permission denied') || lowerError.includes('eacces') || lowerError.includes('eperm')) {
    return 'PERMISSION_DENIED'
  }
  if (logsStr.includes('eaddrinuse') || logsStr.includes('address already in use')) {
    return 'PORT_IN_USE'
  }

  return 'UNKNOWN'
}

export function getErrorDescriptor(code: CrypRQErrorCode): ErrorDescriptor {
  return ERROR_CATALOG[code] || ERROR_CATALOG.UNKNOWN
}

