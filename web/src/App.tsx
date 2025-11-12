import {useEffect, useRef, useState} from 'react';
import { DebugConsole } from './DebugConsole';
import { EncryptionStatus } from './EncryptionStatus';

type Mode = 'listener' | 'dialer';

type Status = {
  connected: boolean;
  encryption: 'ML-KEM (Kyber768) + X25519 hybrid' | 'None';
  peerId?: string;
  connectedPeer?: string;
  keyEpoch?: number;
  rotationInterval?: number;
  mode?: 'listener' | 'dialer';
};

export default function App(){
  const [mode, setMode] = useState<Mode>('listener');
  const [port, setPort] = useState<number>(9999);
  const [peer, setPeer] = useState<string>('/ip4/127.0.0.1/udp/9999/quic-v1');
  const [vpnMode, setVpnMode] = useState<boolean>(false);
  const [events, setEvents] = useState<{t:string,level:'status'|'rotation'|'peer'|'info'|'error'}[]>([]);
  const [connecting, setConnecting] = useState(false);
  const [status, setStatus] = useState<Status>({
    connected: false,
    encryption: 'ML-KEM (Kyber768) + X25519 hybrid'
  });
  const esRef = useRef<EventSource|null>(null);

  useEffect(()=>{
    if(esRef.current) esRef.current.close();
    const es = new EventSource('/events');
    es.onmessage = (m)=> {
      try {
        const e = JSON.parse(m.data);
        setEvents(prev=>[...prev, e]);
        
        // Parse status from events
        const text = e.t || '';
        
        // Extract peer ID
        if (text.includes('Local peer id:')) {
          const match = text.match(/Local peer id: (\S+)/);
          if (match) {
            setStatus(prev => ({ ...prev, peerId: match[1] }));
          }
        }
        
        // Extract connected peer (dialer connects to peer, listener receives connection)
        // Match "Connected to <peer-id> via Dialer" or similar patterns
        if (text.includes('Connected to') || text.includes('connected to')) {
          // Try to match peer ID (starts with 12D3KooW followed by alphanumeric)
          const match = text.match(/[Cc]onnected to (12D3KooW\w+)/);
          if (match) {
            setStatus(prev => ({ 
              ...prev, 
              connected: true, 
              connectedPeer: match[1],
              mode: mode
            }));
          } else {
            // Fallback: if we see "Connected to" but can't extract peer, still mark as connected
            setStatus(prev => ({ 
              ...prev, 
              connected: true,
              mode: mode
            }));
          }
        }
        
        // Listener is listening - mark as ready (will be connected when peer dials)
        if (text.includes('Listening on') || text.includes('listening on')) {
          setStatus(prev => ({ 
            ...prev, 
            mode: mode,
            // Keep connected status if already connected, otherwise ready but not connected yet
          }));
        }
        
        // Listener receives connection (check for incoming connection events)
        if (text.includes('New connection') || text.includes('Connection established') || 
            text.includes('connection established') || text.includes('peer connected') ||
            text.includes('Incoming connection')) {
          setStatus(prev => ({ 
            ...prev, 
            connected: true,
            mode: mode
          }));
        }
        
        // Check for successful dialer connection
        if (text.includes('Dialing peer') && mode === 'dialer') {
          setStatus(prev => ({ 
            ...prev, 
            mode: mode,
            connected: false // Reset on new dial attempt
          }));
        }
        
        // Check for listener starting - this means it's ready
        if (text.includes('Starting listener') && mode === 'listener') {
          setStatus(prev => ({ 
            ...prev, 
            mode: mode,
            // Don't reset connected - might already be connected from previous attempt
          }));
        }
        
        // Extract key rotation info
        if (text.includes('key_rotation') && text.includes('epoch=')) {
          const epochMatch = text.match(/epoch=(\d+)/);
          const intervalMatch = text.match(/interval_secs=(\d+)/);
          if (epochMatch) {
            setStatus(prev => ({ 
              ...prev, 
              keyEpoch: parseInt(epochMatch[1]),
              rotationInterval: intervalMatch ? parseInt(intervalMatch[1]) : prev.rotationInterval
            }));
          }
        }
        
        // Check for disconnection - only mark disconnected on fatal errors
        // Don't disconnect on clean exits (exit 0) - those are normal shutdowns
        if (text.includes('exit 1') || text.includes('exit 2')) {
          // Only disconnect if it's a fatal error, not just a port conflict
          if (text.includes('Address already in use')) {
            // Port conflict - don't disconnect, just note the error
            // The connection might still be active from a previous instance
          } else if (text.includes('fatal') || text.includes('FATAL') || text.includes('panic')) {
            setStatus(prev => ({ ...prev, connected: false, connectedPeer: undefined }));
          }
        }
        
        // On clean exit (exit 0), preserve connection status
        // The connection was successful, process is just shutting down
        // Keep the connected status to show it was working
        
        // Check for successful connection start
        if (text.includes('Post-quantum encryption enabled')) {
          setStatus(prev => ({ 
            ...prev, 
            encryption: 'ML-KEM (Kyber768) + X25519 hybrid',
            mode: mode
          }));
        }
      } catch {}
    };
    es.onerror = ()=> setEvents(prev=>[...prev, {t:'event stream error', level:'error'}]);
    esRef.current = es;
    return ()=>{ es.close(); };
  },[mode]);

  async function connect(){
    setConnecting(true);
    try {
      const res = await fetch('/connect', {
        method:'POST',
        headers:{'Content-Type':'application/json'},
        body: JSON.stringify({ mode, port, peer, vpn: vpnMode })
      });
      if (!res.ok) {
        const err = await res.text();
        setEvents(prev=>[...prev, {t:`Connect failed: ${err}`, level:'error'}]);
      } else {
        const data = await res.json();
        const vpnMsg = vpnMode ? ' (VPN mode: system-wide routing)' : '';
        setEvents(prev=>[...prev, {t:`Connect initiated: ${mode} on port ${port}${vpnMsg}`, level:'status'}]);
        setStatus(prev => ({ ...prev, mode: mode, connected: false }));
      }
    } catch (err: any) {
      setEvents(prev=>[...prev, {t:`Connect error: ${err.message || err}`, level:'error'}]);
    } finally {
      setConnecting(false);
    }
  }

  return (
    <div style={{paddingBottom:270,padding:16}}>
      <h1 style={{display:'flex', alignItems:'center', gap:'12px'}}>
        <img 
          src="/icon_master_1024.png" 
          alt="CrypRQ Icon" 
          style={{width:'32px', height:'32px', borderRadius:'4px'}}
          onError={(e) => {
            // Fallback if icon not found
            e.currentTarget.style.display = 'none';
          }}
        />
        CrypRQ Web Tester
      </h1>
      <div style={{display:'flex',gap:12,alignItems:'center',flexWrap:'wrap'}}>
        <label>Mode:
          <select value={mode} onChange={e=>setMode(e.target.value as Mode)}>
            <option value="listener">listener</option>
            <option value="dialer">dialer</option>
          </select>
        </label>
        <label>Port: <input type="number" value={port} onChange={e=>setPort(parseInt(e.target.value||'0'))}/></label>
        <label>Peer: <input style={{width:420}} value={peer} onChange={e=>setPeer(e.target.value)}/></label>
        <label style={{display:'flex', alignItems:'center', gap:'8px'}}>
          <input type="checkbox" checked={vpnMode} onChange={e=>setVpnMode(e.target.checked)}/>
          <span>VPN Mode (system-wide routing)</span>
        </label>
        <button onClick={connect} disabled={connecting}>
          {connecting ? 'Connecting...' : 'Connect'}
        </button>
      </div>
      <p>Use two tabs/windows: one as listener, one as dialer pointing to the listener's address.</p>
      <EncryptionStatus status={status} />
      <DebugConsole events={events}/>
    </div>
  );
}
