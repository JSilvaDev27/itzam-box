/* ═══════════════════════════════════════════
   ItzamBox — Sprint 16: Edge Cases (Error, Empty, Export/Import, Events) E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses, setTauriOverride } from './fixtures/mockDockerResponses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 16: Edge Cases E2E', () => {
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
  // Scenario 1: ErrorState renders when list_containers throws
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: ErrorState renders on the Containers view when list_containers throws', async ({ page }) => {
    // Override list_containers to throw
    await setTauriOverride(page, 'list_containers', { error: 'Cannot connect to Docker daemon' })

    await page.goto('/containers')
    await waitForApp(page)

    // The ErrorState component should render
    const errorState = page.locator('.error-state').first()
    await expect(errorState).toBeVisible()

    // The error message title
    await expect(page.locator('.error-state-title', { hasText: 'Error loading containers' })).toBeVisible()

    // A Retry button is exposed via the .error-state-actions
    const retryBtn = page.locator('.error-state-actions button').first()
    await expect(retryBtn).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: EmptyState renders when list_containers returns []
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: EmptyState renders on the Containers view when list_containers returns []', async ({ page }) => {
    await setTauriOverride(page, 'list_containers', [])

    await page.goto('/containers')
    await waitForApp(page)

    // The EmptyState component should render
    const emptyState = page.locator('.empty-state').first()
    await expect(emptyState).toBeVisible()

    // The title from the empty state
    await expect(page.locator('.empty-state-title', { hasText: 'No containers found' })).toBeVisible()

    // The description
    await expect(page.locator('.empty-state-desc').first()).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: ErrorState renders on Dashboard when check_docker_installed
  //             throws
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: ErrorState renders on Dashboard when check_docker_installed throws', async ({ page }) => {
    await setTauriOverride(page, 'check_docker_installed', { error: 'Docker daemon is not running' })
    await setTauriOverride(page, 'list_containers', { error: 'Cannot list containers' })

    await page.goto('/')
    await waitForApp(page)

    // The dashboard renders its own error UI when the host is unreachable
    // (look for the "ErrorState" component on the page)
    const errorState = page.locator('.error-state').first()
    await expect(errorState).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: EmptyState renders on the Images view when list_images returns []
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: EmptyState renders on Images view when list_images returns []', async ({ page }) => {
    await setTauriOverride(page, 'list_images', [])

    await page.goto('/images')
    await waitForApp(page)

    const emptyState = page.locator('.empty-state').first()
    await expect(emptyState).toBeVisible()

    await expect(page.locator('.empty-state-title', { hasText: 'No images found' })).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: Export/Import view renders the Export tab with an
  //             "Export Container" action card
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: Export/Import view renders the Export tab and its action card', async ({ page }) => {
    await page.goto('/export-import')
    await waitForApp(page)

    // The H1 should be visible
    await expect(page.locator('h1.text-h1', { hasText: 'Export / Import' })).toBeVisible()

    // The Export tab is active by default
    await expect(page.locator('.tab.active', { hasText: 'Export' })).toBeVisible()

    // The "Export Container" action card title
    await expect(page.locator('h2.action-card-title', { hasText: 'Export Container' })).toBeVisible()

    // The primary "Export" button (in the export container card)
    const exportBtn = page.locator('button:has-text("Export")').first()
    await expect(exportBtn).toBeVisible()

    // The Import tab is also visible
    await expect(page.locator('.tab', { hasText: 'Import' })).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 6: Switching to the Import tab shows the import flow
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 6: Clicking the Import tab switches to the import flow', async ({ page }) => {
    await page.goto('/export-import')
    await waitForApp(page)

    // Click the Import tab
    await page.locator('.tab:has-text("Import")').first().click()

    // The Import tab should now be active
    await expect(page.locator('.tab.active', { hasText: 'Import' })).toBeVisible()

    // The import section is rendered (the view exposes a "Load Image" form)
    await expect(page.locator('text=Load Image').first()).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 7: Events view renders the docker events timeline/empty state
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 7: Events view renders the docker events header and start-stream button', async ({ page }) => {
    await page.goto('/events')
    await waitForApp(page)

    // The H1 is "Docker Events"
    await expect(page.locator('h1.text-h1', { hasText: 'Docker Events' })).toBeVisible()

    // The empty state (since we don't auto-start the stream in the test)
    // OR the events-header is visible regardless
    await expect(page.locator('.events-header').first()).toBeVisible()

    // The "Start Stream" button should be visible (the empty state also
    //  exposes a "Start Stream" action)
    const startStreamBtns = page.locator('button:has-text("Start Stream")')
    await expect(startStreamBtns.first()).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 8: Events view shows the stream indicator
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 8: Events view exposes a stream status indicator', async ({ page }) => {
    await page.goto('/events')
    await waitForApp(page)

    // The events-header-right contains a "Stopped" or "Live" status indicator
    const streamIndicator = page.locator('.stream-indicator').first()
    await expect(streamIndicator).toBeVisible()

    // Initial state should be "Stopped"
    await expect(page.locator('.stream-status', { hasText: 'Stopped' })).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 9: Starting the event stream flips the status to "Live"
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 9: Clicking Start Stream flips the status indicator to Live', async ({ page }) => {
    await page.goto('/events')
    await waitForApp(page)

    // Click the "Start Stream" button in the header
    await page.locator('.events-header-right button:has-text("Start Stream")').first().click()

    // The status indicator should now read "Live"
    await expect(page.locator('.stream-status', { hasText: 'Live' })).toBeVisible({ timeout: 5000 })

    // A "Stop Stream" button should be visible
    await expect(page.locator('button:has-text("Stop Stream")').first()).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 10: Export/Import view handles an empty containers list
  //              gracefully (empty state should appear)
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 10: Export/Import view handles an empty containers list gracefully', async ({ page }) => {
    await setTauriOverride(page, 'list_containers', [])
    await setTauriOverride(page, 'list_images', [])

    await page.goto('/export-import')
    await waitForApp(page)

    // The H1 should still be visible
    await expect(page.locator('h1.text-h1', { hasText: 'Export / Import' })).toBeVisible()

    // The Export tab is still functional
    await expect(page.locator('.tab.active', { hasText: 'Export' })).toBeVisible()
  })
})
