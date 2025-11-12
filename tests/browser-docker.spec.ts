// © 2025 Thor Thor
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// SPDX-License-Identifier: MIT

import { test, expect } from '@playwright/test';
import { execSync } from 'child_process';

test.describe('CrypRQ Docker VPN Browser Tests', () => {
  test.beforeAll(async () => {
    // Ensure Docker container is running
    try {
      execSync('docker ps | grep -q cryprq-vpn', { stdio: 'ignore' });
    } catch {
      throw new Error('Docker container cryprq-vpn is not running. Run: ./scripts/docker-vpn-start.sh');
    }
  });

  test('should load web UI', async ({ page }) => {
    await page.goto('http://localhost:8787');
    await expect(page.locator('h1')).toContainText('CrypRQ');
  });

  test('should connect as listener (Docker mode)', async ({ page }) => {
    await page.goto('http://localhost:8787');
    
    // Select listener mode
    await page.selectOption('select', 'listener');
    
    // Click connect
    await page.click('button:has-text("Connect")');
    
    // Wait for Docker mode messages
    await page.waitForTimeout(2000);
    
    // Check for Docker mode indicators
    const bodyText = await page.textContent('body');
    expect(bodyText).toMatch(/Docker mode|Container|172\.19\.0\./);
  });

  test('should connect as dialer to container', async ({ page }) => {
    await page.goto('http://localhost:8787');
    
    // Select dialer mode
    await page.selectOption('select', 'dialer');
    
    // Click connect
    await page.click('button:has-text("Connect")');
    
    // Wait for connection
    await page.waitForTimeout(5000);
    
    // Check debug console for connection messages
    const debugConsole = page.locator('[style*="position: fixed"][style*="bottom: 0"]');
    const consoleText = await debugConsole.textContent();
    
    // Should see connection-related messages
    expect(consoleText).toMatch(/Dialing|Connected|Inbound|Container/);
    
    // Check container logs for connection
    const containerLogs = execSync('docker logs --tail 10 cryprq-vpn 2>&1', { encoding: 'utf8' });
    expect(containerLogs).toMatch(/Inbound connection established|Incoming connection/);
  });

  test('should show encryption status', async ({ page }) => {
    await page.goto('http://localhost:8787');
    
    // Check encryption status is displayed
    const encryptionStatus = page.locator('text=ML-KEM');
    await expect(encryptionStatus).toBeVisible();
  });

  test('should stream container logs in debug console', async ({ page }) => {
    await page.goto('http://localhost:8787');
    
    // Connect as listener
    await page.selectOption('select', 'listener');
    await page.click('button:has-text("Connect")');
    
    await page.waitForTimeout(3000);
    
    // Check debug console has content
    const debugConsole = page.locator('[style*="position: fixed"][style*="bottom: 0"]');
    const consoleText = await debugConsole.textContent();
    
    // Should have some log content
    expect(consoleText?.length).toBeGreaterThan(0);
  });
});

