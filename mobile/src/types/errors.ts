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
  | 'METRICS_FETCH_FAILED'
  | 'SESSION_ENDED'
  | 'UNKNOWN';

export interface ErrorDescriptor {
  title: string;
  description: string;
  help?: string;
}

export const ERROR_CATALOG: Record<CrypRQErrorCode, ErrorDescriptor> = {
  PORT_IN_USE: {
    title: 'Port in Use',
    description: 'The selected UDP port is already in use by another application.',
    help: 'Please choose a different port in Settings.',
  },
  CLI_NOT_FOUND: {
    title: 'CrypRQ Binary Not Found',
    description: 'The CrypRQ executable could not be located.',
    help: 'Ensure CrypRQ is built and its path is correctly configured.',
  },
  CLI_EXITED: {
    title: 'CrypRQ Session Ended Unexpectedly',
    description: 'The CrypRQ process terminated prematurely.',
    help: 'Check logs for details or try restarting the session.',
  },
  BAD_MULTIADDR: {
    title: 'Invalid Multiaddr Format',
    description: 'The provided multiaddr is not in a valid format.',
    help: 'Example: /ip4/127.0.0.1/udp/9999/quic-v1/p2p/Qm...',
  },
  INVALID_PORT: {
    title: 'Invalid Port Number',
    description: 'Port number must be between 1 and 65535.',
    help: 'Ports below 1024 may require administrator privileges.',
  },
  INVALID_ROTATION_MINUTES: {
    title: 'Invalid Rotation Interval',
    description: 'Key rotation interval must be at least 1 minute.',
    help: 'A shorter interval may impact performance.',
  },
  NET_UNREACHABLE: {
    title: 'Network Unreachable',
    description: 'Could not reach the specified network address.',
    help: 'Check your internet connection or peer address.',
  },
  PERMISSION_DENIED: {
    title: 'Permission Denied',
    description: 'Operation requires elevated privileges.',
    help: 'Try running CrypRQ as an administrator or with appropriate permissions.',
  },
  METRICS_TIMEOUT: {
    title: 'Metrics Unavailable',
    description: 'Could not fetch metrics from the CrypRQ process.',
    help: 'The CrypRQ process might not be running or the metrics port is blocked.',
  },
  METRICS_FETCH_FAILED: {
    title: 'Metrics Fetch Failed',
    description: 'Failed to fetch metrics from the endpoint.',
    help: 'Check your network connection and endpoint configuration.',
  },
  SESSION_ENDED: {
    title: 'Session Ended',
    description: 'The CrypRQ session has ended.',
    help: 'This may be expected, or indicate an issue if unexpected.',
  },
  UNKNOWN: {
    title: 'Unknown Error',
    description: 'An unexpected error occurred.',
    help: 'Please check the application logs for more details.',
  },
};

export function getErrorDescriptor(code: CrypRQErrorCode): ErrorDescriptor {
  return ERROR_CATALOG[code] || ERROR_CATALOG.UNKNOWN;
}

