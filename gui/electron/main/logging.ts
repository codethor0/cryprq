import * as fs from 'fs'
import * as path from 'path'
import * as os from 'os'

const LOG_DIR = path.join(os.homedir(), '.cryprq', 'logs')
const SCHEMA_VERSION = 1

export interface StructuredLogEntry {
  v: number
  ts: string
  lvl: 'info' | 'warn' | 'error' | 'debug'
  src: 'cli' | 'ipc' | 'metrics' | 'app'
  event: string
  msg: string
  data?: Record<string, unknown>
}

// Legacy interface for backward compatibility
export interface LogLine {
  ts: string
  level: 'debug' | 'info' | 'warn' | 'error'
  source: 'cli' | 'ipc' | 'metrics' | 'app'
  msg: string
  meta?: Record<string, unknown>
}

function redactString(s: string): string {
  return s
    .replace(/\b(bearer\s+)[A-Za-z0-9._-]+/gi, '$1***REDACTED***')
    .replace(/\b(token|privKey)=([A-Za-z0-9+/=_-]+)/gi, (_m, k) => `${k}=***REDACTED***`)
    .replace(/\bprivKey\S*/gi, 'privKey***REDACTED***')
    .replace(/authorization\s*:\s*\S+/gi, 'authorization: ***REDACTED***')
}

function redactDeep(obj: any): any {
  if (typeof obj === 'string') {
    return redactString(obj)
  }
  if (Array.isArray(obj)) {
    return obj.map(redactDeep)
  }
  if (obj && typeof obj === 'object') {
    const redacted: Record<string, any> = {}
    for (const [key, value] of Object.entries(obj)) {
      redacted[key] = redactDeep(value)
    }
    return redacted
  }
  return obj
}

export function redactSecrets(s: string): string {
  return redactString(s)
}

function validateEntry(entry: Partial<StructuredLogEntry>): { valid: boolean; reason?: string } {
  if (typeof entry.v !== 'number') {
    return { valid: false, reason: 'Missing or invalid v' }
  }
  if (!entry.ts || typeof entry.ts !== 'string') {
    return { valid: false, reason: 'Missing or invalid ts' }
  }
  // Validate ISO 8601
  if (isNaN(Date.parse(entry.ts))) {
    return { valid: false, reason: 'Invalid ts format' }
  }
  if (!['info', 'warn', 'error', 'debug'].includes(entry.lvl || '')) {
    return { valid: false, reason: 'Missing or invalid lvl' }
  }
  if (!['cli', 'ipc', 'metrics', 'app'].includes(entry.src || '')) {
    return { valid: false, reason: 'Missing or invalid src' }
  }
  if (!entry.event || typeof entry.event !== 'string') {
    return { valid: false, reason: 'Missing or invalid event' }
  }
  if (!entry.msg || typeof entry.msg !== 'string') {
    return { valid: false, reason: 'Missing or invalid msg' }
  }
  if (entry.msg.length > 200) {
    return { valid: false, reason: 'msg exceeds 200 chars' }
  }
  return { valid: true }
}

function ensureDir(p: string) {
  fs.mkdirSync(p, { recursive: true })
}

function todayFile(): string {
  const d = new Date().toISOString().slice(0, 10)
  return path.join(LOG_DIR, `cryprq-${d}.log`)
}

function formatLogLine(line: LogLine): string {
  const metaStr = line.meta ? ' ' + JSON.stringify(line.meta) : ''
  return `[${line.ts}] ${line.level.toUpperCase()} [${line.source}] ${line.msg}${metaStr}\n`
}

function formatStructuredEntry(entry: StructuredLogEntry): string {
  return JSON.stringify(entry) + '\n'
}

export function appendLog(line: LogLine): void {
  // Convert legacy format to structured
  const entry: StructuredLogEntry = {
    v: SCHEMA_VERSION,
    ts: line.ts,
    lvl: line.level,
    src: line.source,
    event: 'cli.raw', // Default event for legacy logs
    msg: line.msg,
    data: line.meta ? redactDeep(line.meta) : undefined,
  }
  appendStructuredLog(entry)
}

