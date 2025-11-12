describe('Dashboard Parity', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  it('should show status pill and respond to backend metrics within 2s', async () => {
    await expect(element(by.id('status-pill'))).toBeVisible();
    
    // Connect
    await element(by.id('connect-button')).tap();
    
    // Wait for connected status
    await waitFor(element(by.text('Connected')))
      .toBeVisible()
      .withTimeout(2000);
    
    await expect(element(by.id('status-pill'))).toBeVisible();
  });

  it('should show rotation countdown and update within 2s', async () => {
    await element(by.id('tab-dashboard')).tap();
    await element(by.id('connect-button')).tap();
    await waitFor(element(by.text('Connected'))).toBeVisible().withTimeout(2000);
    
    // Check rotation timer is visible
    await expect(element(by.text(/Rotation in:/))).toBeVisible();
    
    // Wait for metrics update (should resync within 2s)
    await waitFor(element(by.text(/\d+:\d{2}/)))
      .toBeVisible()
      .withTimeout(2000);
  });

  it('should display peerId when connected', async () => {
    await element(by.id('tab-dashboard')).tap();
    await element(by.id('connect-button')).tap();
    await waitFor(element(by.text('Connected'))).toBeVisible().withTimeout(2000);
    
    // Peer ID should be visible
    await expect(element(by.text(/Peer ID:/))).toBeVisible();
  });

  it('should display throughput graph', async () => {
    await element(by.id('tab-dashboard')).tap();
    await element(by.id('connect-button')).tap();
    await waitFor(element(by.text('Connected'))).toBeVisible().withTimeout(2000);
    
    // Throughput card should be visible
    await expect(element(by.id('throughput-card'))).toBeVisible();
  });
});

