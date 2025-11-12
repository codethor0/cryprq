describe('Peers', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  it('should navigate to peers tab', async () => {
    await element(by.id('tab-peers')).tap();
    await expect(element(by.text('Peers'))).toBeVisible();
  });

  it('should show add peer button', async () => {
    await element(by.id('tab-peers')).tap();
    await expect(element(by.id('add-peer-button'))).toBeVisible();
  });

  it('should open add peer modal', async () => {
    await element(by.id('tab-peers')).tap();
    await element(by.id('add-peer-button')).tap();
    await expect(element(by.id('peer-id-input'))).toBeVisible();
    await expect(element(by.id('multiaddr-input'))).toBeVisible();
  });

  it('should add peer with valid multiaddr', async () => {
    await element(by.id('tab-peers')).tap();
    await element(by.id('add-peer-button')).tap();
    
    await element(by.id('peer-id-input')).typeText('QmTestPeer1234567890123456789012345678901234567890');
    await element(by.id('multiaddr-input')).typeText('/ip4/127.0.0.1/udp/9999/quic-v1/p2p/QmTestPeer1234567890123456789012345678901234567890');
    
    await element(by.id('confirm-add-peer')).tap();
    
    // Modal should close and peer should appear
    await waitFor(element(by.text('QmTestPeer1234567890123456789012345678901234567890')))
      .toBeVisible()
      .withTimeout(2000);
  });
});

