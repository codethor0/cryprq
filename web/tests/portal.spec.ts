// Copyright (c) 2025 Thor Thor
// License: MIT (see LICENSE file for details)

import { test, expect, type APIRequestContext, type APIResponse, type Page } from '@playwright/test';
import { createHash, generateKeyPairSync, randomBytes, randomUUID } from 'node:crypto';
import { readFile } from 'node:fs/promises';

const portalA = process.env.CRYPRQ_WEB_BASE_URL || 'http://127.0.0.1:18787';
const portalB = process.env.CRYPRQ_WEB_PEER_URL || 'http://127.0.0.1:18788';
const sha256 = (bytes: Buffer) => createHash('sha256').update(bytes).digest('hex');
const distinctPublicKey = () => `ed25519:${generateKeyPairSync('ed25519').publicKey.export({ format: 'der', type: 'spki' }).subarray(-32).toString('hex')}`;

type Pairing = { version: number; key: string; address: string };
type State = {
  identity: { key: string; pairing: Pairing };
  receiver: { status: string };
  peers: { id: string; key: string }[];
  transfers: { id: string; state: string }[];
  files: { id: string; name: string; size: number; sha256: string; verified: boolean }[];
  limits: { maxFileBytes: number };
};

async function openAPI(request: APIRequestContext, base: string) {
  const response = await request.get(`${base}/api/session`);
  expect(response.ok()).toBeTruthy();
  const { token } = await response.json();
  const headers = { 'X-Portal-Token': token };
  return {
    headers,
    base,
    async state(): Promise<State> {
      const result = await request.get(`${base}/api/state`, { headers });
      expect(result.ok()).toBeTruthy();
      return result.json();
    },
    async pin(name: string, pairing: Pairing): Promise<{ id: string }> {
      const existing = (await this.state()).peers.find(peer => peer.key === pairing.key);
      if (existing) return existing;
      let result: APIResponse | undefined;
      await expect.poll(async () => {
        result = await request.post(`${base}/api/peers`, { headers, data: { name, pairing } });
        expect([201, 409]).toContain(result.status());
        return result.status();
      }, { timeout: 10_000 }).toBe(201);
      return (await result!.json()).peer;
    },
    async revoke(id: string) {
      await expect.poll(async () => {
        const result = await request.delete(`${base}/api/peers/${id}`, { headers });
        expect([200, 409]).toContain(result.status());
        return result.status();
      }, { timeout: 10_000 }).toBe(200);
    },
    async send(peer: string, name: string, bytes: Buffer) {
      return request.post(`${base}/api/send?peer=${encodeURIComponent(peer)}`, {
        headers: { ...headers, 'Content-Type': 'application/octet-stream', 'X-Filename': encodeURIComponent(name) },
        data: bytes,
      });
    },
  };
}

async function pairInBrowser(page: Page, name: string, pairing: string) {
  // The form is collapsed once a device is pinned. Open it as a user would.
  if (!(await page.getByTestId('pair-name').isVisible())) await page.locator('details.pair-form > summary').click();
  await page.getByTestId('pair-name').fill(name);
  await page.getByTestId('pair-peer').fill(pairing);
  expect(JSON.parse(await page.getByTestId('pair-peer').inputValue()).key).toBe(JSON.parse(pairing).key);
  await page.getByTestId('pair-confirm').check();
  await page.getByTestId('pair-device').click();
  await expect(page.getByTestId('receiver-status')).toContainText(/listening|ready/i, { timeout: 30_000 });
}

test('two independent browsers pair, send both ways, and download identical files', async ({ browser, request }, testInfo) => {
  test.setTimeout(150_000);
  const contextA = await browser.newContext({ acceptDownloads: true });
  const contextB = await browser.newContext({ acceptDownloads: true });
  try {
    const a = await contextA.newPage();
    const b = await contextB.newPage();
    await Promise.all([a.goto(portalA), b.goto(portalB)]);
    await expect(a.getByTestId('pairing-bundle')).toHaveValue(/ed25519:/);
    await expect(b.getByTestId('pairing-bundle')).toHaveValue(/ed25519:/);
    const pairingA = await a.getByTestId('pairing-bundle').inputValue();
    const pairingB = await b.getByTestId('pairing-bundle').inputValue();
    expect(JSON.parse(pairingA).key).not.toBe(JSON.parse(pairingB).key);
    await pairInBrowser(a, 'Portal B', pairingB);
    await pairInBrowser(b, 'Portal A', pairingA);

    const evidence: object[] = [];
    const repeatedName = `A-to-B-${randomUUID()}.bin`;
    for (const [sender, receiver, receiverURL, direction] of [
      [a, b, portalB, 'A-to-B'],
      [b, a, portalA, 'B-to-A'],
      [a, b, portalB, 'A-to-B-repeat'],
    ] as const) {
      const name = direction.startsWith('A-to-B') ? repeatedName : `${direction}-${randomUUID()}.bin`;
      const original = Buffer.concat([Buffer.from(`CrypRQ ${direction}\n`), randomBytes(64 * 1024 + 37)]);
      const originalSHA = sha256(original);
      await sender.getByTestId('send-file').setInputFiles({ name, mimeType: 'application/octet-stream', buffer: original });
      await expect(sender.getByTestId('send-file-button')).toBeEnabled({ timeout: 30_000 });
      await sender.getByTestId('send-file-button').click();
      const rows = receiver.getByTestId('received-file').filter({ hasText: name });
      if (direction === 'A-to-B-repeat') await expect(rows).toHaveCount(2, { timeout: 45_000 });
      const row = rows.first();
      await expect(row).toBeVisible({ timeout: 45_000 });
      await expect(row).toContainText(/SHA-256 verified/i);
      const downloadPromise = receiver.waitForEvent('download');
      await row.getByRole('button', { name: /download/i }).click();
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toBe(name);
      const output = testInfo.outputPath(name);
      await download.saveAs(output);
      const downloaded = await readFile(output);
      expect(downloaded.length).toBe(original.length);
      expect(sha256(downloaded)).toBe(originalSHA);
      expect(downloaded.equals(original)).toBe(true);
      const receiverAPI = await openAPI(request, receiverURL);
      const received = (await receiverAPI.state()).files.find(file => file.name === name);
      expect(received).toMatchObject({ size: original.length, sha256: originalSHA, verified: true });
      evidence.push({ direction, filename: name, bytes: original.length, originalSHA256: originalSHA, downloadedSHA256: sha256(downloaded) });
    }
    await testInfo.attach('transfer-sha256-evidence', { body: JSON.stringify(evidence, null, 2), contentType: 'application/json' });
  } finally {
    await Promise.all([contextA.close(), contextB.close()]);
  }
});

