// Copyright (c) 2025 Thor Thor
// Author: Thor Thor (GitHub: https://github.com/codethor0)
// Contact: codethor@gmail.com
// LinkedIn: https://www.linkedin.com/in/thor-thor0
// License: MIT (see LICENSE file for details)

import { test, expect } from '@playwright/test';

const baseURL = process.env.CRYPRQ_WEB_BASE_URL || 'http://127.0.0.1:18787';

test.describe('Web E2E Smoke', () => {
  test('web UI loads and shows critical elements', async ({ page }) => {
    await page.goto(baseURL);

    await expect(page).toHaveTitle(/CrypRQ/i);

    const appRoot = page.getByTestId('app-root');
    await expect(appRoot).toBeVisible();

    const appTitle = page.getByTestId('app-title');
    await expect(appTitle).toBeVisible();
    await expect(appTitle).toContainText('CrypRQ');

    await expect(page.getByTestId('pairing-bundle')).toBeVisible();
    await expect(page.getByTestId('send-file')).toBeAttached();
    await expect(page.getByRole('heading', { name: 'Received files' })).toBeVisible();
  });

  test('shows the public pairing identity without claiming encryption', async ({ page }) => {
    await page.goto(baseURL);

    await expect(page.getByTestId('pairing-bundle')).toHaveValue(/ed25519:[a-f0-9]{64}/);
    await expect(page.getByTestId('pairing-bundle')).toHaveValue(/"version":\s*1/);
    await expect(page.getByText('End-to-end encrypted', { exact: true })).toHaveCount(0);
  });
});
