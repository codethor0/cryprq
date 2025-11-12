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
        if (text.includes('Connected to')) {
          const match = text.match(/Connected to (\S+)/);
          if (match) {
            setStatus(prev => ({ 
              ...prev, 
              connected: true, 
              connectedPeer: match[1],
              mode: mode
            }));
          }
        }
        
        // Listener receives connection (check for incoming connection events)
        if (text.includes('New connection') || text.includes('Connection established')) {
          setStatus(prev => ({ 
            ...prev, 
            connected: true,
            mode: mode
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
        
        // Check for disconnection (exit 0 means clean shutdown, non-zero means error)
        if (text.includes('exit 1') || text.includes('exit 2') || (text.includes('exit') && text.includes('Error'))) {
          setStatus(prev => ({ ...prev, connected: false, connectedPeer: undefined }));
        }
        
        // On clean exit, also mark as disconnected
        if (text.includes('exit 0')) {
          setStatus(prev => ({ ...prev, connected: false }));
        }
        
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
        body: JSON.stringify({ mode, port, peer })
      });
      if (!res.ok) {
        const err = await res.text();
        setEvents(prev=>[...prev, {t:`Connect failed: ${err}`, level:'error'}]);
      } else {
        await res.json();
        setEvents(prev=>[...prev, {t:`Connect initiated: ${mode} on port ${port}`, level:'status'}]);
        setStatus(prev => ({ ...prev, mode: mode, connected: false }));
      }
    } catch (err: any) {
      setEvents(prev=>[...prev, {t:`Connect error: ${err.message || err}`, level:'error'}]);
    } finally {
      setConnecting(false);
    }
  }

  return (
    <div style={{paddingBottom:232,padding:16}}>
      <h1>CrypRQ Web Tester</h1>
      <div style={{display:'flex',gap:12,alignItems:'center',flexWrap:'wrap'}}>
        <label>Mode:
          <select value={mode} onChange={e=>setMode(e.target.value as Mode)}>
            <option value="listener">listener</option>
            <option value="dialer">dialer</option>
          </select>
        </label>
        <label>Port: <input type="number" value={port} onChange={e=>setPort(parseInt(e.target.value||'0'))}/></label>
        <label>Peer: <input style={{width:420}} value={peer} onChange={e=>setPeer(e.target.value)}/></label>
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
