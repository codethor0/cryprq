import { test, expect } from '@playwright/test';

const BRIDGE = process.env.BRIDGE_URL ?? 'http://localhost:8787';
const SITE   = process.env.SITE_URL   ?? 'http://localhost:5173';
const PORT   = Number(process.env.CRYPRQ_PORT ?? 9999);

test.describe.configure({ mode: 'serial', timeout: 60_000 });

test('listener then dialer emit events', async ({ page, request }) => {
  // Start listener via bridge
  const r1 = await request.post(`${BRIDGE}/connect`, {
    data: { mode: 'listener', port: PORT }
  });
  expect(r1.ok()).toBeTruthy();

  await page.goto(SITE, { waitUntil: 'domcontentloaded' });
  await page.waitForSelector('text=CrypRQ Web Tester');

  // Open events stream by simply staying on the page; now start the dialer
  const r2 = await request.post(`${BRIDGE}/connect`, {
    data: { mode: 'dialer', port: PORT, peer: `/ip4/127.0.0.1/udp/${PORT}/quic-v1` }
  });
  expect(r2.ok()).toBeTruthy();

  // Expect handshake/peer/rotation to appear in UI within 20s
  await expect(page.getByText(/handshake|peer|rotation/i)).toBeVisible({ timeout: 20_000 });
});

