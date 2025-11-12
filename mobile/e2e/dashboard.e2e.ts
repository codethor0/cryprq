describe('Dashboard', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  it('should show disconnected status on launch', async () => {
    await expect(element(by.id('status-pill'))).toBeVisible();
    await expect(element(by.text('Disconnected'))).toBeVisible();
  });

  it('should show connect button when disconnected', async () => {
    await expect(element(by.id('connect-button'))).toBeVisible();
  });

  it('should navigate to logs', async () => {
    await element(by.id('view-logs-button')).tap();
    await expect(element(by.text('Logs'))).toBeVisible();
  });

  it('should display status card', async () => {
    await expect(element(by.id('status-card'))).toBeVisible();
  });

  it('should display throughput card', async () => {
    await expect(element(by.id('throughput-card'))).toBeVisible();
  });
});

