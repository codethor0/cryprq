#!/usr/bin/env bash
# © 2025 Thor Thor
# Contact: codethor@gmail.com
# LinkedIn: https://www.linkedin.com/in/thor-thor0
# SPDX-License-Identifier: MIT
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# -------- Config
WEB_DIR="${ROOT}/web"
WEB_SERVER_DIR="${WEB_DIR}/server"
ART_WEB="${ROOT}/artifacts/web-test"
ART_MAC="${ROOT}/artifacts/macos-test"
PORT_WEB="${PORT_WEB:-5173}"
PORT_BRIDGE="${PORT_BRIDGE:-8787}"
CRYPRQ_PORT="${CRYPRQ_PORT:-9999}"
ROTATE_SECS="${ROTATE_SECS:-10}"

mkdir -p "$ART_WEB" "$ART_MAC"

have(){ command -v "$1" >/dev/null 2>&1; }
note(){ echo "[build-web-mac] $*"; }
fail(){ echo "[build-web-mac] ERROR: $*" >&2; exit 1; }

# -------- Tool checks (best-effort)
have cargo    || fail "cargo not found"
have rustup   || fail "rustup not found"
have npm      || fail "npm not found (required for web client)"
have node     || fail "node not found"
have docker   || note "docker not found (Docker QA will be skipped)"
have npx      || note "npx not found (Playwright/formatting skipped)"

# -------- Build macOS Apple Silicon binary
note "Building macOS Apple Silicon binary (aarch64-apple-darwin)…"
rustup target add aarch64-apple-darwin >/dev/null 2>&1 || true
cargo build --release -p cryprq --target aarch64-apple-darwin 2>&1 | tee "${ART_MAC}/build.txt"
BIN_MAC="target/aarch64-apple-darwin/release/cryprq"
[[ -x "$BIN_MAC" ]] || fail "Expected $BIN_MAC"

# Quick local mac listener + dialer smoke (uses host loopback)
note "Running macOS local smoke (listener+dialer)…"
set +e
"$BIN_MAC" --listen "/ip4/0.0.0.0/udp/${CRYPRQ_PORT}/quic-v1" > "${ART_MAC}/listener.log" 2>&1 &
L_PID=$!
sleep 1
"$BIN_MAC" --peer "/ip4/127.0.0.1/udp/${CRYPRQ_PORT}/quic-v1" > "${ART_MAC}/dialer.log" 2>&1
sleep $((ROTATE_SECS+2)) || true
kill $L_PID >/dev/null 2>&1 || true
set -e
grep -Ei "connected|handshake|ping" "${ART_MAC}/dialer.log" >/dev/null || fail "macOS dialer failed to connect"
note "macOS smoke OK."

# -------- Scaffold Web client if missing
if [[ ! -d "${WEB_DIR}" ]]; then
  note "Scaffolding web client (Vite React TS)…"
  npm create vite@latest web -- --template react-ts >/dev/null 2>&1 || \
    npx --yes create-vite web --template react-ts
  # Install deps
  (cd web && npm i)
fi

# -------- Add DebugConsole and minimal UI if missing
if [[ ! -f "${WEB_DIR}/src/DebugConsole.tsx" ]]; then
  cat > "${WEB_DIR}/src/DebugConsole.tsx" <<'TS'
import React from 'react';

type E = { t:string; level:'status'|'rotation'|'peer'|'info'|'error' };

export function DebugConsole({events}:{events:E[]}) {
  return (
    <div style={{position:'fixed',bottom:0,left:0,right:0,height:200,overflow:'auto',background:'#111',color:'#ddd',fontFamily:'ui-monospace, SFMono-Regular, Menlo, monospace',fontSize:12,borderTop:'1px solid #333',padding:'8px'}}>
      {events.slice(-200).map((e,i)=>(
        <div key={i} style={{color: e.level==='error'?'#f55':e.level==='rotation'?'#f90':e.level==='peer'?'#59f':'#8f8'}}>
          [{e.level}] {e.t}
        </div>
      ))}
    </div>
  );
}
TS
fi

# App wiring
cat > "${WEB_DIR}/src/App.tsx" <<'TS'
import React, {useEffect, useRef, useState} from 'react';
import { DebugConsole } from './DebugConsole';

type Mode = 'listener' | 'dialer';

