// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import http from 'node:http';
import { spawn } from 'node:child_process';
import { createHash, randomBytes, randomUUID, timingSafeEqual } from 'node:crypto';
import { constants } from 'node:fs';
import * as fs from 'node:fs/promises';
import { homedir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const keyPattern = /^ed25519:[a-f0-9]{64}$/;
const idPattern = /^[a-f0-9-]{36}$/;
const MAX_NATIVE_BYTES = 1 << 30;
const jsonLimit = 8192;
const maximumFiles = 200;

class HttpError extends Error {
  constructor(status, message) { super(message); this.status = status; }
}

export function safeFilename(name) {
  return typeof name === 'string' && Buffer.byteLength(name) <= 180 && name.length > 0 &&
    !name.startsWith('.') && !/[<>:"/\\|?*\x00-\x1f\x7f]/u.test(name) &&
    !/[. ]$/u.test(name) && !/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(name) &&
    name === name.normalize('NFC');
}

function address(value) {
  const match = /^\/ip4\/(\d{1,3}(?:\.\d{1,3}){3})\/udp\/(\d{1,5})$/.exec(value || '');
  if (!match || match[1].split('.').some(part => Number(part) > 255) || Number(match[2]) < 1 || Number(match[2]) > 65535) {
    throw new HttpError(400, 'Use an IPv4 UDP address such as /ip4/127.0.0.1/udp/20440.');
  }
  return value;
}

async function exists(filename) {
  try { return await fs.lstat(filename); } catch (error) { if (error.code === 'ENOENT') return null; throw error; }
}

// Reject links at every component before any state is opened or created.
async function noLinks(filename) {
  let cursor = path.parse(filename).root;
  for (const component of path.resolve(filename).slice(cursor.length).split(path.sep)) {
    cursor = path.join(cursor, component);
    const stat = await exists(cursor);
    if (stat?.isSymbolicLink()) throw new Error('State paths must not contain symbolic links.');
  }
}

async function privateDirectory(directory) {
  await noLinks(directory);
  await fs.mkdir(directory, { recursive: true, mode: 0o700 });
  const stat = await fs.lstat(directory);
  if (!stat.isDirectory() || stat.isSymbolicLink()) throw new Error('Invalid state directory.');
  if (typeof process.getuid === 'function' && stat.uid !== process.getuid()) throw new Error('State directory must belong to the current user.');
  await fs.chmod(directory, 0o700);
}

async function regularFile(filename) {
  await noLinks(filename);
  const handle = await fs.open(filename, constants.O_RDONLY | constants.O_NOFOLLOW);
  try {
    const stat = await handle.stat();
    if (!stat.isFile()) throw new Error('Expected a regular file.');
    return { handle, stat };
  } catch (error) { await handle.close(); throw error; }
}

async function digestFile(filename, maxBytes) {
  const { handle, stat } = await regularFile(filename);
  try {
    if (stat.size > maxBytes) throw new Error('File exceeds the portal size limit.');
    const hash = createHash('sha256');
    let count = 0;
    for await (const chunk of handle.createReadStream({ autoClose: false })) {
      count += chunk.length;
      if (count > maxBytes) throw new Error('File exceeds the portal size limit.');
      hash.update(chunk);
    }
    const after = await handle.stat();
    if (count !== stat.size || after.size !== stat.size || after.mtimeMs !== stat.mtimeMs) throw new Error('File changed during verification.');
    return { size: count, sha256: hash.digest('hex') };
  } finally { await handle.close(); }
}

async function atomicJSON(filename, value) {
  await noLinks(filename);
  const temp = `${filename}.${randomUUID()}.tmp`;
  try {
    await fs.writeFile(temp, `${JSON.stringify(value, null, 2)}\n`, { flag: 'wx', mode: 0o600 });
    await fs.rename(temp, filename);
  } finally { await fs.rm(temp, { force: true }); }
}

function lines(stream, callback) {
  let pending = '';
  stream.setEncoding('utf8');
  stream.on('data', chunk => {
    pending = (pending + chunk).slice(-16384);
    const parts = pending.split('\n');
    pending = parts.pop();
    for (const line of parts) callback(line);
  });
}

function secureEqual(value, expected) {
  if (typeof value !== 'string') return false;
  const actual = Buffer.from(value);
  const wanted = Buffer.from(expected);
  return actual.length === wanted.length && timingSafeEqual(actual, wanted);
}

async function readJSON(request) {
  if (!(request.headers['content-type'] || '').startsWith('application/json')) throw new HttpError(415, 'Expected application/json.');
  let size = 0;
  const chunks = [];
  for await (const chunk of request) {
    size += chunk.length;
    if (size > jsonLimit) throw new HttpError(413, 'Request is too large.');
    chunks.push(chunk);
  }
  try { return JSON.parse(Buffer.concat(chunks).toString('utf8')); } catch { throw new HttpError(400, 'Invalid JSON.'); }
}

export async function createPortal(options = {}) {
  const env = options.env || process.env;
  const host = env.HOST || '127.0.0.1';
  if (!['127.0.0.1', 'localhost', '::1', '0.0.0.0'].includes(host)) throw new Error('Unsupported portal host.');
  // 0.0.0.0 is an explicit container-only opt in. Docker must publish on loopback.
  if (host === '0.0.0.0' && env.CRYPRQ_CONTAINER !== '1') throw new Error('Non-loopback binding requires CRYPRQ_CONTAINER=1 and loopback-only Docker port publication.');
  const port = Number(env.PORT || 8787);
  const maxBytes = Number(env.CRYPRQ_MAX_FILE_BYTES || 64 * 1024 * 1024);
  const idleMs = Number(env.CRYPRQ_RECEIVER_IDLE_MS || 15000);
  if (!Number.isSafeInteger(port) || port < 1 || port > 65535 || !Number.isSafeInteger(maxBytes) || maxBytes < 1 || maxBytes > MAX_NATIVE_BYTES || !Number.isSafeInteger(idleMs) || idleMs < 1000 || idleMs > 120000) throw new Error('Invalid portal limit or port.');
  const listen = address(env.CRYPRQ_LISTEN_ADDR || '/ip4/127.0.0.1/udp/20440');
  const advertise = address(env.CRYPRQ_ADVERTISE_ADDR || listen);
  const stateDir = path.resolve(env.CRYPRQ_STATE_DIR || path.join(homedir(), '.cryprq-portal'));
  const binary = env.CRYPRQ_BIN || 'cryprq';
  const inbox = path.join(stateDir, 'inbox');
  const uploads = path.join(stateDir, 'uploads');
  const staging = path.join(stateDir, 'receiving');
  const identityPath = path.join(stateDir, 'identity.key');
  const trustPath = path.join(stateDir, 'trusted_peers');
  const peersPath = path.join(stateDir, 'peers.json');
  const indexPath = path.join(stateDir, 'files.json');
  const lockPath = path.join(stateDir, 'portal.lock');
  const token = randomBytes(32).toString('hex');
  const peers = [];
  const files = [];
  const transfers = [];
  let receiver = { status: 'needs-pairing' };
  let receiverChild = null;
  let receiverDirectory = null;
  let receiverLastActivity = 0;
  let receiverBytes = 0;
  let restarting = null;
  let scanning = false;
  let scanTask = null;
  const operations = new Set();
  const activeUploads = new Set();
  let senderDone = null;
  let peerMutation = false;
  let peerQueue = Promise.resolve();
  let closing = false;
  let busy = false;
  let sender = null;
  let server;
  let timer;

  for (const directory of [stateDir, inbox, uploads, staging]) await privateDirectory(directory);
  // One adapter owns each identity, trust store and receiver directory.
  const oldLock = await exists(lockPath);
  if (oldLock) {
    const { handle } = await regularFile(lockPath);
    let pid;
    try { pid = Number((await handle.readFile('utf8')).trim()); } finally { await handle.close(); }
    if (!Number.isInteger(pid) || pid <= 0) throw new Error('Invalid portal lock; inspect the state directory.');
    let alive = true;
    try { process.kill(pid, 0); } catch (error) { if (error.code === 'ESRCH') alive = false; }
    if (alive) throw new Error('Another portal already owns this state directory.');
    await fs.unlink(lockPath);
  }
  await fs.writeFile(lockPath, `${process.pid}\n`, { mode: 0o600, flag: 'wx' });

  const prefix = ['--identity', identityPath, '--trusted-peers', trustPath];
  function child(args) {
    return spawn(binary, [...prefix, ...args], {
      env: { ...process.env, RUST_LOG: 'info', NO_COLOR: '1' },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
  }
  async function run(args) {
    const proc = child(args);
    let output = '';
    proc.stdout.setEncoding('utf8');
    proc.stdout.on('data', chunk => { output = (output + chunk).slice(-4096); });
    proc.stderr.resume();
    const timeout = setTimeout(() => proc.kill('SIGKILL'), 15000);
    try {
      const code = await new Promise((resolve, reject) => { proc.once('error', reject); proc.once('exit', resolve); });
      if (code !== 0) throw new Error('The authenticated CLI could not load or create this device identity.');
      return output.trim();
    } finally { clearTimeout(timeout); }
  }

  async function terminate(proc) {
    if (!proc || !proc.pid || proc.exitCode !== null || proc.signalCode !== null) return;
    const done = new Promise(resolve => { proc.once('exit', resolve); proc.once('error', resolve); });
    proc.kill('SIGINT');
    const force = setTimeout(() => proc.kill('SIGKILL'), 2500);
    await done;
    clearTimeout(force);
  }

  // Reuse the CLI's canonical Ed25519 parser, including weak-key rejection.
  // A short loopback listener on an ephemeral port validates the candidate trust
  // store without replacing the active receiver or implementing key validation.
  async function validatePeerKey(candidate) {
    const directory = await fs.mkdtemp(path.join(staging, 'validate-'));
    const trust = path.join(directory, 'trusted_peers');
    await fs.writeFile(trust, `${candidate}\n`, { mode: 0o600, flag: 'wx' });
    const proc = spawn(binary, ['--identity', identityPath, '--trusted-peers', trust, 'receive-file', '--listen', '/ip4/127.0.0.1/udp/0', '--output-dir', directory], { env: { ...process.env, RUST_LOG: 'info', NO_COLOR: '1' }, stdio: ['ignore', 'pipe', 'pipe'] });
    let timeout;
    try {
      await new Promise((resolve, reject) => {
        const inspect = line => { if (/event=listening addr=/.test(line)) resolve(); };
        lines(proc.stdout, inspect);
        lines(proc.stderr, inspect);
        proc.once('error', reject);
        proc.once('exit', () => reject(new HttpError(400, 'The CLI rejected this public identity.')));
        timeout = setTimeout(() => reject(new HttpError(400, 'The CLI could not validate this public identity.')), 5000);
      });
    } finally {
      clearTimeout(timeout);
      await terminate(proc);
      await fs.rm(directory, { recursive: true, force: true });
    }
  }

  async function persistPeers() {
    await atomicJSON(peersPath, peers);
    await noLinks(trustPath);
    const temp = `${trustPath}.${randomUUID()}.tmp`;
    try {
      await fs.writeFile(temp, peers.map(peer => peer.key).join('\n') + (peers.length ? '\n' : ''), { flag: 'wx', mode: 0o600 });
      await fs.rename(temp, trustPath);
    } finally { await fs.rm(temp, { force: true }); }
  }

  // Trust-store changes run one at a time, in arrival order. Each waits for
  // the storage scan and any receiver restart in flight, so that it decides on
  // settled state and the receiver it starts reads the store it just wrote.
  function mutatePeers(task) {
    const result = peerQueue.then(async () => {
      if (closing) throw new HttpError(503, 'The portal is shutting down.');
      peerMutation = true;
      try {
        if (scanTask) await scanTask;
        while (restarting) await restarting.catch(() => {});
        return await task();
      } finally { peerMutation = false; }
    });
    peerQueue = result.catch(() => {});
    return result;
  }

  // Every caller gets a receiver started after its call: a restart already in
  // flight may have read an older trust store, so it is awaited, not reused.
  async function restartReceiver() {
    while (restarting) await restarting.catch(() => {});
    restarting = (async () => {
      const previous = receiverChild;
      receiverChild = null;
      await terminate(previous);
      if (receiverDirectory) {
        await noLinks(receiverDirectory);
        await fs.rm(receiverDirectory, { recursive: true, force: true });
        receiverDirectory = null;
      }
      receiver = { status: peers.length ? 'starting' : 'needs-pairing' };
      if (closing || !peers.length) return;
      receiverDirectory = await fs.mkdtemp(path.join(staging, 'session-'));
      await fs.chmod(receiverDirectory, 0o700);
      const currentDirectory = receiverDirectory;
      const proc = child(['receive-file', '--listen', listen, '--output-dir', currentDirectory]);
      receiverChild = proc;
      receiverBytes = 0;
      receiverLastActivity = Date.now();
      const statusLine = line => {
        if (receiverChild !== proc) return;
        if (/event=listening addr=/.test(line)) receiver = { status: 'listening' };
        const match = /event=tunnel_established role=responder peer=(ed25519:[a-f0-9]{64}) session_id=[a-f0-9]+$/.exec(line);
        if (match && peers.some(peer => peer.key === match[1])) {
          receiver = { status: 'authenticated', authenticatedPeer: match[1] };
          receiverLastActivity = Date.now();
        }
      };
      lines(proc.stdout, statusLine);
      lines(proc.stderr, statusLine);
      proc.once('error', () => { if (receiverChild === proc) receiver = { status: 'error', error: 'Could not start the authenticated receiver.' }; });
      proc.once('exit', () => {
        if (receiverChild === proc && !closing) {
          receiver = { status: 'error', error: 'Receiver stopped. Restarting.' };
          // Only replace this stopped receiver. One started since then stays.
          setTimeout(() => { if (!closing && receiverChild === proc) restartReceiver().catch(() => { receiver = { status: 'error', error: 'Receiver restart failed.' }; }); }, 500);
        }
      });
    })();
    try { await restarting; } finally { restarting = null; }
  }

  async function scanReceiver() {
    if (scanning || restarting || peerMutation || !receiverDirectory || !receiverChild) return;
    scanning = true;
    try {
      const names = await fs.readdir(receiverDirectory);
      let size = 0;
      const inodes = new Set();
      const completed = [];
      for (const name of names) {
        const filename = path.join(receiverDirectory, name);
        const stat = await exists(filename);
        if (!stat) continue;
        if (!stat.isFile() || stat.isSymbolicLink()) throw new Error('Receiver storage contained an unsafe file.');
        const inode = `${stat.dev}:${stat.ino}`;
        if (!inodes.has(inode)) size += stat.size;
        inodes.add(inode);
        if (!name.startsWith('.cryprq-')) completed.push({ name, filename, stat });
      }
      if (size > maxBytes || files.length + completed.length > maximumFiles) throw new Error('Receiver storage limit reached.');
      if (size !== receiverBytes) { receiverBytes = size; receiverLastActivity = Date.now(); }
      if (receiver.status === 'authenticated') {
        for (const item of completed) {
          if (!safeFilename(item.name)) throw new Error('Rejected unsafe incoming filename.');
          const verified = await digestFile(item.filename, maxBytes);
          const id = randomUUID();
          await fs.rename(item.filename, path.join(inbox, id));
          await fs.chmod(path.join(inbox, id), 0o600);
          files.unshift({ id, name: item.name, ...verified, verified: true, receivedAt: new Date().toISOString() });
        }
        if (completed.length) {
          await atomicJSON(indexPath, files);
          await restartReceiver();
        } else if (Date.now() - receiverLastActivity > idleMs) await restartReceiver();
      }
    } catch {
      await restartReceiver();
    } finally { scanning = false; }
  }

  try {
    await noLinks(identityPath);
    const stat = await exists(identityPath);
    if (stat && !stat.isFile()) throw new Error('Identity must be a regular private file.');
    const key = await run(['identity', stat ? 'show' : 'init']);
    if (!keyPattern.test(key)) throw new Error('CLI returned an invalid public identity.');
    await fs.chmod(identityPath, 0o600);
    if (await exists(peersPath)) {
      const { handle } = await regularFile(peersPath);
      let stored;
      try { stored = JSON.parse(await handle.readFile('utf8')); } finally { await handle.close(); }
      if (!Array.isArray(stored) || stored.length > 16) throw new Error('Invalid peer store.');
      for (const peer of stored) {
        if (!idPattern.test(peer.id) || !keyPattern.test(peer.key) || typeof peer.name !== 'string' || peer.name.length > 64 || peer.key === key) throw new Error('Invalid pinned peer.');
        address(peer.address);
        peers.push(peer);
      }
    }
    if (await exists(indexPath)) {
      const { handle } = await regularFile(indexPath);
      let stored;
      try { stored = JSON.parse(await handle.readFile('utf8')); } finally { await handle.close(); }
      if (!Array.isArray(stored) || stored.length > maximumFiles) throw new Error('Invalid inbox index.');
      for (const entry of stored) {
        if (!idPattern.test(entry.id) || !safeFilename(entry.name) || !/^[a-f0-9]{64}$/.test(entry.sha256)) throw new Error('Invalid inbox entry.');
        const verified = await digestFile(path.join(inbox, entry.id), maxBytes);
        if (verified.sha256 !== entry.sha256 || verified.size !== entry.size || entry.verified !== true) throw new Error('Inbox verification failed.');
        files.push(entry);
      }
    }
    // Only generated temporary content is removed. Published inbox files remain.
    for (const directory of [uploads, staging]) {
      for (const name of await fs.readdir(directory)) {
        const filename = path.join(directory, name);
        await noLinks(filename);
        await fs.rm(filename, { recursive: true });
      }
    }
    await persistPeers();
    await restartReceiver();
    timer = setInterval(() => {
      if (!scanTask) scanTask = scanReceiver().catch(() => { receiver = { status: 'error', error: 'Receiver storage could not be checked.' }; }).finally(() => { scanTask = null; });
    }, 150);
    timer.unref();

    const pairing = { version: 1, key, address: advertise };
    function snapshot() {
      return { identity: { key, address: advertise, pairing }, receiver, peers, transfers, files, limits: { maxFileBytes: maxBytes } };
    }
    function json(response, status, body) {
      response.writeHead(status, { 'Content-Type': 'application/json; charset=utf-8', 'Cache-Control': 'no-store' });
      response.end(JSON.stringify(body));
    }
    function originGuard(request) {
      const hostname = request.headers.host;
      const allowed = [`127.0.0.1:${port}`, `localhost:${port}`, `[::1]:${port}`];
      if (!allowed.includes(hostname)) throw new HttpError(403, 'Unrecognized portal host.');
      if (request.headers.origin && request.headers.origin !== `http://${hostname}`) throw new HttpError(403, 'Cross-origin requests are not allowed.');
      if (request.headers['sec-fetch-site'] && !['same-origin', 'none'].includes(request.headers['sec-fetch-site'])) throw new HttpError(403, 'Cross-site requests are not allowed.');
    }
    async function upload(request, response, url) {
      if (closing) throw new HttpError(503, 'The portal is shutting down.');
      if (busy || peerMutation) throw new HttpError(409, 'A transfer or pairing change is already active.');
      const peer = peers.find(item => item.id === url.searchParams.get('peer'));
      if (!peer) throw new HttpError(400, 'Choose an explicitly pinned peer.');
      if (request.headers['content-type'] !== 'application/octet-stream') throw new HttpError(415, 'Expected a binary file upload.');
      let name;
      try { name = decodeURIComponent(request.headers['x-filename'] || ''); } catch { throw new HttpError(400, 'Invalid filename.'); }
      if (!safeFilename(name)) throw new HttpError(400, 'Use a safe filename without path components or control characters.');
      const length = request.headers['content-length'];
      if (length !== undefined && (!/^\d+$/.test(length) || Number(length) > maxBytes)) throw new HttpError(413, 'File exceeds the portal size limit.');
      busy = true;
      activeUploads.add(request);
      let directory;
      try {
        directory = await fs.mkdtemp(path.join(uploads, 'send-'));
        const filename = path.join(directory, name);
        const handle = await fs.open(filename, 'wx', 0o600);
        let size = 0;
        const hash = createHash('sha256');
        try {
          for await (const chunk of request) {
            size += chunk.length;
            if (size > maxBytes) throw new HttpError(413, 'File exceeds the portal size limit.');
            hash.update(chunk);
            await handle.writeFile(chunk);
          }
          await handle.sync();
        } finally { await handle.close(); }
        if (closing) throw new HttpError(503, 'The portal is shutting down.');
        // A body that ended early (client abort) must never start a sender.
        if (!request.complete) throw new HttpError(400, 'The upload was interrupted.');
        const transfer = { id: randomUUID(), name, size, sha256: hash.digest('hex'), state: 'sending', authenticated: false };
        transfers.unshift(transfer);
        transfers.splice(30);
        const proc = child(['send-file', '--peer', peer.address, '--peer-key', peer.key, '--file', filename]);
        sender = proc;
        const statusLine = line => {
          if (/event=tunnel_established role=initiator peer=ed25519:[a-f0-9]{64} session_id=[a-f0-9]+$/.test(line)) transfer.authenticated = true;
        };
        lines(proc.stdout, statusLine);
        lines(proc.stderr, statusLine);
        const timeout = setTimeout(() => proc.kill('SIGINT'), Math.max(120000, Math.ceil(size / 600000) * 1000 + 30000));
        const cleanup = async (code) => {
          clearTimeout(timeout);
          transfer.state = code === 0 && transfer.authenticated ? 'sent-awaiting-receiver' : 'failed';
          if (transfer.state === 'failed') transfer.error = 'Authenticated transfer failed. Confirm both devices have pinned the correct identity and try again.';
          sender = null;
          await fs.rm(directory, { recursive: true, force: true });
          busy = false;
        };
        let settled = false;
        let resolveCleanup;
        senderDone = new Promise(resolve => { resolveCleanup = resolve; });
        const settle = code => { if (!settled) { settled = true; cleanup(code).catch(() => { busy = false; }).finally(resolveCleanup); } };
        proc.once('error', () => settle(1));
        proc.once('exit', settle);
        json(response, 202, { transfer });
      } catch (error) {
        if (directory) await fs.rm(directory, { recursive: true, force: true });
        busy = false;
        throw error;
      } finally { activeUploads.delete(request); }
    }

    const handleRequest = async (request, response) => {
      response.setHeader('X-Content-Type-Options', 'nosniff');
      response.setHeader('Referrer-Policy', 'no-referrer');
      response.setHeader('Cross-Origin-Resource-Policy', 'same-origin');
      response.setHeader('Content-Security-Policy', "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'");
      try {
        if (closing) throw new HttpError(503, 'The portal is shutting down.');
        originGuard(request);
        const url = new URL(request.url, `http://${request.headers.host}`);
        if (url.pathname === '/api/session' && request.method === 'GET') return json(response, 200, { token });
        if (url.pathname.startsWith('/api/')) {
          if (!secureEqual(request.headers['x-portal-token'], token)) throw new HttpError(401, 'Portal authorization is required.');
          if (url.pathname === '/api/state' && request.method === 'GET') return json(response, 200, snapshot());
          if (url.pathname === '/api/peers' && request.method === 'POST') {
            const body = await readJSON(request);
            const bundle = body?.pairing;
            if (!bundle || bundle.version !== 1 || !keyPattern.test(bundle.key) || bundle.key === key || typeof body.name !== 'string' || !body.name.trim() || body.name.length > 64 || /[\x00-\x1f\x7f]/.test(body.name)) throw new HttpError(400, 'Enter a device name and a valid pairing bundle from another device.');
            address(bundle.address);
            return await mutatePeers(async () => {
              // Pairing is idempotent: the same identity at the same address is
              // one pin. Repeating it changes nothing and leaves the receiver alone.
              const pinned = peers.find(peer => peer.key === bundle.key);
              if (pinned) {
                if (pinned.address !== bundle.address) throw new HttpError(409, 'This identity is already pinned. Remove it before changing its address.');
                return json(response, 200, { peer: pinned, alreadyPinned: true });
              }
              if (peers.length >= 16) throw new HttpError(409, 'This portal supports at most 16 pinned peers.');
              if (busy || receiver.status === 'authenticated') throw new HttpError(409, 'Wait for the active transfer to finish.');
              await validatePeerKey(bundle.key);
              if (closing) throw new HttpError(503, 'The portal is shutting down.');
              if (busy || receiver.status === 'authenticated') throw new HttpError(409, 'Wait for the active transfer to finish.');
              const peer = { id: randomUUID(), name: body.name.trim(), key: bundle.key, address: bundle.address };
              peers.push(peer);
              try { await persistPeers(); } catch (error) { peers.splice(peers.indexOf(peer), 1); throw error; }
              await restartReceiver();
              return json(response, 201, { peer });
            });
          }
          const peerMatch = /^\/api\/peers\/([a-f0-9-]{36})$/.exec(url.pathname);
          if (peerMatch && request.method === 'DELETE') {
            return await mutatePeers(async () => {
              const index = peers.findIndex(peer => peer.id === peerMatch[1]);
              if (index < 0) throw new HttpError(404, 'Pinned peer not found.');
              if (busy || receiver.status === 'authenticated') throw new HttpError(409, 'Wait for the active transfer to finish.');
              const [removed] = peers.splice(index, 1);
              try { await persistPeers(); } catch (error) { peers.splice(index, 0, removed); throw error; }
              await restartReceiver();
              return json(response, 200, { removed: true });
            });
          }
          if (url.pathname === '/api/send' && request.method === 'POST') return await upload(request, response, url);
          const downloadMatch = /^\/api\/files\/([a-f0-9-]{36})$/.exec(url.pathname);
          if (downloadMatch && request.method === 'GET') {
            const file = files.find(item => item.id === downloadMatch[1]);
            if (!file) throw new HttpError(404, 'Verified file not found.');
            const filename = path.join(inbox, file.id);
            const verified = await digestFile(filename, maxBytes);
            if (verified.size !== file.size || verified.sha256 !== file.sha256) throw new HttpError(409, 'File verification failed.');
            const { handle, stat } = await regularFile(filename);
            if (stat.size !== file.size) { await handle.close(); throw new HttpError(409, 'File changed.'); }
            response.writeHead(200, { 'Content-Type': 'application/octet-stream', 'Content-Length': file.size, 'Content-Disposition': `attachment; filename*=UTF-8''${encodeURIComponent(file.name)}`, 'Cache-Control': 'no-store', 'X-Content-SHA256': file.sha256 });
            const stream = handle.createReadStream();
            stream.on('error', () => response.destroy());
            response.on('close', () => stream.destroy());
            stream.pipe(response);
            return;
          }
          throw new HttpError(404, 'API endpoint not found.');
        }
        if (!['GET', 'HEAD'].includes(request.method)) throw new HttpError(405, 'Method not allowed.');
        const rawPath = decodeURIComponent(url.pathname);
        if (rawPath.includes('\\') || rawPath.split('/').includes('..')) throw new HttpError(400, 'Unsafe path.');
        const relative = rawPath === '/' || rawPath === '/advanced' ? 'index.html' : rawPath.slice(1);
        const root = path.resolve(here, '../dist');
        const filename = path.resolve(root, relative);
        if (!filename.startsWith(`${root}${path.sep}`)) throw new HttpError(403, 'Unsafe path.');
        const { handle, stat } = await regularFile(filename).catch(() => { throw new HttpError(404, 'Build the web interface first.'); });
        const types = { '.html': 'text/html; charset=utf-8', '.js': 'text/javascript; charset=utf-8', '.css': 'text/css; charset=utf-8', '.png': 'image/png', '.svg': 'image/svg+xml', '.ico': 'image/x-icon' };
        response.writeHead(200, { 'Content-Type': types[path.extname(filename)] || 'application/octet-stream', 'Content-Length': stat.size, 'Cache-Control': 'no-cache' });
        if (request.method === 'HEAD') { await handle.close(); response.end(); } else { const stream = handle.createReadStream(); stream.on('error', () => response.destroy()); response.on('close', () => stream.destroy()); stream.pipe(response); }
      } catch (error) {
        if (!response.headersSent && !response.destroyed) json(response, error instanceof HttpError ? error.status : 500, { error: error instanceof HttpError ? error.message : 'The portal could not complete this request.' });
        else response.destroy();
      }
    };
    server = http.createServer((request, response) => {
      const operation = handleRequest(request, response);
      operations.add(operation);
      operation.finally(() => operations.delete(operation));
    });
    server.requestTimeout = 120000;
    server.headersTimeout = 15000;
    server.maxHeadersCount = 32;
    await new Promise((resolve, reject) => { server.once('error', reject); server.listen(port, host, resolve); });
    return {
      server, stateDir,
      async close() {
        if (closing) return;
        closing = true;
        clearInterval(timer);
        const closed = new Promise(resolve => server.close(resolve));
        for (const request of activeUploads) request.destroy();
        server.closeAllConnections();
        await Promise.allSettled([...operations]);
        if (scanTask) await scanTask;
        await Promise.all([terminate(sender), terminate(receiverChild)]);
        if (senderDone) await senderDone;
        if (restarting) await restarting;
        if (receiverDirectory) await fs.rm(receiverDirectory, { recursive: true, force: true });
        await closed;
        await fs.rm(lockPath, { force: true });
      },
    };
  } catch (error) {
    closing = true;
    clearInterval(timer);
    await terminate(receiverChild);
    await fs.rm(lockPath, { force: true });
    throw error;
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  createPortal().then(portal => {
    console.log(`CrypRQ portal listening on http://${process.env.HOST || '127.0.0.1'}:${process.env.PORT || 8787}`);
    let stopping = false;
    const stop = () => { if (!stopping) { stopping = true; portal.close().finally(() => process.exit(0)); } };
    process.on('SIGINT', stop);
    process.on('SIGTERM', stop);
  }).catch(() => { console.error('CrypRQ portal startup failed. Check the CLI, private state paths, and configuration.'); process.exitCode = 1; });
}
