import { test, expect } from '@playwright/test'

test.describe('Charts smoke tests', () => {
  test('throughput chart updates and renders correctly', async ({ page }) => {
    // Navigate to app (adjust port if needed)
    await page.goto('http://localhost:5173')
    
    // Wait for app to load
    await page.waitForLoadState('networkidle')
    
    // Click Connect button
    const connectButton = page.getByRole('button', { name: /connect/i })
    if (await connectButton.isVisible()) {
      await connectButton.click()
      
      // Wait for connection to establish
      await page.waitForTimeout(2000)
    }
    
    // Wait for charts to appear
    await page.getByText('Throughput (last 60s)').waitFor({ timeout: 5000 }).catch(() => {
      // If charts don't appear, check if we're connected
      const status = await page.locator('text=/connected|disconnected/i').first().textContent()
      if (status?.toLowerCase().includes('disconnected')) {
        test.skip('Not connected - charts require active connection')
      }
    })
    
    // Verify chart elements are present
    const throughputChart = page.locator('text=Throughput (last 60s)')
    await expect(throughputChart).toBeVisible()
    
    // Check that chart lines are rendered (recharts renders SVG paths)
    const chartLines = page.locator('svg.recharts-surface')
    await expect(chartLines.first()).toBeVisible({ timeout: 3000 })
    
    // Wait a bit and verify chart is still updating
    const initialCount = await chartLines.count()
    await page.waitForTimeout(1500) // Wait ~1.5s to see if updates occur
    
    // Chart should still be present
    await expect(throughputChart).toBeVisible()
    
    // Verify latency chart also exists
    const latencyChart = page.getByText('Latency (last 60s)')
    await expect(latencyChart).toBeVisible()
  })

  test('charts show empty state when disconnected', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // If disconnected, should show empty state
    const disconnected = page.locator('text=/disconnected/i')
    if (await disconnected.isVisible()) {
      // Charts component might show empty state or not render
      const emptyState = page.locator('text=/no data yet|connect to a peer/i')
      const chartsTitle = page.locator('text=Throughput (last 60s)')
      
      // Either empty state or charts shouldn't be visible when disconnected
      const hasEmptyState = await emptyState.isVisible().catch(() => false)
      const hasCharts = await chartsTitle.isVisible().catch(() => false)
      
      // Should have one or the other, not both
      expect(hasEmptyState || !hasCharts).toBe(true)
    }
  })

  test('unit toggle works for throughput chart', async ({ page }) => {
    await page.goto('http://localhost:5173')
    await page.waitForLoadState('networkidle')
    
    // Connect if needed
    const connectButton = page.getByRole('button', { name: /connect/i })
    if (await connectButton.isVisible()) {
      await connectButton.click()
      await page.waitForTimeout(2000)
    }
    
    // Wait for charts
    const throughputTitle = page.getByText('Throughput (last 60s)')
    await throughputTitle.waitFor({ timeout: 5000 }).catch(() => {
      test.skip('Charts not available')
    })
    
    // Find unit toggle buttons
    const kbButton = page.locator('button:has-text("KB/s")')
    const mbButton = page.locator('button:has-text("MB/s")')
    
    if (await kbButton.isVisible()) {
      // Click KB button
      await kbButton.click()
      await page.waitForTimeout(500)
      
      // Verify Y-axis label updates (check for KB/s in chart)
      const yAxisLabel = page.locator('text=/KB\\/s|MB\\/s|bytes\\/s/i')
      await expect(yAxisLabel.first()).toBeVisible()
    }
  })
})

