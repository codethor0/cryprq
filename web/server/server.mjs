import express from 'express';
import cors from 'cors';
import { spawn } from 'node:child_process';
import { exec } from 'node:child_process';
import { promisify } from 'util';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const execAsync = promisify(exec);
const app = express();
app.use(cors());
app.use(express.json());

// Serve static files from the web dist directory (built files)
// If dist doesn't exist, serve from parent directory (for development)
const distPath = join(__dirname, '..', 'dist');
const staticPath = require('fs').existsSync(distPath) ? distPath : join(__dirname, '..');
app.use(express.static(staticPath));

// Fallback to index.html for SPA routing
app.get('*', (req, res) => {
    const indexPath = join(staticPath, 'index.html');
    if (require('fs').existsSync(indexPath)) {
        res.sendFile(indexPath);
    } else {
        res.status(404).send('Web UI not found. Run: cd web && npm run build');
    }
});

// Check if Docker mode is enabled
const USE_DOCKER = process.env.USE_DOCKER === 'true' || process.env.USE_DOCKER === '1';
const CONTAINER_NAME = process.env.CRYPRQ_CONTAINER || 'cryprq-listener';

// Helper to get container IP
async function getContainerIP() {
    try {
        const { stdout } = await execAsync(
            `docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' ${CONTAINER_NAME}`
        );
        return stdout.trim();
    } catch {
        return null;
    }
}

// Helper to check if container is running
async function isContainerRunning() {
    try {
        const { stdout } = await execAsync(
            `docker ps --filter "name=${CONTAINER_NAME}" --format "{{.Names}}"`
        );
        return stdout.trim() === CONTAINER_NAME;
    } catch {
        return false;
    }
}

// Helper to get container logs
async function getContainerLogs(lines = 20) {
    try {
        const { stdout } = await execAsync(
            `docker logs --tail ${lines} ${CONTAINER_NAME} 2>&1`
        );
        return stdout;
    } catch {
        return '';
    }
}

let proc = null;
let currentMode = null;
let currentPort = null;
const events = [];
function push(level, t){ const e={level,t}; events.push(e); if(events.length>500) events.shift(); }

