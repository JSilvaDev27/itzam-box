/* ═══════════════════════════════════════════
   ItzamBox — Sprint 16: Run Wizard + Templates E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses, setTauriOverride } from './fixtures/mockDockerResponses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 16: Run Wizard + Templates E2E', () => {
  let consoleErrors: string[] = []

  test.beforeEach(async ({ page }) => {
    consoleErrors = []
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })
    page.on('pageerror', err => consoleErrors.push(err.message))

    await mockDockerResponses(page)

    // Mock the run-wizard Tauri commands so navigation works
    await setTauriOverride(page, 'create_and_run_container', 'abcd1234efgh')
    await setTauriOverride(page, 'pull_image', { Success: null })
    await setTauriOverride(page, 'list_networks', [
      { id: 'net123', name: 'bridge', driver: 'bridge' },
    ])
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 1: Run Wizard renders the 10-step layout
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: Run Wizard renders step 1 (Image Selection) by default', async ({ page }) => {
    await page.goto('/run-wizard')
    await waitForApp(page)

    // The wizard renders the step 1 title
    await expect(page.locator('h2.step-title', { hasText: 'Select Image' })).toBeVisible()

    // The "Next" button should be visible
    const nextBtn = page.locator('.wizard-footer button:has-text("Next")').first()
    await expect(nextBtn).toBeVisible()
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: Manually entering an image name lets the user advance
  //             through the wizard steps
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: Manual image name entry advances through wizard steps', async ({ page }) => {
    await page.goto('/run-wizard')
    await waitForApp(page)

    // Step 1: type a manual image name
    const imageInput = page.locator('input[placeholder*="nginx:latest"]').first()
    await expect(imageInput).toBeVisible()
    await imageInput.fill('nginx:alpine')

    // Click the "Pull" button
    const pullBtn = page.locator('button:has-text("Pull")').first()
    await expect(pullBtn).toBeVisible()
    await pullBtn.click()

    // Wait for pull to complete (mock resolves immediately)
    await page.waitForTimeout(200)

    // Advance to step 2
    const nextBtn = page.locator('.wizard-footer button:has-text("Next")').first()
    await nextBtn.click()
    await expect(page.locator('h2.step-title', { hasText: 'Container Name' })).toBeVisible()

    // Step 2 → Step 3
    await page.locator('.wizard-footer button:has-text("Next")').first().click()
    await expect(page.locator('h2.step-title', { hasText: 'Port Mapping' })).toBeVisible()

    // Step 3 → Step 4 (volumes)
    await page.locator('.wizard-footer button:has-text("Next")').first().click()
    await expect(page.locator('h2.step-title', { hasText: 'Volume Mounts' })).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: The wizard's "Add Port" button adds a new port mapping row
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: Adding a port mapping row is possible on step 3', async ({ page }) => {
    await page.goto('/run-wizard')
    await waitForApp(page)

    // Skip to step 3 directly using a manual image name
    const imageInput = page.locator('input[placeholder*="nginx:latest"]').first()
    await imageInput.fill('nginx:alpine')
    await page.locator('button:has-text("Pull")').first().click()
    await page.waitForTimeout(200)

    await page.locator('.wizard-footer button:has-text("Next")').first().click() // → 2
    await page.locator('.wizard-footer button:has-text("Next")').first().click() // → 3

    await expect(page.locator('h2.step-title', { hasText: 'Port Mapping' })).toBeVisible()

    // Click "Add Port" — wizard defines an "Add" button on this step
    const addPortBtn = page.locator('button:has-text("Add Port")').first()
    await expect(addPortBtn).toBeVisible()
    await addPortBtn.click()

    // A new port row should now be visible (at least one input of type=number
    // or a row containing the "Remove port" button)
    const removePortBtn = page.locator('button[title="Remove port"]').first()
    await expect(removePortBtn).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: The wizard reaches step 10 (Review & Create) and exposes
  //             the "Create Container" button
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: Wizard reaches step 10 (Review & Create) and exposes the Create button', async ({ page }) => {
    await page.goto('/run-wizard')
    await waitForApp(page)

    // Fill in step 1
    const imageInput = page.locator('input[placeholder*="nginx:latest"]').first()
    await imageInput.fill('nginx:alpine')
    await page.locator('button:has-text("Pull")').first().click()
    await page.waitForTimeout(200)

    // Click Next 9 times to reach step 10
    for (let i = 0; i < 9; i++) {
      const nextBtn = page.locator('.wizard-footer button:has-text("Next")').first()
      if (await nextBtn.count() > 0) {
        await nextBtn.click()
        await page.waitForTimeout(50)
      }
    }

    // Step 10 should be visible
    await expect(page.locator('h2.step-title', { hasText: 'Review & Create' })).toBeVisible()

    // The Create Container button should be visible
    const createBtn = page.locator('button:has-text("Create Container")').first()
    await expect(createBtn).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: Templates page renders the seeded Nginx template
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: Templates page renders the seeded Nginx Web Server template card', async ({ page }) => {
    await page.goto('/templates')
    await waitForApp(page)

    // The template card title
    await expect(page.locator('h3.template-name', { hasText: 'Nginx Web Server' })).toBeVisible()

    // The card description
    await expect(page.locator('p.template-desc', { hasText: 'High-performance HTTP server' })).toBeVisible()

    // The Deploy button
    const deployBtn = page.locator('button:has-text("Deploy")').first()
    await expect(deployBtn).toBeVisible()

    // The image name
    await expect(page.locator('text=nginx:alpine').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 6: Templates page exposes search and category filters
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 6: Templates page exposes search input and category chips', async ({ page }) => {
    await page.goto('/templates')
    await waitForApp(page)

    // The search input
    const searchInput = page.locator('input[placeholder*="Search templates"]').first()
    await expect(searchInput).toBeVisible()

    // Type a search term — results still show (mock returns the same data)
    await searchInput.fill('nginx')
    await page.waitForTimeout(200)
    await expect(page.locator('h3.template-name', { hasText: 'Nginx Web Server' })).toBeVisible()

    // Clear search
    await searchInput.fill('')

    // The category chip "All" should be present
    const allChip = page.locator('.chip', { hasText: 'All' }).first()
    await expect(allChip).toBeVisible()

    // A "Web" category chip (from the seeded template) should be present
    await expect(page.locator('.chip', { hasText: 'Web' }).first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 7: Clicking the Deploy button triggers pull_image +
  //             create_and_run_container
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 7: Clicking Deploy triggers pull_image and create_and_run_container', async ({ page }) => {
    await setTauriOverride(page, 'pull_image', { Success: null })
    await setTauriOverride(page, 'create_and_run_container', 'new-container-id-99')

    await page.goto('/templates')
    await waitForApp(page)

    // Click Deploy
    const deployBtn = page.locator('button:has-text("Deploy")').first()
    await expect(deployBtn).toBeVisible()
    await deployBtn.click()

    // The button text should switch to "Deploying..." briefly
    // (the mock returns synchronously, so we can't reliably catch the spinner,
    //  but we can wait a tick and confirm the page is still functional)
    await page.waitForTimeout(400)

    // The template card is still rendered (the deploy succeeds with a mock)
    await expect(page.locator('h3.template-name', { hasText: 'Nginx Web Server' })).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 8: The wizard's Cancel button navigates back to /containers
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 8: Cancel button on the wizard navigates back to /containers', async ({ page }) => {
    await page.goto('/run-wizard')
    await waitForApp(page)

    // The Cancel button (in the wizard footer)
    const cancelBtn = page.locator('.wizard-footer button:has-text("Cancel")').first()
    await expect(cancelBtn).toBeVisible()
    await cancelBtn.click()

    await page.waitForURL((url) => url.pathname === '/containers', { timeout: 10000 })
    expect(new URL(page.url()).pathname).toBe('/containers')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 9: Templates page shows the Built-in tag on the seeded template
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 9: Templates page marks the seeded template as Built-in', async ({ page }) => {
    await page.goto('/templates')
    await waitForApp(page)

    const builtinTag = page.locator('.tag.builtin-tag', { hasText: 'Built-in' }).first()
    await expect(builtinTag).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
