/* ═══════════════════════════════════════════
   ItzamBox — Sprint 16: Image Management E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses, setTauriOverride } from './fixtures/mockDockerResponses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 16: Image Management E2E', () => {
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
  // Scenario 1: Image list shows seeded nginx:alpine and metadata
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: Image list shows seeded nginx:alpine with size and tag', async ({ page }) => {
    await page.goto('/images')
    await waitForApp(page)

    // The row should display the repository name
    const repo = page.locator('text=nginx').first()
    await expect(repo).toBeVisible()

    // The tag chip should display "alpine"
    const tag = page.locator('.port-tag', { hasText: 'alpine' }).first()
    await expect(tag).toBeVisible()

    // The row meta should include the formatted size (e.g. "23 MB")
    // The mock returns 23,000,000 bytes → 23.0 GB? No, formatter is:
    //   b > 1e9 -> GB, else MB → 23,000,000 / 1e6 = 23 MB
    await expect(page.locator('text=23 MB').first()).toBeVisible()

    // Section header should show the count
    await expect(page.locator('text=Local Images (1)').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: Pull Image modal opens and accepts an image name
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: Pull Image modal opens, accepts input and triggers pull', async ({ page }) => {
    await setTauriOverride(page, 'pull_image', { Success: null })

    await page.goto('/images')
    await waitForApp(page)

    // Click the "Pull Image" button in the header
    const pullBtn = page.locator('button:has-text("Pull Image")').first()
    await expect(pullBtn).toBeVisible()
    await pullBtn.click()

    // Modal should appear with an image-name input
    const imageInput = page.locator('input[placeholder*="nginx"]').first()
    await expect(imageInput).toBeVisible()

    // Type a custom image name
    await imageInput.fill('redis:7-alpine')

    // Click the modal "Pull" button
    const modalPull = page.locator('.modal-footer button:has-text("Pull")').first()
    await expect(modalPull).toBeVisible()
    await modalPull.click()

    // Modal should close and the image list should still be visible
    await page.waitForTimeout(300)
    await expect(page.locator('text=nginx').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: Remove image — click trash icon, image stays (mock
  //             list returns the same items) and no error is shown
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: Click remove on an image invokes remove_image and does not error', async ({ page }) => {
    await setTauriOverride(page, 'remove_image', [
      { Untagged: ['sha256:abcd1234'] },
    ])

    await page.goto('/images')
    await waitForApp(page)

    // The trash icon button on the row
    const removeBtn = page.locator('button[title="Remove"]').first()
    await expect(removeBtn).toBeVisible()
    await removeBtn.click()

    // The list should re-render and the image name should still be visible
    // (the mock doesn't actually remove it from the list, but the command
    //  is invoked without error)
    await page.waitForTimeout(300)
    await expect(page.locator('text=nginx').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: Pull modal can be cancelled
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: Pull Image modal can be cancelled', async ({ page }) => {
    await page.goto('/images')
    await waitForApp(page)

    // Open modal
    await page.locator('button:has-text("Pull Image")').first().click()

    // Modal is visible
    const modal = page.locator('.modal-content').first()
    await expect(modal).toBeVisible()

    // Click cancel
    await page.locator('.modal-footer button:has-text("Cancel")').first().click()

    // Modal should be gone
    await expect(modal).toBeHidden()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: View Layers button navigates to /images/<id>/layers
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: View layers button navigates to the image layers view', async ({ page }) => {
    await setTauriOverride(page, 'get_image_history', [
      { digest: 'sha256:1111', size_bytes: 5_000_000, command: 'ADD file:...', created_at: 1718320000 },
      { digest: 'sha256:2222', size_bytes: 18_000_000, command: 'CMD ["nginx"]', created_at: 1718320000 },
    ])

    await page.goto('/images')
    await waitForApp(page)

    // The first "View layers" button
    const viewLayersBtn = page.locator('button[title="View layers"]').first()
    await expect(viewLayersBtn).toBeVisible()
    await viewLayersBtn.click()

    await page.waitForURL(/\/images\/.+\/layers$/, { timeout: 5000 })
    expect(page.url()).toMatch(/\/images\/.+\/layers$/)

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 6: Clicking the image row itself also navigates to layers
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 6: Clicking the image row navigates to its layers view', async ({ page }) => {
    await page.goto('/images')
    await waitForApp(page)

    await page.locator('text=nginx').first().click()
    await page.waitForURL(/\/images\/.+\/layers$/, { timeout: 5000 })
    expect(page.url()).toMatch(/\/images\/.+\/layers$/)

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
