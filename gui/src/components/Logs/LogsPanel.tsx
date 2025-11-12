import React, { useMemo, useRef, useState, useEffect } from 'react'
import { FixedSizeList as List } from 'react-window'

export type LogLine = {
  ts: string
  level: 'debug' | 'info' | 'warn' | 'error'
  source: 'cli' | 'ipc' | 'metrics' | 'app'
  msg: string
  meta?: Record<string, unknown>
}

interface LogsPanelProps {
  lines: LogLine[]
  onClear?: () => void
  initialSearch?: string
  initialTimeRange?: { start: Date; end: Date }
}

export default function LogsPanel({
  lines,
  onClear,
  initialSearch = '',
  initialTimeRange,
}: LogsPanelProps) {
  const [query, setQuery] = useState(initialSearch)
  const [level, setLevel] = useState<'all' | 'info' | 'warn' | 'error'>('all')
  const [follow, setFollow] = useState(true)
  const listRef = useRef<List>(null)

  const filtered = useMemo(() => {
    const q = query.toLowerCase()
    let result = lines.filter(
      (l) =>
        (level === 'all' || l.level === level) &&
        (!q || l.msg.toLowerCase().includes(q) || l.source.toLowerCase().includes(q))
    )

    // Filter by time range if provided
    if (initialTimeRange) {
      result = result.filter((l) => {
        const logTime = new Date(l.ts)
        return logTime >= initialTimeRange.start && logTime <= initialTimeRange.end
      })
    }

    return result
  }, [lines, query, level, initialTimeRange])

  // Auto-scroll on new lines when follow ON
  useEffect(() => {
    if (!follow || !listRef.current) return
    listRef.current.scrollToItem(filtered.length - 1, 'end')
  }, [filtered.length, follow])

  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const l = filtered[index]
    const highlight = query && l.msg.toLowerCase().includes(query.toLowerCase())

    const handleCopy = () => {
      const text = `[${l.ts}] ${l.level.toUpperCase()} [${l.source}] ${l.msg}`
      navigator.clipboard.writeText(text)
    }

    return (
      <div
        style={{
          ...style,
          fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
          fontSize: '12px',
          padding: '4px 8px',
          backgroundColor: highlight ? '#2A2A2A' : 'transparent',
          color:
            l.level === 'error'
              ? '#EF5350'
              : l.level === 'warn'
              ? '#FFB74D'
              : l.level === 'info'
              ? '#B0B0B0'
              : '#757575',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
          cursor: 'pointer',
        }}
        onClick={handleCopy}
        title="Click to copy line"
      >
        <strong>[{new Date(l.ts).toLocaleTimeString()}] {l.level.toUpperCase()} [{l.source}]:</strong>{' '}
        <span>{l.msg}</span>
      </div>
    )
  }

  const handleCopyLine = (line: LogLine) => {
    const text = `[${line.ts}] ${line.level.toUpperCase()} [${line.source}] ${line.msg}`
    navigator.clipboard.writeText(text)
  }

  const handleExportDiagnostics = async () => {
    if (typeof window !== 'undefined' && window.electronAPI) {
      try {
        const result = await window.electronAPI.diagnosticsExport()
        if (result.ok) {
          alert(`Diagnostics exported to: ${result.path}`)
        }
      } catch (error) {
        console.error('Failed to export diagnostics:', error)
        alert('Failed to export diagnostics')
      }
    }
  }

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
      <div
        style={{
          display: 'flex',
          gap: '12px',
          padding: '16px',
          backgroundColor: '#1E1E1E',
          borderBottom: '1px solid #333',
          flexWrap: 'wrap',
          alignItems: 'center',
        }}
      >
        <select
          value={level}
          onChange={(e) => setLevel(e.target.value as any)}
          style={{
            padding: '8px 12px',
            backgroundColor: '#121212',
            border: '1px solid #333',
            borderRadius: '6px',
            color: '#E0E0E0',
            fontSize: '14px',
          }}
        >
          <option value="all">All</option>
          <option value="info">Info</option>
          <option value="warn">Warn</option>
          <option value="error">Error</option>
        </select>

        <input
          placeholder="Search…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          style={{
            flex: 1,
            minWidth: '200px',
            padding: '8px 12px',
            backgroundColor: '#121212',
            border: '1px solid #333',
            borderRadius: '6px',
            color: '#E0E0E0',
            fontSize: '14px',
          }}
        />

        <label
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            fontSize: '14px',
            color: '#B0B0B0',
            cursor: 'pointer',
          }}
        >
          <input
            type="checkbox"
            checked={follow}
            onChange={(e) => setFollow(e.target.checked)}
            style={{ width: '18px', height: '18px', cursor: 'pointer' }}
          />
          Follow
        </label>

        <button
          onClick={() => {
            setQuery('')
            if (onClear) onClear()
          }}
          style={{
            padding: '8px 16px',
            backgroundColor: 'transparent',
            border: '1px solid #666',
            borderRadius: '6px',
            color: '#E0E0E0',
            fontSize: '14px',
            cursor: 'pointer',
          }}
        >
          Clear view
        </button>

        <button
          onClick={handleExportDiagnostics}
          style={{
            padding: '8px 16px',
            backgroundColor: '#1DE9B6',
            border: 'none',
            borderRadius: '6px',
            color: '#000',
            fontSize: '14px',
            fontWeight: 600,
            cursor: 'pointer',
          }}
        >
          Export diagnostics…
        </button>
      </div>

      <div style={{ flex: 1, backgroundColor: '#121212' }}>
        {filtered.length === 0 ? (
          <div
            style={{
              padding: '48px',
              textAlign: 'center',
              color: '#757575',
              fontSize: '14px',
            }}
          >
            No logs to display
          </div>
        ) : (
          <List
            ref={listRef}
            height={600}
            width="100%"
            itemCount={filtered.length}
            itemSize={22}
          >
            {Row}
          </List>
        )}
      </div>
    </div>
  )
}