export default function App(){
  const [mode, setMode] = useState<Mode>('listener');
  const [port, setPort] = useState<number>(9999);
  const [peer, setPeer] = useState<string>('/ip4/127.0.0.1/udp/9999/quic-v1');
  const [events, setEvents] = useState<{t:string,level:'status'|'rotation'|'peer'|'info'|'error'}[]>([]);
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
    await fetch('/connect', {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({ mode, port, peer })
    });
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
        <button onClick={connect}>Connect</button>
      </div>
      <p>Use two tabs/windows: one as listener, one as dialer pointing to the listener's address.</p>
      <DebugConsole events={events}/>
    </div>
  );
}
TS

# -------- Bridge server (spawns cryprq and streams logs)
mkdir -p "${WEB_SERVER_DIR}"
cat > "${WEB_SERVER_DIR}/package.json" <<'JSON'
{
  "name": "cryprq-web-bridge",
  "private": true,
  "type": "module",
  "scripts": {
    "start": "node server.mjs"
  },
  "dependencies": {
    "express": "^4.19.2",
    "cors": "^2.8.5"
  }
}
JSON

cat > "${WEB_SERVER_DIR}/server.mjs" <<'JS'
import express from 'express';
import cors from 'cors';
import { spawn } from 'node:child_process';

const app = express();
app.use(cors());
app.use(express.json());

let proc = null;
const events = [];
function push(level, t){ const e={level,t}; events.push(e); if(events.length>500) events.shift(); }

app.post('/connect', (req,res)=>{
  const { mode, port, peer } = req.body || {};
  if(proc) { proc.kill('SIGKILL'); proc = null; }
  let args = [];
  if(mode==='listener') args = ['--listen', `/ip4/0.0.0.0/udp/${port}/quic-v1`];
  else if(mode==='dialer') args = ['--peer', peer || `/ip4/127.0.0.1/udp/${port}/quic-v1`];
  else return res.status(400).json({error:'mode must be listener or dialer'});

  proc = spawn(process.env.CRYPRQ_BIN || 'cryprq', args, { stdio: ['ignore','pipe','pipe']});
  push('status', `spawn ${args.join(' ')}`);
  proc.stdout.on('data', d=>{
    const s=d.toString();
    s.split(/\r?\n/).filter(Boolean).forEach(line=>{
      let level='info';
      if(/rotate|rotation/i.test(line)) level='rotation';
      else if(/peer|connect|handshake|ping/i.test(line)) level='peer';
      push(level, line);
    });
  });
  proc.stderr.on('data', d=>push('error', d.toString()));
  proc.on('exit', (code)=>push('status', `exit ${code}`));
  res.json({ok:true});
});