export function appendStructuredLog(entry: Partial<StructuredLogEntry>): void {
  ensureDir(LOG_DIR)
  
  // Ensure required fields
  const fullEntry: StructuredLogEntry = {
    v: SCHEMA_VERSION,
    ts: entry.ts || new Date().toISOString(),
    lvl: entry.lvl || 'info',
    src: entry.src || 'app',
    event: entry.event || 'unknown',
    msg: redactString(entry.msg || ''),
    data: entry.data ? redactDeep(entry.data) : undefined,
  }
  
  // Validate
  const validation = validateEntry(fullEntry)
  if (!validation.valid) {
    // Wrap invalid entry
    const invalidEntry: StructuredLogEntry = {
      v: SCHEMA_VERSION,
      ts: new Date().toISOString(),
      lvl: 'error',
      src: 'app',
      event: 'log.invalid',
      msg: 'Invalid log entry',
      data: { reason: validation.reason, original: redactDeep(entry) },
    }
    const formatted = formatStructuredEntry(invalidEntry)
    const file = todayFile()
    fs.appendFileSync(file, formatted, { encoding: 'utf8' })
    rotateIfNeeded(file)
    return
  }
  
  // Redact msg
  fullEntry.msg = redactString(fullEntry.msg)
  
  const formatted = formatStructuredEntry(fullEntry)
  const file = todayFile()
  fs.appendFileSync(file, formatted, { encoding: 'utf8' })
  rotateIfNeeded(file)
}

function rotateIfNeeded(file: string): void {
  try {
    const st = fs.statSync(file)
    if (st.size < 10 * 1024 * 1024) return // 10MB

    const timestamp = Date.now()
    const rotated = file.replace('.log', `-${timestamp}.log`)
    fs.renameSync(file, rotated)

    // Keep last 7 files
    const files = fs
      .readdirSync(LOG_DIR)
      .filter((f) => f.startsWith('cryprq-') && f.endsWith('.log'))
      .sort()
      .reverse()

    for (const f of files.slice(7)) {
      fs.rmSync(path.join(LOG_DIR, f), { force: true })
    }
  } catch (error) {
    // Ignore rotation errors
  }
}

export async function readTail({ lines = 1000 }: { lines?: number }): Promise<string[]> {
  ensureDir(LOG_DIR)
  const file = todayFile()

  if (!fs.existsSync(file)) {
    return []
  }

  try {
    const data = fs.readFileSync(file, 'utf8').split('\n')
    return data.slice(Math.max(0, data.length - lines)).filter(Boolean)
  } catch (error) {
    return []
  }
}

export async function readStructuredLogs({
  since,
  until,
  event,
  limit = 1000,
}: {
  since?: Date
  until?: Date
  event?: string
  limit?: number
}): Promise<StructuredLogEntry[]> {
  ensureDir(LOG_DIR)
  const files = getLogFiles()
  const entries: StructuredLogEntry[] = []

  for (const file of files.reverse()) {
    if (entries.length >= limit) break

    try {
      const content = fs.readFileSync(file, 'utf8')
      const lines = content.split('\n').filter(Boolean)

      for (const line of lines.reverse()) {
        if (entries.length >= limit) break

        try {
          const entry = JSON.parse(line) as StructuredLogEntry
          
          // Filter by time range
          if (since && new Date(entry.ts) < since) continue
          if (until && new Date(entry.ts) > until) continue
          
          // Filter by event
          if (event && entry.event !== event) continue
          
          entries.push(entry)
        } catch {
          // Skip invalid JSON lines
        }
      }
    } catch (error) {
      // Skip files that can't be read
    }
  }

  return entries.reverse().slice(0, limit)
}

export function getLogFiles(): string[] {
  ensureDir(LOG_DIR)
  try {
    return fs
      .readdirSync(LOG_DIR)
      .filter((f) => f.startsWith('cryprq-') && f.endsWith('.log'))
      .map((f) => path.join(LOG_DIR, f))
      .sort()
  } catch (error) {
    return []
  }
}

