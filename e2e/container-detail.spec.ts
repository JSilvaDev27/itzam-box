/* ═══════════════════════════════════════════
   ItzamBox — Sprint 16: Container Detail (8 tabs) E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses, setTauriOverride } from './fixtures/mockDockerResponses'
import { waitForApp } from './fixtures/waitForApp'

/**
 * The 8 detail tabs in the order they render in the UI:
 *   1. Info      (i.fa-circle-info)
 *   2. Env Vars  (i.fa-list)
 *   3. Volumes   (i.fa-database)
 *   4. Network   (i.fa-network-wired)
 *   5. Health    (i.fa-heart-pulse)
 *   6. Labels    (i.fa-tags)
 *   7. Logs      (i.fa-scroll)
 *   8. Stats     (i.fa-chart-line)
 */
const DETAIL_TABS = ['Info', 'Env Vars', 'Volumes', 'Network', 'Health', 'Labels', 'Logs', 'Stats']

test.describe('Sprint 16: Container Detail (8 tabs) E2E', () => {
  let consoleErrors: string[] = []

  test.beforeEach(async ({ page }) => {
    consoleErrors = []
    page.on('console', (msg: ConsoleMessage) => {
      if (msg.type() === 'error') consoleErrors.push(msg.text())
    })
    page.on('pageerror', err => consoleErrors.push(err.message))

    await mockDockerResponses(page)

    // Provide a richer inspect payload so all tabs render something meaningful
    await setTauriOverride(page, 'inspect_container', {
      Id: '1234567890ab',
      Name: '/nginx-test',
      Created: '2026-06-13T19:00:00.000Z',
      Image: 'sha256:abcd1234',
      Driver: 'overlay2',
      Architecture: 'amd64',
      Platform: 'linux',
      RestartCount: 0,
      LogPath: '/var/lib/docker/containers/1234567890ab/1234567890ab-json.log',
      Config: {
        Image: 'nginx:alpine',
        Env: [
          'PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin',
          'NGINX_VERSION=1.27.0',
        ],
        Cmd: ['nginx', '-g', 'daemon off;'],
        Entrypoint: ['/docker-entrypoint.sh'],
        Labels: { 'com.example.app': 'web', 'maintainer': 'sodigtech' },
      },
      State: {
        Status: 'running',
        Running: true,
        Paused: false,
        ExitCode: 0,
        StartedAt: '2026-06-13T19:01:00.000Z',
        FinishedAt: '0001-01-01T00:00:00Z',
      },
      HostConfig: { RestartPolicy: { Name: 'always' } },
      Mounts: [
        {
          Type: 'bind',
          Source: '/host/data',
          Destination: '/data',
          Mode: 'ro',
          RW: false,
        },
      ],
      NetworkSettings: {
        Networks: {
          bridge: {
            IPAddress: '172.17.0.2',
            IPPrefixLen: 16,
            Gateway: '172.17.0.1',
            MacAddress: '02:42:ac:11:00:02',
            NetworkID: 'net-abcdef0123456789',
            EndpointID: 'ep-0123456789abcdef',
          },
        },
        Ports: {
          '80/tcp': [{ HostIp: '0.0.0.0', HostPort: '8080' }],
        },
      },
    })
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 1: 8 tabs render and URL is /containers/<id>
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: 8 detail tabs render under /containers/<id>', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    // URL should match the pattern
    expect(page.url()).toMatch(/\/containers\/1234567890ab$/)

    // All 8 tabs should be visible
    for (const tab of DETAIL_TABS) {
      const tabBtn = page.locator(`.tab:has-text("${tab}")`).first()
      await expect(tabBtn).toBeVisible()
    }

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: Clicking each tab swaps the content area
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: Clicking each tab switches the active tab', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    for (const tab of DETAIL_TABS) {
      const tabBtn = page.locator(`.tab:has-text("${tab}")`).first()
      await expect(tabBtn).toBeVisible()
      await tabBtn.click()

      // The clicked tab should now have the .active class
      await expect(tabBtn).toHaveClass(/active/)
    }

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: Info tab shows container metadata
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: Info tab shows container general and configuration sections', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    // Click the Info tab
    await page.locator('.tab:has-text("Info")').first().click()

    // The Info tab renders <div class="inspect-grid"> with multiple sections
    await expect(page.locator('.inspect-grid').first()).toBeVisible()

    // Check some specific labels
    await expect(page.locator('text=Container ID').first()).toBeVisible()
    await expect(page.locator('text=Image').first()).toBeVisible()
    await expect(page.locator('text=State').first()).toBeVisible()
    await expect(page.locator('text=nginx:alpine').first()).toBeVisible()
    await expect(page.locator('text=running').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: Env Vars tab shows the variables
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: Env Vars tab lists environment variables', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    await page.locator('.tab:has-text("Env Vars")').first().click()

    // Should show NGINX_VERSION (from the mock inspect)
    await expect(page.getByText('NGINX_VERSION', { exact: true }).first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: Volumes tab shows the mounts
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: Volumes tab shows the mount bindings', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    await page.locator('.tab:has-text("Volumes")').first().click()

    // Should show the bind mount source/destination
    await expect(page.getByText('/host/data', { exact: true }).first()).toBeVisible()
    await expect(page.getByText('/data', { exact: true }).first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 6: Network tab shows network configuration
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 6: Network tab shows IP and gateway', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    await page.locator('.tab:has-text("Network")').first().click()

    await expect(page.locator('text=172.17.0.2').first()).toBeVisible()
    await expect(page.locator('text=172.17.0.1').first()).toBeVisible()
    await expect(page.locator('text=8080').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 7: Logs tab fetches and renders the log content
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 7: Logs tab fetches and renders log content', async ({ page }) => {
    await setTauriOverride(page, 'get_container_logs',
      '2026-06-13T20:00:00Z [info] Starting nginx...\n2026-06-13T20:00:01Z [info] Ready for connections.\n')

    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    await page.locator('.tab:has-text("Logs")').first().click()

    // The log content area should render the text
    const logContent = page.locator('.log-content').first()
    await expect(logContent).toBeVisible()
    // Some log text should be in the rendered HTML
    await expect(logContent).toContainText('Starting nginx')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 8: Stats tab shows the polling indicator and metric cards
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 8: Stats tab shows the live polling indicator and metric cards', async ({ page }) => {
    await setTauriOverride(page, 'get_container_stats', {
      container_id: '1234567890ab',
      cpu_percentage: 5.4,
      memory_usage_bytes: 25_000_000,
      memory_limit_bytes: 8_000_000_000,
      memory_percentage: 0.31,
      network_rx_bytes: 2048,
      network_tx_bytes: 1024,
      block_read_bytes: 0,
      block_write_bytes: 4096,
      pids: 4,
      timestamp: Date.now() / 1000,
    })

    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    await page.locator('.tab:has-text("Stats")').first().click()

    // The "Live (polling every 3s)" indicator should be visible
    await expect(page.locator('text=Live').first()).toBeVisible()
    await expect(page.locator('text=polling').first()).toBeVisible()

    // Metric cards should render
    await expect(page.locator('.metric-card', { hasText: 'CPU' }).first()).toBeVisible()
    await expect(page.locator('.metric-card', { hasText: 'Memory' }).first()).toBeVisible()
    await expect(page.locator('.metric-card', { hasText: 'Network' }).first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 9: Labels tab shows the labels key-value list
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 9: Labels tab shows the container labels', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    await page.locator('.tab:has-text("Labels")').first().click()

    await expect(page.locator('text=com.example.app').first()).toBeVisible()
    await expect(page.locator('text=web').first()).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 10: Action buttons (Stop / Restart) are visible on running container
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 10: Action buttons (Stop / Restart) render for a running container', async ({ page }) => {
    await page.goto('/containers/1234567890ab')
    await waitForApp(page)

    // For a running container, the Stop and Restart buttons should be visible
    const stopBtn = page.locator('.detail-actions button:has-text("Stop")').first()
    const restartBtn = page.locator('.detail-actions button:has-text("Restart")').first()

    await expect(stopBtn).toBeVisible()
    await expect(restartBtn).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
