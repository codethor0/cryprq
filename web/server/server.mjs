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
  const { mode, port, peer, vpn } = req.body || {};
  if(proc) { 
    proc.kill('SIGKILL'); 
    proc = null; 
    // Give it a moment to release the port
    setTimeout(() => {}, 500);
  }
  let args = [];
  if(mode==='listener') args = ['--listen', `/ip4/0.0.0.0/udp/${port}/quic-v1`];
  else if(mode==='dialer') args = ['--peer', peer || `/ip4/127.0.0.1/udp/${port}/quic-v1`];
  else return res.status(400).json({error:'mode must be listener or dialer'});

  // Add VPN mode flag if requested
  if(vpn) {
    args.push('--vpn');
    push('status', 'VPN mode enabled - system-wide routing');
  }

  // Set maximum verbosity
  const env = { ...process.env, RUST_LOG: 'debug' };
  
  proc = spawn(process.env.CRYPRQ_BIN || 'cryprq', args, { 
    stdio: ['ignore','pipe','pipe'],
    env: env
  });
  push('status', `spawn ${args.join(' ')}`);
  proc.stdout.on('data', d=>{
    const s=d.toString();
    s.split(/\r?\n/).filter(Boolean).forEach(line=>{
      let level='info';
      if(/rotate|rotation/i.test(line)) level='rotation';
      else if(/peer|connect|handshake|ping|connected/i.test(line)) level='peer';
      else if(/vpn|tun|interface/i.test(line)) level='status';
      else if(/error|failed|panic/i.test(line)) level='error';
      else if(/debug|trace/i.test(line)) level='info';
      push(level, line);
    });
  });
  proc.stderr.on('data', d=>{
    const s=d.toString();
    s.split(/\r?\n/).filter(Boolean).forEach(line=>{
      // stderr can contain both errors and info logs
      let level='error';
      if(/INFO|DEBUG|TRACE/i.test(line)) {
        level='info';
        if(/rotate|rotation/i.test(line)) level='rotation';
        if(/peer|connect|handshake|ping|connected/i.test(line)) level='peer';
      }
      push(level, line);
    });
  });
  proc.on('exit', (code)=>{
    if(code === 0) {
      push('status', `exit ${code} (clean shutdown)`);
    } else {
      push('status', `exit ${code} (error)`);
    }
  });
  res.json({ok:true, vpn: !!vpn});
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