test('real portals reject wrong identities, untrusted peers, unsafe uploads, and unauthorized downloads', async ({ request }, testInfo) => {
  test.setTimeout(150_000);
  const a = await openAPI(request, portalA);
  const b = await openAPI(request, portalB);
  const identityA = (await a.state()).identity;
  const identityB = (await b.state()).identity;
  const peerB = await a.pin('Portal B', identityB.pairing);
  let peerA = await b.pin('Portal A', identityA.pairing);
  const ready = async (portal: typeof a) => {
    await expect.poll(async () => (await portal.state()).receiver.status, { timeout: 30_000 }).toBe('listening');
  };
  await Promise.all([ready(a), ready(b)]);
  const rejectHandshake = async (peer: string, name: string) => {
    const response = await a.send(peer, name, Buffer.from('must not be published'));
    expect(response.status()).toBe(202);
    const { transfer } = await response.json();
    await expect.poll(async () => (await a.state()).transfers.find(item => item.id === transfer.id)?.state, { timeout: 25_000 }).toBe('failed');
    expect((await b.state()).files.some(file => file.name === name)).toBe(false);
  };

  const wrongIdentity = await a.pin('Wrong identity', { ...identityB.pairing, key: distinctPublicKey() });
  try {
    await ready(b);
    await rejectHandshake(wrongIdentity.id, `wrong-identity-${randomUUID()}.txt`);
  } finally {
    await a.revoke(wrongIdentity.id);
  }
  const outsider = await b.pin('Untrusted sender identity', { ...identityA.pairing, key: distinctPublicKey() });
  try {
    await b.revoke(peerA.id);
    await ready(b);
    await rejectHandshake(peerB.id, `untrusted-${randomUUID()}.txt`);
  } finally {
    peerA = await b.pin('Portal A', identityA.pairing);
    await b.revoke(outsider.id);
  }
  await ready(b);

  const limit = (await a.state()).limits.maxFileBytes;
  const tooLarge = await a.send(peerB.id, 'oversized.bin', Buffer.alloc(limit + 1, 0x42));
  expect(tooLarge.status()).toBe(413);
  const unsafe = await a.send(peerB.id, '../unsafe.txt', Buffer.from('unsafe'));
  expect(unsafe.status()).toBe(400);
  const crossOrigin = await request.get(`${portalA}/api/session`, { headers: { Origin: 'https://untrusted.invalid', 'Sec-Fetch-Site': 'cross-site' } });
  expect(crossOrigin.status()).toBe(403);

  const name = `recovery-${randomUUID()}.txt`;
  const original = Buffer.from('A verified transfer succeeds after rejected handshakes and uploads.\n');
  const successful = await a.send(peerB.id, name, original);
  expect(successful.status()).toBe(202);
  await expect.poll(async () => (await b.state()).files.some(file => file.name === name), { timeout: 45_000 }).toBe(true);
  const file = (await b.state()).files.find(item => item.name === name)!;
  expect(file.sha256).toBe(sha256(original));
  const unauthorized = await request.get(`${portalB}/api/files/${file.id}`);
  expect([401, 403]).toContain(unauthorized.status());
  const downloaded = await request.get(`${portalB}/api/files/${file.id}`, { headers: b.headers });
  expect(downloaded.ok()).toBeTruthy();
  expect(sha256(await downloaded.body())).toBe(sha256(original));
  await testInfo.attach('security-evidence', { body: JSON.stringify({ wrongIdentity: 'rejected', untrustedPeer: 'rejected', oversizedUpload: tooLarge.status(), unsafeFilename: unsafe.status(), crossOrigin: crossOrigin.status(), unauthorizedDownload: unauthorized.status(), recoverySHA256: file.sha256 }, null, 2), contentType: 'application/json' });
});