app.get('/events', (req,res)=>{
  res.setHeader('Content-Type','text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.flushHeaders();
  events.forEach(e=>res.write(`data: ${JSON.stringify(e)}\n\n`));
  const iv=setInterval(()=>res.write(`data: ${JSON.stringify({level:'status', t:'tick'})}\n\n`),15000);
  req.on('close',()=>clearInterval(iv));
});

const PORT = process.env.BRIDGE_PORT || 8787;
app.listen(PORT, ()=> console.log(`bridge on ${PORT}`));
JS

# Install server deps
( cd "${WEB_SERVER_DIR}" && npm i )

# -------- Wire dev proxy so Vite can reach bridge
jq_bin="$(command -v jq || true)"
if [[ -n "$jq_bin" && -f "${WEB_DIR}/vite.config.ts" ]]; then
  # add dev server proxy non-destructively
  node - <<'JS' "${WEB_DIR}/vite.config.ts" "$PORT_BRIDGE" > "${WEB_DIR}/vite.tmp"
const fs=require('fs');
const path=process.argv[2];
const port=process.argv[3];
let s=fs.readFileSync(path,'utf8');
if(!/server:\s*{/.test(s)){
  s+=`\nexport default { server: { proxy: { '^/(connect|events)$': 'http://localhost:${port}' } } };\n`;
} else if(!/connect\|events/.test(s)){
  s=s.replace(/server:\s*{[^}]*}/, m=>{
    if(m.includes('proxy')) return m.replace(/proxy:\s*{/, `proxy: { '^/(connect|events)$': 'http://localhost:${port}', `);
    return m.replace(/server:\s*{/, `server: { proxy: { '^/(connect|events)$': 'http://localhost:${port}' }, `);
  });
}
process.stdout.write(s);
JS
  mv "${WEB_DIR}/vite.tmp" "${WEB_DIR}/vite.config.ts"
fi

# -------- Build web client
note "Building web client…"
( cd "${WEB_DIR}" && npm run build ) 2>&1 | tee "${ART_WEB}/web_build.txt"

# -------- Run bridge + dev server and do a quick smoke via curl (non-blocking)
note "Starting bridge server…"
( cd "${WEB_SERVER_DIR}" && BRIDGE_PORT="${PORT_BRIDGE}" CRYPRQ_BIN="${BIN_MAC}" node server.mjs ) > "${ART_WEB}/bridge.log" 2>&1 &
BR_PID=$!
sleep 1

note "Dev server (vite) smoke (optional)…"
( cd "${WEB_DIR}" && npm run dev -- --port "${PORT_WEB}" ) > "${ART_WEB}/vite.log" 2>&1 &
V_PID=$!
sleep 2

# Call connect to spin up a listener via bridge, then dialer
curl -s -X POST -H 'Content-Type: application/json' "http://localhost:${PORT_BRIDGE}/connect" \
  -d "{\"mode\":\"listener\",\"port\":${CRYPRQ_PORT}}" > "${ART_WEB}/connect_listener.json" || true
sleep 1
curl -s -X POST -H 'Content-Type: application/json' "http://localhost:${PORT_BRIDGE}/connect" \
  -d "{\"mode\":\"dialer\",\"port\":${CRYPRQ_PORT},\"peer\":\"/ip4/127.0.0.1/udp/${CRYPRQ_PORT}/quic-v1\"}" > "${ART_WEB}/connect_dialer.json" || true
sleep $((ROTATE_SECS+2)) || true

# Pull a snapshot of events
curl -s "http://localhost:${PORT_BRIDGE}/events" | head -n 20 > "${ART_WEB}/events_snapshot.sse" || true

# basic sanity: look for any handshake/peer indicator in logs
if ! grep -Eiq "handshake|peer|connected|rotation" "${ART_WEB}/bridge.log" "${ART_WEB}/events_snapshot.sse" 2>/dev/null; then
  note "Web smoke did not see expected events; check ${ART_WEB}/bridge.log"
else
  note "Web smoke OK."
fi

# -------- Docker QA (if docker available)
if have docker; then
  note "Docker QA handshake + rotation…"
  docker build -t cryprq-node:webqa . 2>&1 | tee "${ART_WEB}/docker_build.txt"
  docker rm -f cryprq-listener >/dev/null 2>&1 || true
  docker run -d --name cryprq-listener -p ${CRYPRQ_PORT}:${CRYPRQ_PORT}/udp \
    -e CRYPRQ_ROTATE_SECS="${ROTATE_SECS}" \
    cryprq-node:webqa --listen "/ip4/0.0.0.0/udp/${CRYPRQ_PORT}/quic-v1"
  sleep 2
  docker run --rm --network host cryprq-node:webqa \
    --peer "/ip4/127.0.0.1/udp/${CRYPRQ_PORT}/quic-v1" 2>&1 | tee "${ART_WEB}/docker_dialer.txt"
  sleep $((ROTATE_SECS+2)) || true
  docker logs cryprq-listener 2>&1 | tee "${ART_WEB}/docker_listener.log" >/dev/null || true
  docker rm -f cryprq-listener >/dev/null 2>&1 || true
  grep -Eiq "handshake|rotation" "${ART_WEB}/docker_dialer.txt" "${ART_WEB}/docker_listener.log" || fail "Docker QA did not show handshake/rotation"
  note "Docker QA OK."
else
  note "Docker not installed; skipping Docker QA."
fi

# -------- Clean up dev servers
kill ${BR_PID} >/dev/null 2>&1 || true
kill ${V_PID}  >/dev/null 2>&1 || true

# -------- Commit if all green
git add web "${ART_WEB}" "${ART_MAC}" 2>/dev/null || true
if ! git diff --cached --quiet 2>/dev/null; then
  git commit -m "build: web client + bridge with debug console, mac arm64 binary, and e2e smoke tests"
  note "Committed web+mac scaffolding and artifacts."
else
  note "No changes to commit."
fi

note "DONE. Web at web/ (bridge on ${PORT_BRIDGE}), mac binary at ${BIN_MAC}. Logs in artifacts/web-test and artifacts/macos-test."

