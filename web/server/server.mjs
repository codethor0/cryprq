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
  if(proc) { proc.kill('SIGKILL'); proc = null; }
  let args = [];
  if(mode==='listener') args = ['--listen', `/ip4/0.0.0.0/udp/${port}/quic-v1`];
  else if(mode==='dialer') args = ['--peer', peer || `/ip4/127.0.0.1/udp/${port}/quic-v1`];
  else return res.status(400).json({error:'mode must be listener or dialer'});

  // Add VPN mode flag if requested
  if(vpn) {
    args.push('--vpn');
    push('status', 'VPN mode enabled - system-wide routing');
  }

  proc = spawn(process.env.CRYPRQ_BIN || 'cryprq', args, { stdio: ['ignore','pipe','pipe']});
  push('status', `spawn ${args.join(' ')}`);
  proc.stdout.on('data', d=>{
    const s=d.toString();
    s.split(/\r?\n/).filter(Boolean).forEach(line=>{
      let level='info';
      if(/rotate|rotation/i.test(line)) level='rotation';
      else if(/peer|connect|handshake|ping/i.test(line)) level='peer';
      else if(/vpn|tun|interface/i.test(line)) level='status';
      push(level, line);
    });
  });
  proc.stderr.on('data', d=>push('error', d.toString()));
  proc.on('exit', (code)=>push('status', `exit ${code}`));
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
