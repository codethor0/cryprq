import {useEffect, useRef, useState} from 'react';
import { DebugConsole } from './DebugConsole';

type Mode = 'listener' | 'dialer';

export default function App(){
  const [mode, setMode] = useState<Mode>('listener');
  const [port, setPort] = useState<number>(9999);
  const [peer, setPeer] = useState<string>('/ip4/127.0.0.1/udp/9999/quic-v1');
  const [events, setEvents] = useState<{t:string,level:'status'|'rotation'|'peer'|'info'|'error'}[]>([]);
  const [connecting, setConnecting] = useState(false);
  const esRef = useRef<EventSource|null>(null);

  useEffect(()=>{
    if(esRef.current) esRef.current.close();
    const es = new EventSource('/events');
    es.onmessage = (m)=> {
      try {
        const e = JSON.parse(m.data);
        setEvents(prev=>[...prev, e]);
      } catch {}
    };
    es.onerror = ()=> setEvents(prev=>[...prev, {t:'event stream error', level:'error'}]);
    esRef.current = es;
    return ()=>{ es.close(); };
  },[]);

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
      <DebugConsole events={events}/>
    </div>
  );
}
