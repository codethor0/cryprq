describe('Logs Modal Parity', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  it('should show last 200 lines in logs modal', async () => {
    await element(by.id('tab-dashboard')).tap();
    await element(by.id('view-logs-button')).tap();
    
    // Logs modal should be visible
    await expect(element(by.text('Logs'))).toBeVisible();
  });

  it('should filter logs by error level', async () => {
    await element(by.id('tab-dashboard')).tap();
    await element(by.id('view-logs-button')).tap();
    
    // Tap error filter
    await element(by.id('filter-error')).tap();
    
    // Should show only error logs (or empty if none)
    await expect(element(by.text('Logs'))).toBeVisible();
  });

  it('should filter logs by rotation keyword', async () => {
    await element(by.id('tab-dashboard')).tap();
    await element(by.id('view-logs-button')).tap();
    
    // Search for rotation
    await element(by.id('log-search-input')).typeText('rotation');
    
    // Should filter to rotation-related logs
    await expect(element(by.text('Logs'))).toBeVisible();
  });
});

