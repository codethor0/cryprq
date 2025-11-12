// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

// Docker bridge server - connects web UI to CrypRQ container
// This replaces the local binary execution with container communication

import express from 'express';
import cors from 'cors';
import { exec } from 'node:child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);
const app = express();
app.use(cors());
app.use(express.json());

const CONTAINER_NAME = process.env.CRYPRQ_CONTAINER || 'cryprq-vpn';
const CONTAINER_IP = process.env.CRYPRQ_CONTAINER_IP || null;

const events = [];
function push(level, t) {
    const e = { level, t };
    events.push(e);
    if (events.length > 500) events.shift();
}

// Get container IP address
async function getContainerIP() {
    if (CONTAINER_IP) return CONTAINER_IP;
    
    try {
        const { stdout } = await execAsync(
            `docker inspect -f '{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' ${CONTAINER_NAME}`
        );
        return stdout.trim();
    } catch (err) {
        push('error', `Failed to get container IP: ${err.message}`);
        return null;
    }
}

// Check if container is running
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

// Get container logs
async function getContainerLogs(lines = 50) {
    try {
        const { stdout } = await execAsync(
            `docker logs --tail ${lines} ${CONTAINER_NAME} 2>&1`
        );
        return stdout;
    } catch (err) {
        return `Error getting logs: ${err.message}`;
    }
}

// Start container if not running
async function ensureContainerRunning() {
    const running = await isContainerRunning();
    if (!running) {
        push('status', `Starting container ${CONTAINER_NAME}...`);
        try {
            await execAsync(`docker-compose -f docker-compose.vpn.yml up -d`);
            await new Promise(resolve => setTimeout(resolve, 3000)); // Wait for container to start
            push('status', `Container ${CONTAINER_NAME} started`);
        } catch (err) {
            push('error', `Failed to start container: ${err.message}`);
            throw err;
        }
    }
}

// Connect endpoint - connects Mac to container
app.post('/connect', async (req, res) => {
    const { mode, port, peer, vpn } = req.body || {};
    
    try {
        // Ensure container is running
        await ensureContainerRunning();
        
        const containerIP = await getContainerIP();
        if (!containerIP) {
            return res.status(500).json({ error: 'Could not get container IP' });
        }
        
        // For listener mode, container is already listening
        if (mode === 'listener') {
            push('status', `Container ${CONTAINER_NAME} is listening on port ${port}`);
            push('status', `Container IP: ${containerIP}`);
            push('status', `Connect to: /ip4/${containerIP}/udp/${port}/quic-v1`);
            
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
            const { spawn } = require('child_process');
            const args = ['--peer', containerPeer];
            if (vpn) args.push('--vpn');
            
            const proc = spawn(process.env.CRYPRQ_BIN || 'cryprq', args, {
                stdio: ['ignore', 'pipe', 'pipe'],
                env: { ...process.env, RUST_LOG: 'debug' }
            });
            
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
                } else {
                    push('error', `exit ${code} (signal: ${signal || 'none'})`);
                }
            });
            
            return res.json({ 
                ok: true, 
                vpn: !!vpn,
                containerIP,
                containerPeer,
                mode: 'dialer'
            });
        }
        
        return res.status(400).json({ error: 'mode must be listener or dialer' });
    } catch (err) {
        push('error', `Connect error: ${err.message}`);
        return res.status(500).json({ error: err.message });
    }
});

// Events endpoint - streams container logs
app.get('/events', (req, res) => {
    res.setHeader('Content-Type', 'text/event-stream');
    res.setHeader('Cache-Control', 'no-cache');
    res.flushHeaders();
    
    // Send existing events
    events.forEach(e => res.write(`data: ${JSON.stringify(e)}\n\n`));
    
    // Stream container logs
    const logInterval = setInterval(async () => {
        try {
            const logs = await getContainerLogs(5);
            logs.split('\n').filter(Boolean).forEach(line => {
                let level = 'info';
                if (/🔐|ENCRYPT|encrypt/i.test(line)) level = 'rotation';
                else if (/🔓|DECRYPT|decrypt/i.test(line)) level = 'rotation';
                else if (/rotate|rotation/i.test(line)) level = 'rotation';
                else if (/peer|connect|handshake|ping|connected/i.test(line)) level = 'peer';
                else if (/vpn|tun|interface/i.test(line)) level = 'status';
                else if (/error|failed|panic/i.test(line)) level = 'error';
                
                const e = { level, t: line };
                events.push(e);
                if (events.length > 500) events.shift();
                res.write(`data: ${JSON.stringify(e)}\n\n`);
            });
        } catch (err) {
            // Ignore errors
        }
    }, 2000);
    
    req.on('close', () => clearInterval(logInterval));
});

// Container status endpoint
app.get('/status', async (req, res) => {
    try {
        const running = await isContainerRunning();
        const containerIP = running ? await getContainerIP() : null;
        
        res.json({
            running,
            containerName: CONTAINER_NAME,
            containerIP,
            connectAddress: containerIP ? `/ip4/${containerIP}/udp/9999/quic-v1` : null
        });
    } catch (err) {
        res.status(500).json({ error: err.message });
    }
});

const PORT = process.env.BRIDGE_PORT || 8787;
const server = app.listen(PORT, () => {
    console.log(`Docker bridge server on ${PORT}`);
    console.log(`Container: ${CONTAINER_NAME}`);
});

// Export app for use in server.mjs
export default app;

