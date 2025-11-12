export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'rotating' | 'errored';

export type ProfileType = 'LOCAL' | 'LAN' | 'REMOTE';

export interface ConnectionState {
  status: ConnectionStatus;
  peerId?: string;
  latency?: number;
  rotationTimer?: number;
  bytesIn?: number;
  bytesOut?: number;
  lastConnected?: string;
}

export interface Peer {
  id: string;
  alias?: string;
  multiaddr: string;
  status: 'idle' | 'connecting' | 'connected' | 'disconnected' | 'error';
  latency?: number;
  lastHandshake?: string;
}

export interface AppSettings {
  profile: ProfileType;
  endpoint?: string;
  rotationInterval: number;
  logLevel: 'debug' | 'info' | 'warn' | 'error';
  theme: 'light' | 'dark' | 'system';
  notifications: {
    connectDisconnect: boolean;
    rotations: boolean;
  };
  telemetryEnabled?: boolean;
  postQuantumEnabled?: boolean; // Post-quantum encryption (ML-KEM + X25519 hybrid, default: true)
  mode?: 'controller' | 'on-device';
  crashReportingEnabled?: boolean;
  eulaAccepted?: boolean;
  privacyAccepted?: boolean;
}

export interface LogLine {
  ts: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  source: 'cli' | 'app';
  msg: string;
  meta?: Record<string, any>;
}

export interface Metrics {
  bytesIn: number;
  bytesOut: number;
  latency: number;
  rotationTimer: number;
  peerId?: string;
}

export interface ThroughputPoint {
  timestamp: number;
  bytesIn: number;
  bytesOut: number;
}

