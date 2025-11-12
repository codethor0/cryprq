import {redactSecrets} from '@/services/security';
import {useAppStore} from '@/store/appStore';

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export function log(level: LogLevel, message: string, meta?: Record<string, any>): void {
  const {settings, addLog} = useAppStore.getState();
  
  // Gate log levels behind Developer screen setting
  // Default to 'info' only
  const allowedLevels: LogLevel[] = settings.logLevel === 'debug' 
    ? ['debug', 'info', 'warn', 'error']
    : ['info', 'warn', 'error'];
  
  if (!allowedLevels.includes(level)) {
    return; // Skip if level not allowed
  }
  
  // Redact secrets
  const redactedMessage = redactSecrets(message);
  const redactedMeta = meta ? (() => {
    const result: Record<string, any> = {};
    for (const [k, v] of Object.entries(meta)) {
      result[k] = typeof v === 'string' ? redactSecrets(v) : v;
    }
    return result;
  })() : undefined;
  
  addLog({
    ts: new Date().toISOString(),
    level,
    source: 'app',
    msg: redactedMessage,
    meta: redactedMeta,
  });
}

export const logger = {
  debug: (msg: string, meta?: Record<string, any>) => log('debug', msg, meta),
  info: (msg: string, meta?: Record<string, any>) => log('info', msg, meta),
  warn: (msg: string, meta?: Record<string, any>) => log('warn', msg, meta),
  error: (msg: string, meta?: Record<string, any>) => log('error', msg, meta),
};

