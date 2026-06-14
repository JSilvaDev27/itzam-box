/* ═══════════════════════════════════════════
   ItzamBox — Sprint 20: Backup & Restore E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   Task: T-074 — Verify history/scheduled tabs, summary card, create/restore modals, progress bar.
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses } from './fixtures/mockDockerResponses'
import { mockV12Responses, setV12Override } from './fixtures/mockV12Responses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 20: Backup & Restore E2E (T-074)', () => {
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
  // Scenario 1: /backup renders History and Scheduled tabs
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: History and Scheduled tabs are present and switchable', async ({ page }) => {
    await page.goto('/backup')
    await waitForApp(page)

    // Page header is visible
    await expect(page.locator('.text-h1').first()).toContainText('Backup & Restore')

    // Two tabs: History + Scheduled
    const tabs = page.locator('.tabs__item')
    await expect(tabs).toHaveCount(2)
    await expect(tabs.nth(0)).toContainText('History')
    await expect(tabs.nth(1)).toContainText('Scheduled')

    // History is active by default
    await expect(tabs.nth(0)).toHaveClass(/active/)

    // History table is rendered (the BackupHistoryTable component)
    await expect(page.locator('.table-toolbar__title').first()).toContainText('Backup History')
    await expect(page.locator('table[aria-label="Backup History"]')).toBeVisible()

    // Click Scheduled tab
    await tabs.nth(1).click()
    await expect(tabs.nth(1)).toHaveClass(/active/)
    await expect(tabs.nth(0)).not.toHaveClass(/active/)

    // Scheduled section is rendered
    await expect(page.locator('.section-title', { hasText: 'Scheduled Backup Jobs' })).toBeVisible()
    // The mock provides 2 backup jobs
    await expect(page.locator('.data-row')).toHaveCount(2)
    await expect(page.locator('text=Nightly DB Backup')).toBeVisible()
    await expect(page.locator('text=Hourly Config Snapshot')).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: BackupSummaryCard shows 4 metric tiles
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: BackupSummaryCard renders 4 metric tiles', async ({ page }) => {
    await page.goto('/backup')
    await waitForApp(page)

    // The summary grid is rendered
    const summary = page.locator('.backup-summary-grid')
    await expect(summary).toBeVisible()

    // 4 metric cards
    const cards = page.locator('.backup-summary-card')
    await expect(cards).toHaveCount(4)

    // Verify the 4 labels
    await expect(page.locator('.backup-summary-card__label', { hasText: 'Total Backups' })).toBeVisible()
    await expect(page.locator('.backup-summary-card__label', { hasText: 'Last Backup' })).toBeVisible()
    await expect(page.locator('.backup-summary-card__label', { hasText: 'Total Size' })).toBeVisible()
    await expect(page.locator('.backup-summary-card__label', { hasText: 'Scheduled Jobs' })).toBeVisible()

    // The mock has 12 total backups and 2 active jobs
    await expect(page.locator('.backup-summary-card', { hasText: 'Total Backups' }).locator('.backup-summary-card__value')).toContainText('12')
    await expect(page.locator('.backup-summary-card', { hasText: 'Scheduled Jobs' }).locator('.backup-summary-card__value')).toContainText('2')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: Create Snapshot modal — volume selector + destination input
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: Create Snapshot modal exposes volume selector and destination', async ({ page }) => {
    await page.goto('/backup')
    await waitForApp(page)

    // Open the create modal
    await page.locator('button.btn-primary', { hasText: 'Snapshot Now' }).click()

    // The modal is visible
    const modal = page.locator('.modal-content').first()
    await expect(modal).toBeVisible()
    await expect(modal.locator('.modal-title')).toContainText('Create Snapshot')

    // Volume selector is present
    const volumeSelect = page.locator('#backupVolume')
    await expect(volumeSelect).toBeVisible()
    // It has at least 4 options (1 placeholder + 3 volumes from the view's hardcoded list)
    const volumeOptions = page.locator('#backupVolume option')
    expect(await volumeOptions.count()).toBeGreaterThanOrEqual(4)

    // Select a volume
    await volumeSelect.selectOption({ index: 1 })
    // Snapshot name is auto-generated
    const nameInput = page.locator('#snapshotName')
    await expect(nameInput).toBeVisible()
    await expect(nameInput).not.toHaveValue('')

    // Destination field is present with a placeholder
    const destInput = page.locator('#backupDest')
    await expect(destInput).toBeVisible()
    await expect(destInput).toHaveAttribute('placeholder', /\/.*backups/)

    // The Browse button is visible
    await expect(modal.locator('button[aria-label="Browse destination"]')).toBeVisible()

    // The Cancel + Create Snapshot buttons are present
    await expect(modal.locator('button', { hasText: 'Cancel' })).toBeVisible()
    await expect(modal.locator('button', { hasText: /Create Snapshot|Stop/ }).first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: Restore modal shows overwrite warning on mounted volume
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: Restore modal shows overwrite warning on mounted volume', async ({ page }) => {
    await page.goto('/backup')
    await waitForApp(page)

    // Wait for the history table to render
    await expect(page.locator('table[aria-label="Backup History"]')).toBeVisible()

    // Click the Restore button on the first row (mock snapshot: postgres_data — has attached containers)
    const restoreBtn = page.locator('button[aria-label="Restore"]').first()
    await expect(restoreBtn).toBeVisible()
    await restoreBtn.click()

    // The Restore modal is open
    const modal = page.locator('.modal-content').first()
    await expect(modal).toBeVisible()
    await expect(modal.locator('.modal-title')).toContainText('Restore Snapshot')

    // Snapshot name and source volume fields are pre-filled (readonly)
    await expect(modal.locator('input[readonly]').first()).toBeVisible()
    await expect(modal.locator('input[readonly]').nth(1)).toHaveValue('postgres_data')

    // Select a target volume that has attached containers (postgres_data is one)
    const targetSelect = page.locator('#restoreTarget')
    await expect(targetSelect).toBeVisible()
    await targetSelect.selectOption('postgres_data')

    // Overwrite warning is rendered
    const warningBox = modal.locator('.warning-box').first()
    await expect(warningBox).toBeVisible()
    await expect(warningBox).toContainText(/overwrite|already exists/i)

    // Container-attached warning is also rendered (postgres_data has attached containers in the mock)
    const containerWarning = modal.locator('.warning-box', { hasText: /mounted|running/i })
    await expect(containerWarning).toBeVisible()

    // Action mode radio (Stop & Restore / Cancel)
    await expect(modal.locator('text=Stop & Restore')).toBeVisible()
    await expect(modal.locator('.radio-option', { hasText: 'Cancel' })).toBeVisible()

    // The Restore button is in the footer
    await expect(modal.locator('button.btn-danger', { hasText: /Stop & Restore|Restore/ })).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: Progress bar renders during active backup simulation
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: BackupProgressBar renders with progress bar fill and stats', async ({ page }) => {
    // Simulate an in-progress backup via a Tauri event emission
    // We do this by listening on the page side to inject a progress event payload
    // that the useBackup composable will read on subscription.
    await page.addInitScript(() => {
      // Mock listen() to immediately emit a 'backup-progress' event for testing
      const realListen = (window as any).__TAURI_INTERNALS__?.listen
      if (realListen) {
        (window as any).__TAURI_INTERNALS__.listen = async (eventName: string, handler: any) => {
          if (eventName === 'backup-progress') {
            setTimeout(() => {
              handler({
                payload: {
                  job_id: 'job-1',
                  snapshot_name: 'postgres_data_2026-06-14T10-30-00.tar.gz',
                  bytes_processed: 600_000_000,
                  bytes_total: 1_200_000_000,
                  elapsed_seconds: 30,
                  percent: 50,
                  status: 'in_progress',
                  message: 'Snapshotting postgres_data',
                },
              })
            }, 100)
          }
          return () => {}
        }
      }
    })

    await page.goto('/backup')
    await waitForApp(page)

    // The progress bar component is mounted with class .backup-progress
    const progressBar = page.locator('.backup-progress').first()
    await expect(progressBar).toBeVisible({ timeout: 4000 })

    // The progress fill is rendered with width > 0
    const progressFill = progressBar.locator('.progress-bar-fill')
    await expect(progressFill).toBeVisible()
    const fillWidth = await progressFill.evaluate((el) => (el as HTMLElement).style.width)
    expect(fillWidth).toMatch(/%/)
    expect(parseInt(fillWidth)).toBeGreaterThan(0)
    expect(parseInt(fillWidth)).toBeLessThanOrEqual(100)

    // Stats: Progress, Size, Elapsed
    await expect(progressBar.locator('.backup-progress__stat-value', { hasText: '50%' })).toBeVisible()
    // Size is formatted in KB/MB/GB
    await expect(progressBar.locator('.backup-progress__stat-value').nth(1)).toBeVisible()
    // Elapsed time "30s"
    await expect(progressBar.locator('.backup-progress__stat-value', { hasText: '30s' })).toBeVisible()

    // Cancel button is present while in progress
    await expect(progressBar.locator('button.btn-danger', { hasText: 'Cancel' })).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