app.post('/connect', async (req,res)=>{
  const { mode, port, peer, vpn } = req.body || {};
  
  // If Docker mode is enabled, use container instead of local process
  if (USE_DOCKER) {
    const containerRunning = await isContainerRunning();
    if (!containerRunning) {
      push('error', `Container ${CONTAINER_NAME} is not running`);
      push('status', `Start container with: ./scripts/docker-vpn-start.sh`);
      return res.status(500).json({ error: 'Container not running' });
    }
    
    const containerIP = await getContainerIP();
    if (!containerIP) {
      push('error', 'Could not get container IP');
      return res.status(500).json({ error: 'Could not get container IP' });
    }
    
    // For listener mode, container is already listening
    if (mode === 'listener') {
      push('status', `🐳 Container ${CONTAINER_NAME} is listening on port ${port}`);
      push('status', `Container IP: ${containerIP}`);
      push('status', `Connect to: /ip4/${containerIP}/udp/${port}/quic-v1`);
      push('status', `✅ Docker VPN mode active - container handling encryption`);
      
      // Stream container logs
      const logs = await getContainerLogs(20);
      logs.split('\n').filter(Boolean).forEach(line => {
        let level = 'info';
        if (/🔐|ENCRYPT|encrypt/i.test(line)) level = 'rotation';
        else if (/🔓|DECRYPT|decrypt/i.test(line)) level = 'rotation';
        else if (/rotate|rotation/i.test(line)) level = 'rotation';
        else if (/peer|connect|handshake|ping|connected/i.test(line)) level = 'peer';
        else if (/vpn|tun|interface/i.test(line)) level = 'status';
        else if (/error|failed|panic/i.test(line)) level = 'error';
        push(level, line);
      });
      
      return res.json({ 
        ok: true, 
        vpn: !!vpn,
        containerIP,
        containerName: CONTAINER_NAME,
        mode: 'listener',
        dockerMode: true
      });
    }
    
    // For dialer mode, Mac connects to container
    if (mode === 'dialer') {
      const containerPeer = `/ip4/${containerIP}/udp/${port}/quic-v1`;
      push('status', `🐳 Connecting Mac to container at ${containerPeer}`);
      push('status', `✅ Docker VPN mode - container will handle encryption and routing`);
      
      // Run local cryprq binary to connect to container
      const args = ['--peer', containerPeer];
      if (vpn) args.push('--vpn');
      
      proc = spawn(process.env.CRYPRQ_BIN || 'cryprq', args, {
        stdio: ['ignore', 'pipe', 'pipe'],
        env: { ...process.env, RUST_LOG: 'debug' }
      });
      
      currentMode = mode;
      currentPort = port;
      push('status', `spawn ${args.join(' ')}`);
      
      proc.stdout.on('data', d => {
        const s = d.toString();
        s.split(/\r?\n/).filter(Boolean).forEach(line => {
          let level = 'info';
          if (/🔐|ENCRYPT|encrypt/i.test(line)) level = 'rotation';
          else if (/🔓|DECRYPT|decrypt/i.test(line)) level = 'rotation';
          else if (/rotate|rotation/i.test(line)) level = 'rotation';
          else if (/peer|connect|handshake|ping|connected/i.test(line)) level = 'peer';
          else if (/vpn|tun|interface/i.test(line)) level = 'status';
          else if (/error|failed|panic/i.test(line)) level = 'error';
          push(level, line);
        });
      });
      
      proc.stderr.on('data', d => {
        const s = d.toString();
        s.split(/\r?\n/).filter(Boolean).forEach(line => {
          let level = 'error';
          if (/INFO|DEBUG|TRACE/i.test(line)) {
            level = 'info';
            if (/🔐|ENCRYPT|encrypt/i.test(line)) level = 'rotation';
            else if (/🔓|DECRYPT|decrypt/i.test(line)) level = 'rotation';
            else if (/rotate|rotation/i.test(line)) level = 'rotation';
            else if (/peer|connect|handshake|ping|connected/i.test(line)) level = 'peer';
          }
          push(level, line);
        });
      });
      
      proc.on('exit', (code, signal) => {
        if (code === 0) {
          push('status', `exit ${code} (clean shutdown)`);
        } else if (code === null && signal) {
          push('error', `Process killed by signal: ${signal}`);
        } else if (code === null) {
          push('error', `Process exited unexpectedly (exit code: null, signal: ${signal || 'none'})`);
        } else {
          push('error', `exit ${code} (signal: ${signal || 'none'})`);
        }
        proc = null;
        currentMode = null;
        currentPort = null;
      });
      
      proc.on('error', (err) => {
        push('error', `Process spawn error: ${err.message}`);
        proc = null;
        currentMode = null;
        currentPort = null;
      });
      
      return res.json({ 
        ok: true, 
        vpn: !!vpn,
        containerIP,
        containerPeer,
        mode: 'dialer',
        dockerMode: true
      });
    }
    
    return res.status(400).json({ error: 'mode must be listener or dialer' });
  }
  
  // Local mode (original code)
  // Only kill existing process if we're switching modes or ports
  // This prevents killing the listener when dialer tries to connect
  if(proc && (currentMode !== mode || currentPort !== port)) {
    push('status', `🔄 Switching from ${currentMode} to ${mode} on port ${port}`);
    proc.kill('SIGKILL'); 
    proc = null;
    currentMode = null;
    currentPort = null;
    
    // Wait for process to die
    const { execSync } = require('child_process');
    try {
      execSync('sleep 0.5', {stdio: 'ignore'});
    } catch(e) {}
  }
  
  // If we already have a process running for this exact mode/port, don't restart it
  if(proc && currentMode === mode && currentPort === port) {
    push('status', `ℹ️ ${mode} already running on port ${port} - keeping alive`);
    res.json({ok:true, vpn: !!vpn, alreadyRunning: true});
    return;
  }
  
  // Kill any cryprq processes on this port ONLY if we're starting a listener
  // For dialer, we want to keep the listener alive
  const { execSync } = require('child_process');
  if(mode === 'listener') {
    try {
      // Only kill processes if we don't already have THIS listener running
      // This prevents killing our own process
      if(!proc || currentMode !== 'listener' || currentPort !== port) {
        // Get PIDs using this port BEFORE we spawn
        const existingPids = execSync(`lsof -ti:${port} 2>/dev/null || echo ""`, {encoding: 'utf8'}).trim();
        if(existingPids) {
          // Kill existing processes (but remember we'll spawn a new one)
          execSync(`lsof -ti:${port} | xargs kill -9 2>/dev/null || true`, {stdio: 'ignore'});
          execSync('sleep 0.5', {stdio: 'ignore'}); // Give processes time to die
          push('status', `🧹 Cleaned up port ${port} - ready for listener`);
        }
      }
    } catch(e) {}
  } else if(mode === 'dialer') {
    // For dialer, check if listener is running - if not, warn user
    try {
      const portUsers = execSync(`lsof -ti:${port} 2>/dev/null || echo ""`, {encoding: 'utf8'}).trim();
      if(!portUsers) {
        push('status', `⚠️ No listener detected on port ${port} - make sure listener is running first`);
      } else {
        push('status', `✅ Listener detected on port ${port} - connecting...`);
      }
    } catch(e) {}
  }
  let args = [];
  if(mode==='listener') args = ['--listen', `/ip4/0.0.0.0/udp/${port}/quic-v1`];
  else if(mode==='dialer') args = ['--peer', peer || `/ip4/127.0.0.1/udp/${port}/quic-v1`];
  else return res.status(400).json({error:'mode must be listener or dialer'});

  // Add VPN mode flag if requested
  if(vpn) {
    args.push('--vpn');
    push('status', '🔒 VPN MODE ENABLED - System-wide routing mode');
    push('status', '⚠️ Note: Full system routing requires Network Extension framework on macOS');
    push('status', '✅ P2P encrypted tunnel is active - all peer traffic is encrypted');
  }

  // Set maximum verbosity
  const env = { ...process.env, RUST_LOG: 'debug' };
  
  // Spawn process with proper stdio handling
  proc = spawn(process.env.CRYPRQ_BIN || 'cryprq', args, { 
    stdio: ['ignore','pipe','pipe'], // stdin: ignore, stdout/stderr: pipe for logging
    env: env
  });
  currentMode = mode;
  currentPort = port;
  push('status', `spawn ${args.join(' ')}`);
  proc.stdout.on('data', d=>{
    const s=d.toString();
    s.split(/\r?\n/).filter(Boolean).forEach(line=>{
      let level='info';
      if(/🔐|ENCRYPT|encrypt/i.test(line)) level='rotation'; // Encryption events
      else if(/🔓|DECRYPT|decrypt/i.test(line)) level='rotation'; // Decryption events
      else if(/rotate|rotation/i.test(line)) level='rotation';
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
        if(/🔐|ENCRYPT|encrypt/i.test(line)) level='rotation'; // Encryption events
        else if(/🔓|DECRYPT|decrypt/i.test(line)) level='rotation'; // Decryption events
        else if(/rotate|rotation/i.test(line)) level='rotation';
        else if(/peer|connect|handshake|ping|connected|listening/i.test(line)) level='peer';
        else if(/listening on/i.test(line)) level='peer'; // Listening is a peer event
      } else if(/listening on/i.test(line)) {
        level='peer'; // Listening messages are peer events
      } else if(/Address already in use/i.test(line)) {
        level='error';
        push('status', `⚠️ Port ${port} is in use - killing existing processes...`);
      }
      push(level, line);
    });
  });
  proc.on('exit', (code, signal)=>{
    if(code === 0) {
      push('status', `exit ${code} (clean shutdown)`);
    } else if(code === null && signal) {
      push('error', `Process killed by signal: ${signal}`);
    } else if(code === null) {
      push('error', `Process exited unexpectedly (exit code: null, signal: ${signal || 'none'})`);
    } else {
      push('error', `exit ${code} (signal: ${signal || 'none'})`);
    }
    // Clear process tracking when it exits
    proc = null;
    currentMode = null;
    currentPort = null;
  });
  
  // Handle process errors
  proc.on('error', (err)=>{
    push('error', `Process spawn error: ${err.message}`);
    proc = null;
    currentMode = null;
    currentPort = null;
  });
  
  // Log when process starts
  push('status', `Process started (PID: ${proc.pid}, mode: ${mode}, port: ${port})`);
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
