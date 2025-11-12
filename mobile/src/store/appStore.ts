import {create} from 'zustand';
import {persist, createJSONStorage} from 'zustand/middleware';
import {MMKV} from 'react-native-mmkv';
import {SecureStorage} from '@/services/security';
import type {
  ConnectionStatus,
  ConnectionState,
  Peer,
  AppSettings,
  LogLine,
  ThroughputPoint,
  ProfileType,
} from '@/types';

const storage = new MMKV({id: 'cryprq-storage'});

const mmkvStorage = {
  setItem: (name: string, value: string) => {
    return storage.set(name, value);
  },
  getItem: (name: string) => {
    const value = storage.getString(name);
    return value ?? null;
  },
  removeItem: (name: string) => {
    return storage.delete(name);
  },
};

interface AppState {
  // Connection state
  connectionStatus: ConnectionState;
  
  // Peers
  peers: Peer[];
  
  // Settings
  settings: AppSettings;
  
  // Logs
  logs: LogLine[];
  
  // Throughput history (60s rolling)
  throughputHistory: ThroughputPoint[];
  
  // Actions
  setConnectionStatus: (status: ConnectionStatus, data?: Partial<ConnectionState>) => void;
  updateMetrics: (metrics: Partial<ConnectionState>) => void;
  addPeer: (peer: Peer) => void;
  removePeer: (peerId: string) => void;
  updatePeer: (peerId: string, updates: Partial<Peer>) => void;
  setSettings: (settings: Partial<AppSettings>) => void;
  addLog: (log: LogLine) => void;
  clearLogs: () => void;
  addThroughputPoint: (point: ThroughputPoint) => void;
  connect: (peerId?: string) => Promise<void>;
  disconnect: () => Promise<void>;
  restart: (peerId?: string) => Promise<void>;
  setProfile: (profile: ProfileType) => void;
  setEndpoint: (endpoint: string) => void;
}

const defaultSettings: AppSettings = {
  profile: 'LOCAL',
  endpoint: 'http://127.0.0.1:9464',
  rotationInterval: 5,
  logLevel: 'info',
  theme: 'system',
  notifications: {
    connectDisconnect: true,
    rotations: true,
  },
};

export const useAppStore = create<AppState>()(
  persist(
    (set, get) => ({
      connectionStatus: {
        status: 'disconnected',
      },
      peers: [],
      settings: defaultSettings,
      logs: [],
      throughputHistory: [],
      
      // Keep last 200 logs
      maxLogs: 200,

      setConnectionStatus: (status, data) =>
        set(state => ({
          connectionStatus: {
            ...state.connectionStatus,
            status,
            ...data,
          },
        })),

      updateMetrics: metrics =>
        set(state => ({
          connectionStatus: {
            ...state.connectionStatus,
            ...metrics,
          },
        })),

      addPeer: peer =>
        set(state => ({
          peers: [...state.peers.filter(p => p.id !== peer.id), peer],
        })),

      removePeer: peerId =>
        set(state => ({
          peers: state.peers.filter(p => p.id !== peerId),
        })),

      updatePeer: (peerId, updates) =>
        set(state => ({
          peers: state.peers.map(p =>
            p.id === peerId ? {...p, ...updates} : p,
          ),
        })),

      setSettings: settings =>
        set(state => ({
          settings: {...state.settings, ...settings},
        })),

      addLog: log => {
        const maxLogs = get().maxLogs || 200;
        const newLogs = [...get().logs, log].slice(-maxLogs);
        set({logs: newLogs});
      },

      clearLogs: () => set({logs: []}),

      addThroughputPoint: point => {
        const now = Date.now();
        const history = [
          ...get().throughputHistory.filter(p => p.timestamp > now - 60000), // 60s window
          point,
        ];
        set({throughputHistory: history});
      },

      connect: async (peerId) => {
        set(state => ({
          connectionStatus: {...state.connectionStatus, status: 'connecting'},
        }));
        // Implementation will be in services/backend.ts
      },

      disconnect: async () => {
        set(state => ({
          connectionStatus: {...state.connectionStatus, status: 'disconnected'},
        }));
      },

      restart: async (peerId) => {
        await get().disconnect();
        await get().connect(peerId);
      },

      setProfile: (profile) => {
        const endpoint =
          profile === 'LOCAL'
            ? 'http://127.0.0.1:9464'
            : profile === 'LAN'
            ? get().settings.endpoint || 'http://192.168.1.100:9464'
            : get().settings.endpoint || 'https://gateway.example.com';
        get().setSettings({profile, endpoint});
        
        // Store sensitive endpoints in encrypted storage
        if (profile === 'REMOTE' && endpoint) {
          SecureStorage.setSecure('remote_endpoint', endpoint);
        }
      },

      setEndpoint: (endpoint) => {
        get().setSettings({endpoint});
        
        // Store sensitive endpoints in encrypted storage
        const {settings} = get();
        if (settings.profile === 'REMOTE' && endpoint) {
          SecureStorage.setSecure('remote_endpoint', endpoint);
        }
      },
    }),
    {
      name: 'cryprq-app-store',
      storage: createJSONStorage(() => mmkvStorage),
      partialize: state => ({
        settings: state.settings,
        peers: state.peers,
      }),
    },
  ),
);

