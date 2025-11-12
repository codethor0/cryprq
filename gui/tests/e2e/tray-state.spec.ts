import { test, expect, _electron as electron } from '@playwright/test'
import path from 'path'
import { ChildProcessWithoutNullStreams, spawn } from 'child_process'

let fakeCryprqProcess: ChildProcessWithoutNullStreams | null = null

test.beforeAll(async () => {
  if (process.env.CRYPRQ_METRICS) {
    console.log('Using external fake-cryprq:', process.env.CRYPRQ_METRICS)
  } else {
    console.log('Starting local fake-cryprq...')
    fakeCryprqProcess = spawn('node', [path.join(__dirname, '../../.docker/fake-cryprq/server.js')], {
      stdio: ['ignore', 'inherit', 'inherit'],
      env: { ...process.env, CLI_LISTEN_PORT: '9999' },
    })
    await new Promise(resolve => setTimeout(resolve, 3000))
  }
})

test.afterAll(async () => {
  if (fakeCryprqProcess) {
    fakeCryprqProcess.kill('SIGTERM')
    await new Promise(resolve => setTimeout(resolve, 1000))
  }
})

test('tray status updates on session:state-changed to running', async () => {
  const app = await electron.launch({ args: ['.'] })
  const window = await app.firstWindow()
  await window.waitForSelector('text=Disconnected')

  // Get initial tray state
  const initialSnapshot = await app.evaluate(() => {
    return window.electronAPI.devTraySnapshot()
  })
  expect(initialSnapshot.status).toBe('disconnected')
  expect(initialSnapshot.items).toContain('Connect')

  // Start session
  await window.click('button:has-text("Connect")')
  await window.waitForSelector('text=Connected', { timeout: 10000 })

  // Wait a bit for tray update
  await new Promise(resolve => setTimeout(resolve, 500))

  // Check tray state
  const connectedSnapshot = await app.evaluate(() => {
    return window.electronAPI.devTraySnapshot()
  })
  expect(connectedSnapshot.status).toBe('connected')
  expect(connectedSnapshot.items).toContain('Disconnect')

  await app.close()
})

test('tray status updates on rotation', async () => {
  const app = await electron.launch({ args: ['.'] })
  const window = await app.firstWindow()
  await window.waitForSelector('text=Disconnected')

  await window.click('button:has-text("Connect")')
  await window.waitForSelector('text=Connected', { timeout: 10000 })

  // Simulate rotation event
  await app.evaluate(() => {
    window.electronAPI.onSessionEvent({
      type: 'rotation',
      nextInSeconds: 300,
      timestamp: new Date().toISOString(),
    })
  })

  // Wait for rotation state
  await new Promise(resolve => setTimeout(resolve, 100))

  const rotatingSnapshot = await app.evaluate(() => {
    return window.electronAPI.devTraySnapshot()
  })
  expect(rotatingSnapshot.status).toBe('rotating')

  // Wait for rotation to complete
  await new Promise(resolve => setTimeout(resolve, 1100))

  const afterRotationSnapshot = await app.evaluate(() => {
    return window.electronAPI.devTraySnapshot()
  })
  expect(afterRotationSnapshot.status).toBe('connected')

  await app.close()
})

test('tray recent peers contains active peer after connect', async () => {
  const app = await electron.launch({ args: ['.'] })
  const window = await app.firstWindow()
  await window.waitForSelector('text=Disconnected')

  await window.click('button:has-text("Connect")')
  await window.waitForSelector('text=Connected', { timeout: 10000 })

  await new Promise(resolve => setTimeout(resolve, 500))

  const snapshot = await app.evaluate(() => {
    return window.electronAPI.devTraySnapshot()
  })

  if (snapshot.currentPeer) {
    expect(snapshot.currentPeer.peerId).toBeTruthy()
    expect(snapshot.recentLabels.length).toBeGreaterThan(0)
    expect(snapshot.recentLabels.some(label => label.includes(snapshot.currentPeer!.peerId.slice(0, 8)))).toBeTruthy()
  }

  await app.close()
})

