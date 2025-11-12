import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],
  webServer: [
    {
      command: 'cd web/server && BRIDGE_PORT=8787 CRYPRQ_BIN=../../target/aarch64-apple-darwin/release/cryprq node server.mjs',
      port: 8787,
      reuseExistingServer: !process.env.CI,
    },
    {
      command: 'cd web && npm run dev -- --port 5173',
      port: 5173,
      reuseExistingServer: !process.env.CI,
    },
  ],
});

