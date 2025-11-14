// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { useEffect, useRef } from 'react';

type E = { t:string; level:'status'|'rotation'|'peer'|'info'|'error' };

export function DebugConsole({events}:{events:E[]}) {
  const consoleRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    if (consoleRef.current) {
      consoleRef.current.scrollTop = consoleRef.current.scrollHeight;
    }
  }, [events]);

  return (
    <div
      ref={consoleRef}
      style={{
        position: 'fixed',
        bottom: 0,
        left: 0,
        right: 0,
        height: 280,
        overflow: 'auto',
        background: '#0a0a0a',
        color: '#ddd',
        fontFamily: 'ui-monospace, SFMono-Regular, Menlo, monospace',
        fontSize: 11,
        borderTop: '2px solid #333',
        padding: '12px 16px',
        lineHeight: 1.5,
        boxShadow: '0 -4px 12px rgba(0,0,0,0.5)'
      }}
    >
      <div style={{
        position: 'sticky',
        top: 0,
        background: '#0a0a0a',
        paddingBottom: 8,
        marginBottom: 8,
        borderBottom: '1px solid #222',
        fontSize: 10,
        fontWeight: 600,
        color: '#666',
        textTransform: 'uppercase',
        letterSpacing: 1
      }}>
        Debug Console ({events.length} events)
      </div>
      {events.slice(-500).map((e,i)=>(
        <div
          key={i}
          style={{
            color: e.level==='error'?'#f55':
                   e.level==='rotation'?'#f90':
                   e.level==='peer'?'#59f':
                   e.level==='status'?'#8ff':'#8f8',
            opacity: e.level==='error'?1:0.85,
            marginBottom: 2,
            padding: '2px 0',
            wordBreak: 'break-word'
          }}
        >
          <span style={{ color: '#666', marginRight: 8 }}>
            [{e.level.toUpperCase()}]
          </span>
          {e.t}
        </div>
      ))}
      {events.length === 0 && (
        <div style={{
          color: '#666',
          fontStyle: 'italic',
          textAlign: 'center',
          padding: '40px 20px'
        }}>
          Waiting for events...
        </div>
      )}
    </div>
  );
}
