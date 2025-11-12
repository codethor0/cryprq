import {MMKV} from 'react-native-mmkv';

// Encrypted storage for sensitive data
const encryptedStorage = new MMKV({
  id: 'cryprq-encrypted',
  encryptionKey: 'default-key-change-in-production', // TODO: Generate per-device key
});

// Non-sensitive preferences
const preferencesStorage = new MMKV({
  id: 'cryprq-preferences',
});

export class SecureStorage {
  // Sensitive data (tokens, endpoints with credentials)
  static setSecure(key: string, value: string): void {
    encryptedStorage.set(key, value);
  }

  static getSecure(key: string): string | undefined {
    return encryptedStorage.getString(key);
  }

  static deleteSecure(key: string): void {
    encryptedStorage.delete(key);
  }

  // Non-sensitive preferences
  static setPreference(key: string, value: string | number | boolean): void {
    if (typeof value === 'string') {
      preferencesStorage.set(key, value);
    } else if (typeof value === 'number') {
      preferencesStorage.set(key, value);
    } else {
      preferencesStorage.set(key, value);
    }
  }

  static getPreference(key: string, type: 'string' | 'number' | 'boolean'): string | number | boolean | undefined {
    if (type === 'string') {
      return preferencesStorage.getString(key);
    } else if (type === 'number') {
      return preferencesStorage.getNumber(key);
    } else {
      return preferencesStorage.getBoolean(key);
    }
  }
}

// Network security
export function validateRemoteEndpoint(url: string): {valid: boolean; error?: string} {
  try {
    // Use a simple URL parser for React Native compatibility
    const parsed = url.match(/^(https?):\/\/([^\/]+)/);
    if (!parsed) {
      return {
        valid: false,
        error: 'Invalid URL format',
      };
    }
    
    const protocol = parsed[1];
    
    // REMOTE profile must use HTTPS
    if (protocol !== 'https') {
      return {
        valid: false,
        error: 'REMOTE profile requires HTTPS endpoint',
      };
    }
    
    return {valid: true};
  } catch {
    return {
      valid: false,
      error: 'Invalid URL format',
    };
  }
}

// Timeout configuration
export const NETWORK_TIMEOUTS = {
  connect: 3000, // 3s
  read: 3000, // 3s
  retry: {
    maxAttempts: 3,
    backoffMs: 1000, // Start with 1s, exponential backoff
  },
};

// Log redaction (mirror desktop)
export function redactSecrets(text: string): string {
  return text
    .replace(/\b(bearer\s+)[A-Za-z0-9._-]+/gi, '$1***REDACTED***')
    .replace(/\b(token|privKey)=([A-Za-z0-9+/=_-]+)/gi, (_m, k) => `${k}=***REDACTED***`)
    .replace(/\bprivKey\S*/gi, 'privKey***REDACTED***')
    .replace(/authorization\s*:\s*\S+/gi, 'authorization: ***REDACTED***');
}

// App integrity (stubs)
export class AppIntegrity {
  static async checkAndroid(): Promise<{compromised: boolean; reason?: string}> {
    // Stub: Would use Play Integrity API
    return {compromised: false};
  }

  static async checkIOS(): Promise<{compromised: boolean; reason?: string}> {
    // Stub: Basic jailbreak heuristics
    // Check for common jailbreak files
    const jailbreakPaths = [
      '/Applications/Cydia.app',
      '/Library/MobileSubstrate/MobileSubstrate.dylib',
      '/bin/bash',
      '/usr/sbin/sshd',
    ];
    
    // In real implementation, would check file system
    return {compromised: false};
  }
}

