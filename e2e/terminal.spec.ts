/* ═══════════════════════════════════════════
   ItzamBox — Sprint 16: Host Terminal E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses, setTauriOverride } from './fixtures/mockDockerResponses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 16: Host Terminal E2E', () => {
  let consoleErrors: string[] = []

  test.beforeEach(async ({ page }) => {
    consoleErrors = []
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })
    page.on('pageerror', err => consoleErrors.push(err.message))

    await mockDockerResponses(page)

    // Mock the Tauri spawn/listen APIs that the TerminalPanel uses.
    // The real implementation calls `spawn_host_terminal` and then listens
    // to `host-terminal-output` events. We provide a deterministic mock that
    // echoes any input back to the terminal.
    await setTauriOverride(page, 'spawn_host_terminal', 'mock-session-id')
    await setTauriOverride(page, 'write_to_host_terminal', { Success: null })
    await setTauriOverride(page, 'resize_host_terminal', null)
    await setTauriOverride(page, 'kill_host_terminal', null)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 1: Terminal-min label is visible on every page
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: Terminal-min label is visible on the dashboard', async ({ page }) => {
    await page.goto('/')
    await waitForApp(page)

    const minLabel = page.locator('.terminal-min-label').first()
    await expect(minLabel).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: Clicking the terminal-min label expands the terminal
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: Clicking the terminal-min label opens the expanded terminal panel', async ({ page }) => {
    await page.goto('/')
    await waitForApp(page)

    const minLabel = page.locator('.terminal-min-label').first()
    await expect(minLabel).toBeVisible()
    await minLabel.click()

    // After click, the expanded panel should be visible
    const expanded = page.locator('.terminal-expanded').first()
    await expect(expanded).toBeVisible()

    // The xterm-host element should be present (xterm.js renders into .xterm)
    const xtermHost = page.locator('.terminal-expanded .xterm').first()
    await expect(xtermHost).toBeVisible({ timeout: 5000 })

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: The expanded terminal exposes a minimize button
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: The expanded terminal has a minimize/close button', async ({ page }) => {
    await page.goto('/')
    await waitForApp(page)

    await page.locator('.terminal-min-label').first().click()
    await expect(page.locator('.terminal-expanded').first()).toBeVisible()

    // The minimize button (with the chevron-down icon)
    const minimizeBtn = page.locator('.terminal-tabs-minimize').first()
    await expect(minimizeBtn).toBeVisible()

    // Clicking minimize should hide the expanded panel
    await minimizeBtn.click()
    await expect(page.locator('.terminal-expanded')).toBeHidden()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: The terminal "Add Tab" dropdown is available in the
  //             expanded panel and lets the user open a new Host tab
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: The +Add tab button opens a dropdown to create new terminals', async ({ page }) => {
    await page.goto('/')
    await waitForApp(page)

    await page.locator('.terminal-min-label').first().click()
    await expect(page.locator('.terminal-expanded').first()).toBeVisible()

    // Click the add-tab button
    const addBtn = page.locator('.terminal-tab-add-btn').first()
    await expect(addBtn).toBeVisible()
    await addBtn.click()

    // The dropdown should appear
    const dropdown = page.locator('.terminal-dropdown').first()
    await expect(dropdown).toBeVisible()

    // The dropdown contains a "Host Terminal" option
    await expect(page.locator('text=Host Terminal').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: The terminal tabs bar shows the default "Host" tab
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: The terminal tabs bar shows the default Host tab', async ({ page }) => {
    await page.goto('/')
    await waitForApp(page)

    await page.locator('.terminal-min-label').first().click()
    await expect(page.locator('.terminal-expanded').first()).toBeVisible()

    // The Host tab should be visible
    const hostTab = page.locator('.terminal-tab-name', { hasText: 'Host' }).first()
    await expect(hostTab).toBeVisible()

    // The tab should have the --active modifier
    await expect(page.locator('.terminal-tab--active', { hasText: 'Host' }).first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
