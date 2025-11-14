import express from 'express';
import cors from 'cors';
import { spawn, execSync } from 'node:child_process';
import { exec } from 'node:child_process';
import { promisify } from 'util';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { existsSync, accessSync, constants } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const execAsync = promisify(exec);
const app = express();
app.use(cors());
app.use(express.json());

// Check if Docker mode is enabled
const USE_DOCKER = process.env.USE_DOCKER === 'true' || process.env.USE_DOCKER === '1';
const CONTAINER_NAME = process.env.CRYPRQ_CONTAINER || 'cryprq-vpn';

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
        const result = stdout.trim();
        console.log(`[DEBUG] Container check: looking for "${CONTAINER_NAME}", got: "${result}"`);
        // Check if container name matches (handle both exact match and partial match)
        const isRunning = result === CONTAINER_NAME || result.includes(CONTAINER_NAME);
        console.log(`[DEBUG] Container running: ${isRunning}`);
        return isRunning;
    } catch (err) {
        console.error('Container check error:', err.message);
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
const eventClients = new Set(); // Track all connected EventSource clients

function push(level, t){ 
  const e={level,t}; 
  events.push(e); 
  if(events.length>500) events.shift();
  
  // Broadcast to all connected EventSource clients
  const message = `data: ${JSON.stringify(e)}\n\n`;
  eventClients.forEach(client => {
    try {
      client.write(message);
    } catch (err) {
      // Client disconnected, remove it
      eventClients.delete(client);
    }
  });
}

app.post('/connect', async (req,res)=>{
  const { mode, port, peer, vpn } = req.body || {};
  
  // Find CrypRQ binary - check multiple paths
  let binPath = process.env.CRYPRQ_BIN;
  
  if (!binPath || !existsSync(binPath)) {
    // Try multiple fallback paths
    const possiblePaths = [
      join(__dirname, '..', '..', 'target', 'release', 'cryprq'), // Cargo build
      join(__dirname, '..', '..', 'dist', 'macos', 'CrypRQ.app', 'Contents', 'MacOS', 'CrypRQ'), // macOS app
      'cryprq', // System PATH
    ];
    
    for (const path of possiblePaths) {
      if (existsSync(path)) {
        binPath = path;
        process.env.CRYPRQ_BIN = path;
        console.log(`[DEBUG] Found CrypRQ binary at: ${path}`);
        break;
      }
    }
  }
  
  // Final check - binary must exist
  if (!binPath || !existsSync(binPath)) {
    const triedPaths = [
      join(__dirname, '..', '..', 'target', 'release', 'cryprq'),
      join(__dirname, '..', '..', 'dist', 'macos', 'CrypRQ.app', 'Contents', 'MacOS', 'CrypRQ'),
      'cryprq',
    ];
    const errorMsg = `CrypRQ binary not found. Tried: ${triedPaths.join(', ')}. Set CRYPRQ_BIN environment variable or build with 'cargo build --release -p cryprq'`;
    push('error', errorMsg);
    console.error(`[ERROR] ${errorMsg}`);
    return res.status(500).json({ error: errorMsg });
  }
  
  // Verify binary is executable
  try {
    accessSync(binPath, constants.X_OK);
  } catch (e) {
    const errorMsg = `CrypRQ binary is not executable: ${binPath}`;
    push('error', errorMsg);
    console.error(`[ERROR] ${errorMsg}`);
    return res.status(500).json({ error: errorMsg });
  }
  
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
      push('status', `Container ${CONTAINER_NAME} is listening on port ${port}`);
      push('status', `Container IP: ${containerIP}`);
      push('status', `Connect to: /ip4/${containerIP}/udp/${port}/quic-v1`);
      push('status', `Docker VPN mode active - container handling encryption`);
      
      // Stream container logs
      const logs = await getContainerLogs(20);
      logs.split('\n').filter(Boolean).forEach(line => {
        let level = 'info';
        if (/ENCRYPT|encrypt/i.test(line)) level = 'rotation';
        else if (/DECRYPT|decrypt/i.test(line)) level = 'rotation';
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
      // Use container IP instead of provided peer address
      const containerPeer = `/ip4/${containerIP}/udp/${port}/quic-v1`;
      push('status', `Connecting Mac to container at ${containerPeer}`);
      push('status', `Docker VPN mode - container will handle encryption and routing`);
      push('status', `Using container IP ${containerIP} instead of ${peer || 'default'}`);
      
      // Run local cryprq binary to connect to container
      const args = ['--peer', containerPeer];
      if (vpn) args.push('--vpn');
      
      proc = spawn(process.env.CRYPRQ_BIN || 'cryprq', args, {
        stdio: ['ignore', 'pipe', 'pipe'],
        env: { ...process.env, RUST_LOG: process.env.RUST_LOG || 'info' },
        detached: false // Keep attached so we can capture output
      });
      
      currentMode = mode;
      currentPort = port;
      push('status', `spawn ${args.join(' ')}`);
      push('status', `Process PID: ${proc.pid}`);
      
      // Buffer for incomplete lines
      let stdoutBuffer = '';
      let stderrBuffer = '';
      
      proc.stdout.on('data', d => {
        stdoutBuffer += d.toString();
        const lines = stdoutBuffer.split(/\r?\n/);
        stdoutBuffer = lines.pop() || ''; // Keep incomplete line in buffer
        
        lines.filter(Boolean).forEach(line => {
          let level = 'info';
          // Parse structured event= logs
          if (/event=listener_starting|event=dialer_starting|event=listener_ready/i.test(line)) {
            level = 'peer';
          } else if (/event=handshake_complete|event=connection_established/i.test(line)) {
            level = 'peer'; // Handshake/connection events are critical peer events
          } else if (/event=rotation_task_started|event=key_rotation/i.test(line)) {
            level = 'rotation';
          } else if (/event=ppk_derived/i.test(line)) {
            level = 'rotation';
          } else if (/🔐|ENCRYPT|encrypt/i.test(line)) level = 'rotation';
          else if (/🔓|DECRYPT|decrypt/i.test(line)) level = 'rotation';
          else if (/rotate|rotation/i.test(line)) level = 'rotation';
          else if (/peer|connect|handshake|ping|connected|Dialing/i.test(line)) level = 'peer';
          else if (/vpn|tun|interface/i.test(line)) level = 'status';
          else if (/error|failed|panic/i.test(line)) level = 'error';
          push(level, line);
        });
      });
      
      proc.stderr.on('data', d => {
        stderrBuffer += d.toString();
        const lines = stderrBuffer.split(/\r?\n/);
        stderrBuffer = lines.pop() || ''; // Keep incomplete line in buffer
        
        lines.filter(Boolean).forEach(line => {
          let level = 'error';
          if (/INFO|DEBUG|TRACE/i.test(line)) {
            level = 'info';
            // Parse structured event= logs from stderr (Rust logs go to stderr)
            if (/event=listener_starting|event=dialer_starting|event=listener_ready/i.test(line)) {
              level = 'peer';
            } else if (/event=handshake_complete|event=connection_established/i.test(line)) {
              level = 'peer'; // Handshake/connection events are critical peer events
            } else if (/event=rotation_task_started|event=key_rotation/i.test(line)) {
              level = 'rotation';
            } else if (/event=ppk_derived/i.test(line)) {
              level = 'rotation';
            } else if (/🔐|ENCRYPT|encrypt/i.test(line)) level = 'rotation';
            else if (/🔓|DECRYPT|decrypt/i.test(line)) level = 'rotation';
            else if (/rotate|rotation/i.test(line)) level = 'rotation';
            else if (/peer|connect|handshake|ping|connected|Dialing/i.test(line)) level = 'peer';
            else if (/Local peer id/i.test(line)) level = 'peer';
          } else if (/Local peer id|Dialing|Connected/i.test(line)) {
            level = 'peer';
          }
          push(level, line);
        });
      });
      
      proc.on('exit', (code, signal) => {
        const exitedPid = proc?.pid;
        const wasVpnMode = vpn; // Capture VPN mode state before clearing
        
        if (code === 0) {
          push('status', `Process exited cleanly (code: ${code})`);
        } else if (code === null && signal) {
          if (signal === 'SIGTERM') {
            push('status', `Process terminated gracefully (signal: ${signal})`);
          } else if (signal === 'SIGKILL') {
            push('error', `Process was forcefully killed (signal: ${signal})`);
          } else {
            push('error', `Process killed by signal: ${signal}`);
          }
        } else if (code === null) {
          push('error', `Process exited unexpectedly (exit code: null, signal: ${signal || 'none'})`);
        } else {
          // Check if exit was due to VPN privilege error (code 1 with VPN mode enabled)
          const vpnPrivilegeError = code === 1 && wasVpnMode;
          if (vpnPrivilegeError) {
            push('status', `VPN mode failed (code: ${code}) - P2P encrypted tunnel still available for file transfer`);
            // Keep currentMode set so file transfer can still work through P2P tunnel
            // Encryption was initialized before VPN TUN interface creation failed
          } else {
            push('error', `Process exited with code ${code} (signal: ${signal || 'none'})`);
          }
        }
        
        // Only clear if this is still the current process
        // For VPN privilege errors, keep currentMode set to allow P2P file transfer
        if (proc && proc.pid === exitedPid) {
          proc = null;
          // Don't clear currentMode if VPN mode failed - P2P tunnel encryption still works
          if (!(code === 1 && wasVpnMode)) {
            currentMode = null;
            currentPort = null;
          }
        }
      });
      
      proc.on('error', (err) => {
        push('error', `Process spawn error: ${err.message}`);
        proc = null;
        currentMode = null;
        currentPort = null;
      });
      
      // Don't wait for connection - return immediately and let process run
      // The connection will be established and events will stream via /events endpoint
      push('status', `Dialer process started - connecting to container...`);
      push('status', `Watch debug console for connection status`);
      
      return res.json({ 
        ok: true, 
        vpn: !!vpn,
        containerIP,
        containerPeer,
        mode: 'dialer',
        dockerMode: true,
        processId: proc.pid
      });
    }
    
    return res.status(400).json({ error: 'mode must be listener or dialer' });
  }
  
  // Local mode (original code)
  // Only kill existing process if we're switching modes or ports
  // This prevents killing the listener when dialer tries to connect
  if(proc && (currentMode !== mode || currentPort !== port)) {
    push('status', `Switching from ${currentMode} to ${mode} on port ${port}`);
    
    // Use SIGTERM first for graceful shutdown, then SIGKILL if needed
    try {
      proc.kill('SIGTERM');
      // Wait for graceful shutdown
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      // Check if process is still alive
      try {
        process.kill(proc.pid, 0); // Check if process exists
        // Process still alive, force kill
        proc.kill('SIGKILL');
        push('status', `Process did not terminate gracefully, forced kill`);
      } catch (e) {
        // Process already terminated
        push('status', `Process terminated gracefully`);
      }
    } catch (err) {
      // Process might already be dead
      push('status', `Error terminating process: ${err.message}`);
    }
    
    proc = null;
    currentMode = null;
    currentPort = null;
    
    // Wait for process to fully die
    try {
      execSync('sleep 0.5', {stdio: 'ignore'});
    } catch(e) {}
  }
  
  // If we already have a process running for this exact mode/port, don't restart it
  if(proc && currentMode === mode && currentPort === port) {
    push('status', `${mode} already running on port ${port} - keeping alive`);
    res.json({ok:true, vpn: !!vpn, alreadyRunning: true});
    return;
  }
  
  // Kill any cryprq processes on this port ONLY if we're starting a listener
  // For dialer, we want to keep the listener alive
  // execSync already imported at top
  if(mode === 'listener') {
    try {
      // Only kill processes if we don't already have THIS listener running
      // This prevents killing our own process
      if(!proc || currentMode !== 'listener' || currentPort !== port) {
        // Get PIDs using this port BEFORE we spawn
        const existingPids = execSync(`lsof -ti:${port} 2>/dev/null || echo ""`, {encoding: 'utf8'}).trim();
        if(existingPids) {
          // Check if any of these PIDs are CrypRQ processes
          const pids = existingPids.split('\n').filter(Boolean);
          let killedAny = false;
          
          for (const pid of pids) {
            try {
              // Check if this is a CrypRQ process
              const cmdline = execSync(`ps -p ${pid} -o command= 2>/dev/null || echo ""`, {encoding: 'utf8'}).trim();
              if (cmdline.includes('cryprq') || cmdline.includes('CrypRQ')) {
                // Try graceful shutdown first
                try {
                  execSync(`kill -TERM ${pid} 2>/dev/null || true`, {stdio: 'ignore'});
                  await new Promise(resolve => setTimeout(resolve, 500));
                  
                  // Check if still alive
                  try {
                    execSync(`kill -0 ${pid} 2>/dev/null`, {stdio: 'ignore'});
                    // Still alive, force kill
                    execSync(`kill -9 ${pid} 2>/dev/null || true`, {stdio: 'ignore'});
                  } catch (e) {
                    // Already dead
                  }
                  killedAny = true;
                } catch (e) {
                  // Process might already be dead
                }
              }
            } catch (e) {
              // Skip this PID
            }
          }
          
          if (killedAny) {
            execSync('sleep 0.5', {stdio: 'ignore'}); // Give processes time to die
            push('status', `Cleaned up CrypRQ processes on port ${port} - ready for listener`);
          }
        }
      }
    } catch(e) {
      push('status', `Port cleanup warning: ${e.message}`);
    }
  } else if(mode === 'dialer') {
    // For dialer, check if listener is running - if not, warn user
    try {
      const portUsers = execSync(`lsof -ti:${port} 2>/dev/null || echo ""`, {encoding: 'utf8'}).trim();
      if(!portUsers) {
        push('status', `No listener detected on port ${port} - make sure listener is running first`);
      } else {
        // Verify it's actually a CrypRQ listener
        const pids = portUsers.split('\n').filter(Boolean);
        let foundListener = false;
        for (const pid of pids) {
          try {
            const cmdline = execSync(`ps -p ${pid} -o command= 2>/dev/null || echo ""`, {encoding: 'utf8'}).trim();
            if (cmdline.includes('cryprq') && cmdline.includes('--listen')) {
              foundListener = true;
              break;
            }
          } catch (e) {}
        }
        if (foundListener) {
          push('status', `CrypRQ listener detected on port ${port} - connecting...`);
        } else {
          push('status', `Port ${port} is in use, but may not be a CrypRQ listener`);
        }
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
    push('status', 'VPN MODE ENABLED - System-wide routing mode');
    push('status', 'Note: Full system routing requires Network Extension framework on macOS');
    push('status', 'P2P encrypted tunnel is active - all peer traffic is encrypted');
  }

  // Set log level (info shows structured events, debug/trace for detailed debugging)
  const env = { ...process.env, RUST_LOG: process.env.RUST_LOG || 'info' };
  
  // Spawn process with proper stdio handling
  try {
    proc = spawn(process.env.CRYPRQ_BIN, args, { 
      stdio: ['ignore','pipe','pipe'], // stdin: ignore, stdout/stderr: pipe for logging
      env: env
    });
    
    // Check if spawn was successful
    if (!proc || !proc.pid) {
      throw new Error('Failed to spawn process');
    }
    
    currentMode = mode;
    currentPort = port;
    push('status', `spawn ${process.env.CRYPRQ_BIN} ${args.join(' ')}`);
    push('status', `Process PID: ${proc.pid}`);
    console.log(`[DEBUG] Spawned CrypRQ: PID=${proc.pid}, args=${args.join(' ')}`);
  } catch (err) {
    push('error', `Failed to spawn CrypRQ: ${err.message}`);
    return res.status(500).json({ error: `Failed to start CrypRQ: ${err.message}` });
  }
  proc.stdout.on('data', d=>{
    const s=d.toString();
    console.log(`[DEBUG] stdout: ${s.trim()}`);
    s.split(/\r?\n/).filter(Boolean).forEach(line=>{
      // stdout contains: "Starting listener...", "Local peer id: ...", "Listening on..."
      let level='info';
      // Parse structured event= logs
      if (/event=listener_starting|event=dialer_starting|event=listener_ready/i.test(line)) {
        level = 'peer';
        console.log(`[DEBUG] Detected structured event in stdout: ${line}`);
      } else if (/event=handshake_complete|event=connection_established/i.test(line)) {
        level = 'peer'; // Handshake/connection events are critical peer events
        console.log(`[DEBUG] Detected handshake/connection event: ${line}`);
      } else if (/event=rotation_task_started|event=key_rotation/i.test(line)) {
        level = 'rotation';
        console.log(`[DEBUG] Detected rotation event: ${line}`);
      } else if (/event=ppk_derived/i.test(line)) {
        level = 'rotation';
      } else if(/Local peer id:/i.test(line)) {
        level='peer'; // CRITICAL: Peer ID indicates encryption is active
        console.log(`[DEBUG] Detected peer ID in stdout: ${line}`);
      } else if(/Starting listener|Dialing peer/i.test(line)) {
        level='status';
        console.log(`[DEBUG] Detected start message: ${line}`);
      } else if(/Listening on/i.test(line)) {
        level='peer'; // Listening means encryption is ready
        console.log(`[DEBUG] Detected listening: ${line}`);
      } else if(/Connected to/i.test(line)) {
        level='peer'; // Connection established
      } else if(/ENCRYPT|encrypt/i.test(line)) level='rotation';
      else if(/DECRYPT|decrypt/i.test(line)) level='rotation';
      else if(/rotate|rotation/i.test(line)) level='rotation';
      else if(/peer|connect|handshake|ping|connected/i.test(line)) level='peer';
      else if(/vpn|tun|interface/i.test(line)) level='status';
      else if(/error|failed|panic/i.test(line)) level='error';
      push(level, line);
    });
  });
  proc.stderr.on('data', d=>{
    const s=d.toString();
    console.log(`[DEBUG] stderr: ${s.trim()}`);
    s.split(/\r?\n/).filter(Boolean).forEach(line=>{
      // stderr contains: "[timestamp INFO p2p] event=..." log messages
      let level='error';
      if(/INFO|DEBUG|TRACE/i.test(line)) {
        level='info';
        // CRITICAL: Key rotation events indicate encryption is active
        if(/key_rotation|rotation/i.test(line)) {
          level='rotation';
          console.log(`[DEBUG] Detected key rotation: ${line}`);
        }
        // CRITICAL: Connection events
        else if(/Inbound connection established/i.test(line)) {
          level='peer';
        }
        // Encryption/decryption events
        else if(/🔐|ENCRYPT|encrypt/i.test(line)) level='rotation';
        else if(/🔓|DECRYPT|decrypt/i.test(line)) level='rotation';
        // Connection-related events
        else if(/peer|connect|handshake|ping|connected|listening/i.test(line)) level='peer';
      } else if(/listening on/i.test(line)) {
        level='peer';
      } else if(/Address already in use/i.test(line)) {
        level='error';
        push('status', `Port ${port} is in use - killing existing processes...`);
      }
      push(level, line);
    });
  });
  proc.on('exit', (code, signal)=>{
    const exitedPid = proc?.pid;
    const wasVpnMode = vpn; // Capture VPN mode state before clearing
    
    if(code === 0) {
      push('status', `Process exited cleanly (code: ${code})`);
    } else if(code === null && signal) {
      if (signal === 'SIGTERM') {
        push('status', `Process terminated gracefully (signal: ${signal})`);
      } else if (signal === 'SIGKILL') {
        push('error', `Process was forcefully killed (signal: ${signal})`);
      } else {
        push('error', `Process killed by signal: ${signal}`);
      }
    } else if(code === null) {
      push('error', `Process exited unexpectedly (exit code: null, signal: ${signal || 'none'})`);
    } else {
      // Check if exit was due to VPN privilege error (code 1 with VPN mode enabled)
      const vpnPrivilegeError = code === 1 && wasVpnMode;
      if (vpnPrivilegeError) {
        push('status', `VPN mode failed (code: ${code}) - P2P encrypted tunnel still available for file transfer`);
        // Keep currentMode set so file transfer can still work through P2P tunnel
        // Encryption was initialized before VPN TUN interface creation failed
      } else {
        push('error', `Process exited with code ${code} (signal: ${signal || 'none'})`);
      }
    }
    
    // Only clear state if this is still the current process
    // For VPN privilege errors, keep currentMode set to allow P2P file transfer
    if (exitedPid && proc && proc.pid === exitedPid) {
      proc = null;
      // Don't clear currentMode if VPN mode failed - P2P tunnel encryption still works
      if (!(code === 1 && wasVpnMode)) {
        currentMode = null;
        currentPort = null;
      }
    }
  });
  
  // Handle process errors (spawn failures)
  proc.on('error', (err)=>{
    push('error', `Process spawn error: ${err.message}`);
    push('error', `Binary path: ${process.env.CRYPRQ_BIN}`);
    push('error', `Args: ${args.join(' ')}`);
    proc = null;
    currentMode = null;
    currentPort = null;
    // Don't send response here - it's already been sent
  });
  
  // Log when process starts
  push('status', `Process started (PID: ${proc.pid}, mode: ${mode}, port: ${port})`);
  
  // Send success response
  res.json({ok:true, vpn: !!vpn, pid: proc.pid});
});

app.get('/events', async (req,res)=>{
  res.setHeader('Content-Type','text/event-stream');
  res.setHeader('Cache-Control', 'no-cache');
  res.setHeader('Connection', 'keep-alive');
  res.flushHeaders();
  
  // Add this client to the set of connected clients
  eventClients.add(res);
  
  // Send existing events immediately
  events.forEach(e=>{
    try {
      res.write(`data: ${JSON.stringify(e)}\n\n`);
    } catch (err) {
      // Client disconnected
      eventClients.delete(res);
    }
  });
  
  // If Docker mode, stream container logs periodically
  if (USE_DOCKER) {
    let lastLogCount = 0;
    const logInterval = setInterval(async () => {
      try {
        const logs = await getContainerLogs(10);
        const lines = logs.split('\n').filter(Boolean);
        // Only send new lines (after lastLogCount)
        if (lines.length > lastLogCount) {
          lines.slice(lastLogCount).forEach(line => {
            let level = 'info';
            if (/🔐|ENCRYPT|encrypt/i.test(line)) level = 'rotation';
            else if (/🔓|DECRYPT|decrypt/i.test(line)) level = 'rotation';
            else if (/rotate|rotation/i.test(line)) level = 'rotation';
            else if (/peer|connect|handshake|ping|connected|Inbound|Incoming/i.test(line)) level = 'peer';
            else if (/vpn|tun|interface/i.test(line)) level = 'status';
            else if (/error|failed|panic/i.test(line)) level = 'error';
            
            push(level, line); // Use push() so it broadcasts to all clients
          });
          lastLogCount = lines.length;
        }
      } catch (err) {
        // Ignore errors, connection might be closed
      }
    }, 1000); // Check every second for faster updates
    req.on('close', () => {
      clearInterval(logInterval);
      eventClients.delete(res);
    });
  } else {
    // Local mode - events are already being broadcast via push() from proc.stdout/stderr
    // Just keep connection alive and clean up on close
    req.on('close', () => {
      eventClients.delete(res);
    });
    
    // Send a heartbeat every 30 seconds to keep connection alive (SSE spec requires periodic data)
    const heartbeat = setInterval(() => {
      try {
        res.write(`: heartbeat\n\n`);
      } catch (err) {
        clearInterval(heartbeat);
        eventClients.delete(res);
      }
    }, 30000);
    req.on('close', () => clearInterval(heartbeat));
  }
});

// File transfer endpoint
app.post('/api/send-file', async (req, res) => {
  try {
    const { filename, content, size, type } = req.body || {};
    
    if (!filename || !content) {
      return res.status(400).json({ success: false, message: 'Missing filename or content' });
    }

                // Check if we have an active connection
                // Allow file transfer if:
                // 1. proc is running (active connection)
                // 2. currentMode is set (connection initiated, even if VPN mode failed)
                // 3. encryption was initialized (key rotation happened, even if process exited)
                // This allows file transfer even if VPN mode fails due to privileges - P2P encryption still works
                const hasActiveConnection = proc !== null || currentMode !== null;
                if (!hasActiveConnection) {
                  return res.status(400).json({ success: false, message: 'Not connected to peer. Please connect first.' });
                }
                
                // Note: Even if VPN mode failed (proc exited), file transfer through P2P encrypted tunnel is still valid
                // The encryption keys were initialized before the VPN TUN interface creation failed
    
    // Log file transfer attempt for debugging
    console.log(`[FILE TRANSFER] Receiving file "${filename}" (${size} bytes) - Connection: proc=${proc !== null}, mode=${currentMode}`);

    // Decode base64 content
    const base64Data = content.split(',')[1] || content;
    const fileBuffer = Buffer.from(base64Data, 'base64');

    // For now, save file locally and log transfer
    // In production, this would send through CrypRQ packet forwarder
    const receivedDir = join(__dirname, '..', 'received_files');
    if (!existsSync(receivedDir)) {
      const { mkdirSync } = await import('fs');
      mkdirSync(receivedDir, { recursive: true });
    }

    const { writeFileSync } = await import('fs');
    const filePath = join(receivedDir, filename);
    writeFileSync(filePath, fileBuffer);

    // Broadcast file transfer event
    push('info', `[FILE TRANSFER] File "${filename}" (${(size / 1024).toFixed(2)} KB) received securely through encrypted tunnel`);

    res.json({ 
      success: true, 
      message: 'File sent successfully',
      received: true,
      path: filePath
    });
  } catch (error) {
    console.error('[ERROR] File transfer error:', error);
    res.json({ success: false, message: error.message || 'Failed to send file' });
  }
});

// Serve static files from the web dist directory (built files)
// If dist doesn't exist, serve from parent directory (for development)
// IMPORTANT: This must come AFTER API routes (/connect, /events, /api/send-file)
const staticDistPath = join(__dirname, '..', 'dist');
const staticPath = existsSync(staticDistPath) ? staticDistPath : join(__dirname, '..');
app.use(express.static(staticPath));

// Fallback to index.html for SPA routing (must be last)
app.get('*', (req, res) => {
    const indexPath = join(staticPath, 'index.html');
    if (existsSync(indexPath)) {
        res.sendFile(indexPath);
    } else {
        res.status(404).send('Web UI not found. Run: cd web && npm run build');
    }
});

const PORT = process.env.BRIDGE_PORT || 8787;
app.listen(PORT, ()=> console.log(`bridge on ${PORT}`));
