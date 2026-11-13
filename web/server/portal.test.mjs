// Copyright (c) 2025 Thor Thor
// License: MIT (see LICENSE file for details)

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createHash, randomBytes } from 'node:crypto';
import * as fs from 'node:fs/promises';
import http from 'node:http';
import net from 'node:net';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createPortal, safeFilename } from './server.mjs';

const binary = process.env.CRYPRQ_BIN ? path.resolve(process.env.CRYPRQ_BIN) : fileURLToPath(new URL('../../target/release/cryprq', import.meta.url));
const delay = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
async function eventually(check, timeout = 15000) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) { const result = await check(); if (result) return result; await delay(100); }
  throw new Error('Timed out waiting for the real portal.');
}
async function freePort() {
  const server = net.createServer();
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const port = server.address().port;
  await new Promise(resolve => server.close(resolve));
  return port;
}
async function portal(root, name) {
  const port = await freePort();
  const udp = await freePort();
  const stateDir = path.join(root, name);
  const instance = await createPortal({ env: { PORT: String(port), HOST: '127.0.0.1', CRYPRQ_STATE_DIR: stateDir, CRYPRQ_BIN: binary, CRYPRQ_LISTEN_ADDR: `/ip4/127.0.0.1/udp/${udp}`, CRYPRQ_MAX_FILE_BYTES: '1048576', CRYPRQ_RECEIVER_IDLE_MS: '1000' } });
  const base = `http://127.0.0.1:${port}`;
  const { token } = await (await fetch(`${base}/api/session`)).json();
  async function api(endpoint, options = {}) {
    return fetch(`${base}${endpoint}`, { ...options, headers: { 'X-Portal-Token': token, ...options.headers } });
  }
  return { ...instance, base, api, token, async state() { return (await api('/api/state')).json(); } };
}
async function pin(target, other, pairing) {
  const state = await other.state();
  const response = await target.api('/api/peers', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name: 'Other device', pairing: pairing || state.identity.pairing }) });
  assert.equal(response.status, 201, await response.text());
  return (await target.state()).peers.at(-1);
}
async function send(target, peer, name, bytes) {
  const response = await target.api(`/api/send?peer=${peer.id}`, { method: 'POST', headers: { 'Content-Type': 'application/octet-stream', 'X-Filename': encodeURIComponent(name) }, body: bytes });
  assert.equal(response.status, 202, await response.text());
  return (await target.state()).transfers[0];
}
function cli(stateDir, args) {
  return new Promise((resolve, reject) => {
    const proc = spawn(binary, ['--identity', path.join(stateDir, 'identity.key'), '--trusted-peers', path.join(stateDir, 'trusted_peers'), ...args], { env: { ...process.env, RUST_LOG: 'info' }, stdio: ['ignore', 'pipe', 'pipe'] });
    let output = '';
    proc.stdout.on('data', chunk => { output += chunk; });
    proc.stderr.on('data', chunk => { output += chunk; });
    proc.once('error', reject);
    proc.once('exit', code => resolve({ code, output }));
  });
}

test('safe filenames reject traversal, alternate separators, control bytes and device paths', () => {
  for (const name of ['', '../x', 'a/b', 'a\\b', '.hidden', '..', 'x\n.txt', 'x\0', 'C:x', 'CON', 'aux.txt', 'x.', 'x ']) assert.equal(safeFilename(name), false, name);
  for (const name of ['report.pdf', 'photo 2.png', 'résumé.txt', 'release-1.tar.gz']) assert.equal(safeFilename(name), true, name);
});

