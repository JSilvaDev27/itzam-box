/* ═══════════════════════════════════════════
   ItzamBox — Sprint 22: Metrics & Historical Charts E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   Task: T-080 — Verify sub-tabs, time range pills, SVG chart + tooltip, export dropdown.
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses } from './fixtures/mockDockerResponses'
import { mockV12Responses } from './fixtures/mockV12Responses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 22: Metrics & Historical Charts E2E (T-080)', () => {
  let consoleErrors: string[] = []

  test.beforeEach(async ({ page }) => {
    consoleErrors = []
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })
    page.on('pageerror', err => consoleErrors.push(err.message))

    await mockDockerResponses(page)
    await mockV12Responses(page)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 1: /metrics renders CPU/Memory/Network/Disk sub-tabs
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: CPU, Memory, Network, and Disk sub-tabs are present and switchable', async ({ page }) => {
    await page.goto('/metrics')
    await waitForApp(page)

    // The page header is visible
    await expect(page.locator('.text-h1').first()).toContainText('Metrics')

    // 4 sub-tabs
    const tabs = page.locator('.tabs__item')
    await expect(tabs).toHaveCount(4)
    await expect(tabs.nth(0)).toContainText('CPU')
    await expect(tabs.nth(1)).toContainText('Memory')
    await expect(tabs.nth(2)).toContainText('Network')
    await expect(tabs.nth(3)).toContainText('Disk')

    // CPU is active by default
    await expect(tabs.nth(0)).toHaveClass(/active/)

    // Click Memory tab — URL query updates to ?metric=memory
    await tabs.nth(1).click()
    await expect(tabs.nth(1)).toHaveClass(/active/)
    expect(new URL(page.url()).search).toContain('metric=memory')

    // Click Network tab
    await tabs.nth(2).click()
    await expect(tabs.nth(2)).toHaveClass(/active/)
    expect(new URL(page.url()).search).toContain('metric=network')

    // Click Disk tab
    await tabs.nth(3).click()
    await expect(tabs.nth(3)).toHaveClass(/active/)
    expect(new URL(page.url()).search).toContain('metric=disk')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: TimeRangeSelector exposes 1h/6h/24h/7d/30d pills
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: Time range selector exposes 1h, 6h, 24h, 7d, 30d pills', async ({ page }) => {
    await page.goto('/metrics')
    await waitForApp(page)

    // The time range selector is rendered
    const rangeGroup = page.locator('.time-range')
    await expect(rangeGroup).toBeVisible()
    await expect(rangeGroup).toHaveAttribute('aria-label', 'Time range selector')

    // 5 pills
    const pills = page.locator('.time-range__btn')
    await expect(pills).toHaveCount(5)
    await expect(pills.nth(0)).toContainText('1h')
    await expect(pills.nth(1)).toContainText('6h')
    await expect(pills.nth(2)).toContainText('24h')
    await expect(pills.nth(3)).toContainText('7d')
    await expect(pills.nth(4)).toContainText('30d')

    // Aria labels
    await expect(pills.nth(0)).toHaveAttribute('aria-label', '1 hour')
    await expect(pills.nth(2)).toHaveAttribute('aria-label', '24 hours')
    await expect(pills.nth(4)).toHaveAttribute('aria-label', '30 days')

    // 1h is active by default
    await expect(pills.nth(0)).toHaveClass(/active/)

    // Click 24h pill
    await pills.nth(2).click()
    await expect(pills.nth(2)).toHaveClass(/active/)
    await expect(pills.nth(0)).not.toHaveClass(/active/)
    await expect(pills.nth(2)).toHaveAttribute('aria-pressed', 'true')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: SVG chart renders with data and tooltip on hover
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: SVG chart renders with data lines and hover tooltip', async ({ page }) => {
    await page.goto('/metrics')
    await waitForApp(page)

    // The ChartCard is rendered
    const chartContainer = page.locator('.chart-container').first()
    await expect(chartContainer).toBeVisible()

    // The SVG element with class .chart-svg is rendered
    const svg = chartContainer.locator('svg.chart-svg')
    await expect(svg).toBeVisible()

    // The chart lines are rendered (mock provides 60 data points → paths exist)
    const linePaths = chartContainer.locator('path.chart-line')
    expect(await linePaths.count()).toBeGreaterThan(0)

    // The area fill path is also rendered
    const areaPaths = chartContainer.locator('path.chart-area-fill')
    expect(await areaPaths.count()).toBeGreaterThan(0)

    // Hover over the chart to trigger the tooltip
    const svgBox = await svg.boundingBox()
    expect(svgBox).not.toBeNull()
    if (svgBox) {
      // Move the mouse to the middle of the chart
      await page.mouse.move(svgBox.x + svgBox.width / 2, svgBox.y + svgBox.height / 2)
      // Wait for the tooltip to appear
      const tooltip = page.locator('.chart-tooltip').first()
      await expect(tooltip).toBeVisible({ timeout: 3000 })
      // Tooltip has a time + value
      await expect(tooltip.locator('.chart-tooltip__time')).toBeVisible()
      await expect(tooltip.locator('.chart-tooltip__value').first()).toBeVisible()
      // Value must not be empty / NaN
      const value = await tooltip.locator('.chart-tooltip__value').first().innerText()
      expect(value).not.toMatch(/NaN|undefined/)
    }

    // Summary cards are visible
    const summaryGrid = page.locator('.metrics-summary-grid')
    await expect(summaryGrid).toBeVisible()
    await expect(summaryGrid.locator('.metric-card')).toHaveCount(4)

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: Export dropdown shows CSV, JSON, PNG options
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: Export dropdown exposes CSV, JSON, and PNG options', async ({ page }) => {
    await page.goto('/metrics')
    await waitForApp(page)

    // The chart toolbar is rendered
    await expect(page.locator('.chart-toolbar')).toBeVisible()

    // The Export button is rendered
    const exportBtn = page.locator('.export-wrapper > button', { hasText: 'Export' })
    await expect(exportBtn).toBeVisible()
    await expect(exportBtn).toHaveAttribute('aria-label', 'Export metrics')
    await expect(exportBtn).toHaveAttribute('aria-expanded', 'false')

    // Click the export button to open the dropdown
    await exportBtn.click()
    await expect(exportBtn).toHaveAttribute('aria-expanded', 'true')

    // The export menu is rendered with 3 items
    const exportMenu = page.locator('.export-menu[role="menu"]').first()
    await expect(exportMenu).toBeVisible()
    await expect(exportMenu).toHaveAttribute('aria-label', 'Export options')

    const items = exportMenu.locator('.export-menu__item')
    await expect(items).toHaveCount(3)
    await expect(items.nth(0)).toContainText('Export as CSV')
    await expect(items.nth(1)).toContainText('Export as JSON')
    await expect(items.nth(2)).toContainText('Save as PNG')

    // All items are role="menuitem"
    for (let i = 0; i < 3; i++) {
      await expect(items.nth(i)).toHaveAttribute('role', 'menuitem')
    }

    // Click an item — the menu should close
    await items.nth(0).click()
    await expect(exportMenu).toBeHidden()

    // Reopen the menu and verify it still works
    await exportBtn.click()
    await expect(exportMenu).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
