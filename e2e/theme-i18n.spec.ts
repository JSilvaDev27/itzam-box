/* ═══════════════════════════════════════════
   ItzamBox — Sprint 16: Theme Toggle & i18n E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses, setTauriOverride } from './fixtures/mockDockerResponses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 16: Theme Toggle & i18n E2E', () => {
  let consoleErrors: string[] = []

  test.beforeEach(async ({ page }) => {
    consoleErrors = []
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })
    page.on('pageerror', err => consoleErrors.push(err.message))

    await mockDockerResponses(page)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 1: Theme attribute is set on <html> on initial load
  //             (the useTheme composable applies data-theme to
  //             documentElement, not body)
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: Initial theme is dark (data-theme=dark on <html>)', async ({ page }) => {
    await page.goto('/')
    await waitForApp(page)

    // The mock returns 'dark' for get_config({key:'theme'})
    const theme = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
    expect(theme).toBe('dark')
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: Clicking the toggle theme button flips the theme
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: Clicking the theme toggle switches between dark and light', async ({ page }) => {
    await page.goto('/')
    await waitForApp(page)

    const themeBtn = page.locator('button[title="Toggle theme"]').first()
    await expect(themeBtn).toBeVisible()

    // Click once → light
    await themeBtn.click()
    await page.waitForTimeout(150)
    const afterClick1 = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
    expect(afterClick1).toBe('light')

    // Click again → dark
    await themeBtn.click()
    await page.waitForTimeout(150)
    const afterClick2 = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
    expect(afterClick2).toBe('dark')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: Settings page exposes theme selection cards (dark / light)
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: Settings page exposes Dark and Light theme cards', async ({ page }) => {
    await page.goto('/settings')
    await waitForApp(page)

    // The Settings view renders two clickable theme cards
    await expect(page.locator('text=Dark').first()).toBeVisible()
    await expect(page.locator('text=Light').first()).toBeVisible()

    // Click the Light card → theme becomes light
    const lightCard = page.locator('div:has-text("Light")').first()
    await lightCard.click()
    await page.waitForTimeout(150)
    const theme = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
    expect(theme).toBe('light')

    // Toggle back to dark
    const darkCard = page.locator('div:has-text("Dark")').first()
    await darkCard.click()
    await page.waitForTimeout(150)
    const theme2 = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
    expect(theme2).toBe('dark')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: i18n — switch from ES to EN and verify a key label
  //              changes (the Settings view re-renders t.settings.* keys)
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: Switching to English changes the Settings page title', async ({ page }) => {
    // Mock returns lang = 'es' by default → Settings should show "Configuración"
    await page.goto('/settings')
    await waitForApp(page)

    // The H1 should read "Configuración" in Spanish
    await expect(page.locator('h1', { hasText: 'Configuración' })).toBeVisible()

    // Click the English locale card via stable data-testid
    await page.getByTestId('locale-en').click()

    // Allow Vue to re-render
    await page.waitForTimeout(200)

    // Now the H1 should be "Settings"
    await expect(page.locator('h1', { hasText: 'Settings' })).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: i18n — switch back from EN to ES
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: Switching back to Spanish restores the Configuración label', async ({ page }) => {
    await page.goto('/settings')
    await waitForApp(page)

    // Switch to EN first via stable data-testid
    await page.getByTestId('locale-en').click()
    await page.waitForTimeout(200)
    await expect(page.locator('h1', { hasText: 'Settings' })).toBeVisible()

    // Now switch back to ES via stable data-testid
    await page.getByTestId('locale-es').click()
    await page.waitForTimeout(200)
    await expect(page.locator('h1', { hasText: 'Configuración' })).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 6: Initial lang is 'es' (mock returns 'es' for get_config)
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 6: Default language is Spanish', async ({ page }) => {
    await page.goto('/settings')
    await waitForApp(page)

    // The H1 in ES is "Configuración"
    await expect(page.locator('h1.text-h1', { hasText: 'Configuración' })).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