test('real authenticated adapter transfer, request boundaries, rejection and recovery', { timeout: 100000 }, async t => {
  await fs.access(binary);
  const root = await fs.mkdtemp(path.join(await fs.realpath(tmpdir()), 'cryprq-portal-adapter-'));
  let a;
  let b;
  t.after(async () => { await Promise.all([a?.close(), b?.close()]); await fs.rm(root, { recursive: true, force: true }); });
  a = await portal(root, 'a');
  b = await portal(root, 'b');
  assert.notEqual((await a.state()).identity.key, (await b.state()).identity.key);
  assert.equal((await a.state()).receiver.status, 'needs-pairing');
  // Concurrent identical pairing requests produce one pin, never duplicate trust.
  const bundle = (await b.state()).identity.pairing;
  const concurrentPins = await Promise.all([1, 2].map(() => a.api('/api/peers', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name: 'Other device', pairing: bundle }) })));
  assert.deepEqual(concurrentPins.map(response => response.status).sort(), [200, 201]);
  const ab = (await a.state()).peers[0];
  assert.equal((await a.state()).peers.length, 1);
  assert.equal((await fs.readFile(path.join(a.stateDir, 'trusted_peers'), 'utf8')).trim(), bundle.key);
  const ba = await pin(b, a);
  await eventually(async () => (await a.state()).receiver.status === 'listening' && (await b.state()).receiver.status === 'listening');

  for (const [target, peer, receiver, name] of [[a, ab, b, 'forward.bin'], [b, ba, a, 'reverse.bin'], [a, ab, b, 'forward.bin']]) {
    const bytes = randomBytes(131071);
    const expected = createHash('sha256').update(bytes).digest('hex');
    const count = (await receiver.state()).files.length;
    await send(target, peer, name, bytes);
    const file = await eventually(async () => { const files = (await receiver.state()).files; return files.length > count && files[0]; });
    assert.equal(file.size, bytes.length);
    assert.equal(file.sha256, expected);
    assert.equal(file.verified, true);
    const downloaded = await receiver.api(`/api/files/${file.id}`);
    assert.equal(downloaded.status, 200);
    const downloadedBytes = Buffer.from(await downloaded.arrayBuffer());
    assert.deepEqual(downloadedBytes, bytes);
    console.log(JSON.stringify({ direction: name, bytes: file.size, originalSHA256: expected, downloadedSHA256: createHash('sha256').update(downloadedBytes).digest('hex') }));
    await eventually(async () => (await target.state()).transfers[0].state === 'sent-awaiting-receiver');
    assert.equal((await target.state()).transfers[0].authenticated, true);
    await eventually(async () => (await receiver.state()).receiver.status === 'listening');
  }

  const badPin = await a.api('/api/peers', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name: 'Invalid point', pairing: { version: 1, key: `ed25519:${'00'.repeat(32)}`, address: (await b.state()).identity.address } }) });
  assert.equal(badPin.status, 400);
  assert.equal((await a.state()).peers.length, 1);
  assert.equal((await a.state()).receiver.status, 'listening');

  const file = (await b.state()).files[0];
  assert.equal((await fetch(`${b.base}/api/files/${file.id}`)).status, 401);
  assert.equal((await fetch(`${b.base}/api/state`, { headers: { 'X-Portal-Token': a.token } })).status, 401);
  assert.equal((await fetch(`${b.base}/api/session`, { headers: { Origin: 'https://attacker.invalid' } })).status, 403);
  assert.equal((await fetch(`${b.base}/api/session`, { headers: { 'Sec-Fetch-Site': 'cross-site' } })).status, 403);
  assert.equal((await b.api('/api/files/../../identity.key')).status, 404);
  for (const name of ['../escape.bin', 'a\\escape.bin', '.cryprq-evil.part', 'newline\n.bin']) {
    assert.equal((await a.api(`/api/send?peer=${ab.id}`, { method: 'POST', headers: { 'Content-Type': 'application/octet-stream', 'X-Filename': encodeURIComponent(name) }, body: 'bad' })).status, 400);
  }
  assert.equal((await a.api(`/api/send?peer=${ab.id}`, { method: 'POST', headers: { 'Content-Type': 'application/octet-stream', 'X-Filename': 'oversized.bin' }, body: Buffer.alloc(1048577) })).status, 413);
  assert.deepEqual(await fs.readdir(path.join(a.stateDir, 'uploads')), []);

  // Chunked input crosses the limit without a Content-Length header.
  const chunkedStatus = await new Promise((resolve, reject) => {
    const request = http.request(`${a.base}/api/send?peer=${ab.id}`, { method: 'POST', headers: { 'X-Portal-Token': a.token, 'Content-Type': 'application/octet-stream', 'X-Filename': 'chunked.bin' } }, response => { response.resume(); resolve(response.statusCode); });
    request.on('error', reject);
    request.write(Buffer.alloc(600000)); request.end(Buffer.alloc(600000));
  });
  assert.equal(chunkedStatus, 413);
  await eventually(async () => (await fs.readdir(path.join(a.stateDir, 'uploads'))).length === 0);

  // An interrupted HTTP upload never starts a sender and leaves no temporary file.
  const transfersBefore = (await a.state()).transfers.length;
  await new Promise(resolve => {
    const request = http.request(`${a.base}/api/send?peer=${ab.id}`, { method: 'POST', headers: { 'X-Portal-Token': a.token, 'Content-Type': 'application/octet-stream', 'X-Filename': 'aborted.bin', 'Content-Length': '999999' } });
    request.on('error', () => resolve());
    request.write(Buffer.alloc(20000));
    setTimeout(() => request.destroy(), 50);
  });
  await eventually(async () => (await fs.readdir(path.join(a.stateDir, 'uploads'))).length === 0);
  assert.equal((await a.state()).transfers.length, transfersBefore);

  // The unpinned third identity uses the real authenticated CLI, not a mocked result.
  const outsider = path.join(root, 'outsider');
  await fs.mkdir(outsider, { mode: 0o700 });
  assert.equal((await cli(outsider, ['identity', 'init'])).code, 0);
  await fs.writeFile(path.join(outsider, 'trusted_peers'), `${(await b.state()).identity.key}\n`, { mode: 0o600 });
  const outsiderFile = path.join(outsider, 'untrusted.bin');
  await fs.writeFile(outsiderFile, 'must not arrive');
  const countBefore = (await b.state()).files.length;
  const rejected = await cli(outsider, ['send-file', '--peer', (await b.state()).identity.address, '--peer-key', (await b.state()).identity.key, '--file', outsiderFile]);
  assert.notEqual(rejected.code, 0);
  assert.match(rejected.output, /Handshake|handshake/);
  assert.equal((await b.state()).files.length, countBefore);

  // A trusted sender expecting a different identity must fail the actual handshake.
  const wrongKey = (await cli(outsider, ['identity', 'show'])).output.trim();
  assert.match(wrongKey, /^ed25519:[a-f0-9]{64}$/);
  await fs.writeFile(path.join(outsider, 'trusted_peers'), `${wrongKey}\n`, { mode: 0o600 });
  const wrong = await cli(outsider, ['send-file', '--peer', (await b.state()).identity.address, '--peer-key', wrongKey, '--file', outsiderFile]);
  assert.notEqual(wrong.code, 0);
  assert.equal((await b.state()).files.length, countBefore);

  // A pinned identity whose destination responds with another key also fails.
  assert.equal((await a.api(`/api/peers/${ab.id}`, { method: 'DELETE' })).status, 200);
  const falsePeer = await pin(a, b, { version: 1, key: wrongKey, address: (await b.state()).identity.address });
  await send(a, falsePeer, 'wrong-identity.bin', Buffer.from('no delivery'));
  await eventually(async () => (await a.state()).transfers[0].state === 'failed');
  assert.equal((await a.state()).transfers[0].authenticated, false);
  assert.equal((await b.state()).files.length, countBefore);
  assert.equal((await a.api(`/api/peers/${falsePeer.id}`, { method: 'DELETE' })).status, 200);
  const restored = await pin(a, b);
  await eventually(async () => (await a.state()).receiver.status === 'listening');

  // Interrupt a real authenticated CLI after the receiver has written partial data.
  const interruptedPath = path.join(root, 'interrupted.bin');
  await fs.writeFile(interruptedPath, randomBytes(900000));
  const interrupted = spawn(binary, ['--identity', path.join(a.stateDir, 'identity.key'), '--trusted-peers', path.join(a.stateDir, 'trusted_peers'), 'send-file', '--peer', (await b.state()).identity.address, '--peer-key', (await b.state()).identity.key, '--file', interruptedPath], { stdio: 'ignore' });
  const interruptedExit = new Promise(resolve => interrupted.once('exit', resolve));
  await eventually(async () => {
    for (const directory of await fs.readdir(path.join(b.stateDir, 'receiving'))) {
      const names = await fs.readdir(path.join(b.stateDir, 'receiving', directory));
      if (names.some(name => name.endsWith('.part'))) return true;
    }
    return false;
  });
  interrupted.kill('SIGKILL');
  await interruptedExit;
  await eventually(async () => (await b.state()).receiver.status === 'listening');
  assert.equal((await b.state()).files.some(item => item.name === 'interrupted.bin'), false);
  for (const directory of await fs.readdir(path.join(b.stateDir, 'receiving'))) assert.deepEqual(await fs.readdir(path.join(b.stateDir, 'receiving', directory)), []);

  // A direct authenticated CLI cannot bypass the portal's publication size policy.
  const tooLarge = path.join(root, 'inbound-oversized.bin');
  await fs.writeFile(tooLarge, randomBytes(1048577));
  await cli(a.stateDir, ['send-file', '--peer', (await b.state()).identity.address, '--peer-key', (await b.state()).identity.key, '--file', tooLarge]);
  await eventually(async () => (await b.state()).receiver.status === 'listening');
  assert.equal((await b.state()).files.some(item => item.name === 'inbound-oversized.bin'), false);
  await send(a, restored, 'recovery.bin', Buffer.from('recovered after rejected peers and aborted uploads'));
  await eventually(async () => (await b.state()).files.some(item => item.name === 'recovery.bin'));
  await eventually(async () => (await a.state()).transfers[0].state === 'sent-awaiting-receiver');
  assert.deepEqual(await fs.readdir(path.join(a.stateDir, 'uploads')), []);
  await eventually(async () => {
    for (const directory of await fs.readdir(path.join(b.stateDir, 'receiving'))) if ((await fs.readdir(path.join(b.stateDir, 'receiving', directory))).length) return false;
    return true;
  });

  // Download lookup rejects an inode replaced by a symlink and changed content.
  const destination = path.join(b.stateDir, 'inbox', file.id);
  const original = await fs.readFile(destination);
  await fs.unlink(destination);
  await fs.symlink(path.join(b.stateDir, 'identity.key'), destination);
  assert.equal((await b.api(`/api/files/${file.id}`)).status, 500);
  await fs.unlink(destination);
  await fs.writeFile(destination, 'tampered', { mode: 0o600 });
  assert.equal((await b.api(`/api/files/${file.id}`)).status, 409);
  await fs.writeFile(destination, original, { mode: 0o600 });
  assert.equal((await fs.stat(b.stateDir)).mode & 0o777, 0o700);
  assert.equal((await fs.stat(path.join(b.stateDir, 'identity.key'))).mode & 0o777, 0o600);

  // Shutdown aborts a live HTTP body before it can launch a CLI sender.
  const shutdownState = a.stateDir;
  const shutdownRequest = http.request(`${a.base}/api/send?peer=${restored.id}`, { method: 'POST', headers: { 'X-Portal-Token': a.token, 'Content-Type': 'application/octet-stream', 'X-Filename': 'shutdown.bin', 'Content-Length': '900000' } });
  shutdownRequest.on('error', () => {});
  shutdownRequest.write(Buffer.alloc(20000));
  await eventually(async () => (await fs.readdir(path.join(shutdownState, 'uploads'))).length === 1);
  await a.close();
  assert.equal(a.server.listening, false);
  a = null;
  await delay(200);
  assert.deepEqual(await fs.readdir(path.join(shutdownState, 'uploads')), []);
  assert.deepEqual(await fs.readdir(path.join(shutdownState, 'receiving')), []);
  await assert.rejects(fs.access(path.join(shutdownState, 'portal.lock')), { code: 'ENOENT' });
});

