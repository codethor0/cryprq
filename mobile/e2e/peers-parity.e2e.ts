describe('Peers Parity', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  it('should validate multiaddr and disable Add button for bad multiaddr', async () => {
    await element(by.id('tab-peers')).tap();
    await element(by.id('add-peer-button')).tap();
    
    // Enter invalid multiaddr
    await element(by.id('multiaddr-input')).typeText('invalid-multiaddr');
    await element(by.id('peer-id-input')).typeText('QmTest1234567890123456789012345678901234567890');
    
    // Add button should be disabled or show error
    await expect(element(by.id('confirm-add-peer'))).not.toBeEnabled();
  });

  it('should enable Add button for valid multiaddr', async () => {
    await element(by.id('tab-peers')).tap();
    await element(by.id('add-peer-button')).tap();
    
    const validMultiaddr = '/ip4/127.0.0.1/udp/9999/quic-v1/p2p/QmTest1234567890123456789012345678901234567890';
    await element(by.id('multiaddr-input')).typeText(validMultiaddr);
    await element(by.id('peer-id-input')).typeText('QmTest1234567890123456789012345678901234567890');
    
    // Add button should be enabled
    await expect(element(by.id('confirm-add-peer'))).toBeEnabled();
  });

  it('should connect/disconnect peer', async () => {
    await element(by.id('tab-peers')).tap();
    
    // Add peer first
    await element(by.id('add-peer-button')).tap();
    const validMultiaddr = '/ip4/127.0.0.1/udp/9999/quic-v1/p2p/QmTest1234567890123456789012345678901234567890';
    await element(by.id('multiaddr-input')).typeText(validMultiaddr);
    await element(by.id('peer-id-input')).typeText('QmTest1234567890123456789012345678901234567890');
    await element(by.id('confirm-add-peer')).tap();
    
    // Connect
    await element(by.id('connect-QmTest1234567890123456789012345678901234567890')).tap();
    await waitFor(element(by.text('Connected'))).toBeVisible().withTimeout(3000);
    
    // Disconnect
    await element(by.id('disconnect-QmTest1234567890123456789012345678901234567890')).tap();
    await waitFor(element(by.text('Disconnected'))).toBeVisible().withTimeout(2000);
  });

  it('should test reachability and show latency', async () => {
    await element(by.id('tab-peers')).tap();
    
    // Add peer
    await element(by.id('add-peer-button')).tap();
    const validMultiaddr = '/ip4/127.0.0.1/udp/9999/quic-v1/p2p/QmTest1234567890123456789012345678901234567890';
    await element(by.id('multiaddr-input')).typeText(validMultiaddr);
    await element(by.id('peer-id-input')).typeText('QmTest1234567890123456789012345678901234567890');
    await element(by.id('confirm-add-peer')).tap();
    
    // Test reachability
    await element(by.id('test-QmTest1234567890123456789012345678901234567890')).tap();
    
    // Should show latency or success message within 3s
    await waitFor(element(by.text(/ms|Reachable|Unreachable/)))
      .toBeVisible()
      .withTimeout(3000);
  });
});

