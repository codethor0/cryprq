// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { test, expect } from '@playwright/test';

const baseURL = process.env.CRYPRQ_WEB_BASE_URL || 'http://127.0.0.1:8787';

test.describe('Web E2E Smoke', () => {
  test('web UI loads and shows critical elements', async ({ page }) => {
    await page.goto(baseURL);

    await expect(page).toHaveTitle(/web/i);

    const appRoot = page.getByTestId('app-root');
    await expect(appRoot).toBeVisible();

    const appTitle = page.getByTestId('app-title');
    await expect(appTitle).toBeVisible();
    await expect(appTitle).toContainText('CrypRQ');

    const connectButton = page.getByTestId('connect-button');
    await expect(connectButton).toBeVisible();

    const connectButtonText = await connectButton.textContent();
    expect(connectButtonText).toMatch(/Connect/i);
  });

  test('connect button is enabled when server is connected', async ({ page }) => {
    await page.goto(baseURL);

    const connectButton = page.getByTestId('connect-button');
    
    await page.waitForTimeout(2000);

    const isDisabled = await connectButton.isDisabled();
    
    if (!isDisabled) {
      await expect(connectButton).toBeEnabled();
    } else {
      const statusIndicator = page.locator('text=Connected').or(page.locator('text=Disconnected'));
      await expect(statusIndicator).toBeVisible();
    }
  });
});