test('concurrent pairing is serialized, idempotent and leaves a running receiver alone', { timeout: 60000 }, async t => {
  await fs.access(binary);
  const root = await fs.mkdtemp(path.join(await fs.realpath(tmpdir()), 'cryprq-portal-pairing-'));
  let a;
  t.after(async () => { await a?.close(); await fs.rm(root, { recursive: true, force: true }); });
  a = await portal(root, 'a');
  // Three other devices: real identities created by the CLI, never by this portal.
  const bundles = [];
  for (const name of ['x', 'y', 'z']) {
    const directory = path.join(root, name);
    await fs.mkdir(directory, { mode: 0o700 });
    const created = await cli(directory, ['identity', 'init']);
    assert.equal(created.code, 0);
    bundles.push({ version: 1, key: created.output.trim().split('\n').at(-1), address: `/ip4/127.0.0.1/udp/${await freePort()}` });
  }
  const own = (await a.state()).identity.key;
  assert.equal(new Set([own, ...bundles.map(bundle => bundle.key)]).size, 4);
  const post = (pairing, name = 'Device') => a.api('/api/peers', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ name, pairing }) });
  const trustLines = async () => (await fs.readFile(path.join(a.stateDir, 'trusted_peers'), 'utf8')).split('\n').filter(Boolean);
  const session = async () => (await fs.readdir(path.join(a.stateDir, 'receiving'))).filter(name => name.startsWith('session-'));
  const listening = () => eventually(async () => (await a.state()).receiver.status === 'listening');

  // Six simultaneous requests for one identity: one pin, the rest report it.
  const same = await Promise.all(Array.from({ length: 6 }, () => post(bundles[0])));
  assert.deepEqual(same.map(response => response.status).sort(), [200, 200, 200, 200, 200, 201]);
  const sameBodies = await Promise.all(same.map(response => response.json()));
  assert.equal(new Set(sameBodies.map(body => body.peer.id)).size, 1);
  assert.equal((await a.state()).peers.length, 1);
  assert.deepEqual(await trustLines(), [bundles[0].key]);
  await listening();

  // Repeating a pairing, or refusing a conflicting one, does not restart the receiver.
  const before = await session();
  assert.equal(before.length, 1);
  assert.equal((await post(bundles[0], 'Another name')).status, 200);
  assert.equal((await post({ ...bundles[0], address: '/ip4/127.0.0.1/udp/9' })).status, 409);
  assert.equal((await post({ version: 1, key: own, address: bundles[0].address })).status, 400);
  await delay(700);
  assert.deepEqual(await session(), before);
  assert.equal((await a.state()).receiver.status, 'listening');
  assert.equal((await a.state()).peers[0].name, 'Device');

  // Different identities, each requested twice at once, all interleaved.
  const mixed = await Promise.all([bundles[1], bundles[2], bundles[1], bundles[2], bundles[0]].map(bundle => post(bundle)));
  assert.deepEqual(mixed.map(response => response.status).sort(), [200, 200, 200, 201, 201]);
  const pinned = (await a.state()).peers;
  assert.deepEqual(pinned.map(peer => peer.key).sort(), bundles.map(bundle => bundle.key).sort());
  assert.deepEqual((await trustLines()).sort(), bundles.map(bundle => bundle.key).sort());
  assert.deepEqual(JSON.parse(await fs.readFile(path.join(a.stateDir, 'peers.json'), 'utf8')).map(peer => peer.key).sort(), bundles.map(bundle => bundle.key).sort());
  await listening();
  assert.equal((await session()).length, 1);

  // Concurrent removal and re-pairing of the same identity stay consistent.
  const target = pinned.find(peer => peer.key === bundles[1].key);
  const removals = await Promise.all([1, 2, 3].map(() => a.api(`/api/peers/${target.id}`, { method: 'DELETE' })));
  assert.deepEqual(removals.map(response => response.status).sort(), [200, 404, 404]);
  const churn = await Promise.all([post(bundles[1]), a.api(`/api/peers/${pinned.find(peer => peer.key === bundles[2].key).id}`, { method: 'DELETE' }), post(bundles[1])]);
  assert.deepEqual(churn.map(response => response.status).sort(), [200, 200, 201]);
  const remaining = [bundles[0].key, bundles[1].key].sort();
  assert.deepEqual((await a.state()).peers.map(peer => peer.key).sort(), remaining);
  assert.deepEqual((await trustLines()).sort(), remaining);
  await listening();
  assert.equal((await session()).length, 1);
  assert.deepEqual((await fs.readdir(a.stateDir)).filter(name => name.endsWith('.tmp')), []);
});

test('state roots containing symlinks are rejected before invoking the CLI', async t => {
  const root = await fs.mkdtemp(path.join(await fs.realpath(tmpdir()), 'cryprq-portal-path-'));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  await fs.mkdir(path.join(root, 'real'));
  await fs.symlink(path.join(root, 'real'), path.join(root, 'link'));
  await assert.rejects(createPortal({ env: { CRYPRQ_STATE_DIR: path.join(root, 'link', 'portal'), CRYPRQ_BIN: binary } }), /symbolic links/);
  assert.deepEqual(await fs.readdir(path.join(root, 'real')), []);
});
