describe('Developer Screen', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  it('should open Developer screen after 5 taps on version', async () => {
    await element(by.id('tab-settings')).tap();
    
    // Tap version area 5 times
    for (let i = 0; i < 5; i++) {
      await element(by.id('version-tap-area')).tap();
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    // Developer screen should be visible
    await expect(element(by.text('Developer Tools'))).toBeVisible();
  });

  it('should simulate rotation and show toast', async () => {
    await element(by.id('tab-settings')).tap();
    
    // Open developer screen
    for (let i = 0; i < 5; i++) {
      await element(by.id('version-tap-area')).tap();
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    await element(by.id('simulate-rotation-button')).tap();
    
    // Should show success alert
    await waitFor(element(by.text('Success'))).toBeVisible().withTimeout(2000);
  });

  it('should switch endpoints quickly', async () => {
    await element(by.id('tab-settings')).tap();
    
    // Open developer screen
    for (let i = 0; i < 5; i++) {
      await element(by.id('version-tap-area')).tap();
      await new Promise(resolve => setTimeout(resolve, 100));
    }
    
    // Switch to LAN
    await element(by.id('switch-lan-button')).tap();
    await waitFor(element(by.text('Success'))).toBeVisible().withTimeout(2000);
    
    // Switch to REMOTE
    await element(by.id('switch-remote-button')).tap();
    await waitFor(element(by.text('Success'))).toBeVisible().withTimeout(2000);
  });
});

