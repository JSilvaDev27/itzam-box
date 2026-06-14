/* ═══════════════════════════════════════════
   ItzamBox — Sprint 18: Kubernetes Cluster Viewer E2E
   Copyright (C) 2026 SodigTech — GPL-3.0
   Task: T-068 — Verify K8s toolbar, resource tabs, inspector, secret redaction, install guide.
   ═══════════════════════════════════════════ */

import { test, expect, type ConsoleMessage } from '@playwright/test'
import { mockDockerResponses, setTauriOverride } from './fixtures/mockDockerResponses'
import { mockV12Responses, setV12Override } from './fixtures/mockV12Responses'
import { waitForApp } from './fixtures/waitForApp'

test.describe('Sprint 18: Kubernetes Cluster Viewer E2E (T-068)', () => {
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
  // Scenario 1: /kubernetes renders K8sToolbar with context/namespace selectors
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 1: K8sToolbar renders with context and namespace selectors', async ({ page }) => {
    await page.goto('/kubernetes')
    await waitForApp(page)

    // Wait for the view to be visible
    await expect(page.locator('.kubernetes-view')).toBeVisible()
    await expect(page.locator('.page-title')).toContainText('Kubernetes')

    // Toolbar is rendered
    const toolbar = page.locator('.k8s-toolbar').first()
    await expect(toolbar).toBeVisible()

    // Two selector buttons exist (context + namespace)
    const selectorBtns = page.locator('.k8s-selector-btn')
    await expect(selectorBtns).toHaveCount(2)

    // Context selector — opens dropdown with context list
    await selectorBtns.nth(0).click()
    const contextDropdown = page.locator('.k8s-context-dropdown').first()
    await expect(contextDropdown).toBeVisible()
    await expect(contextDropdown.locator('.k8s-dropdown-header')).toContainText('Kubernetes Contexts')
    // "minikube" comes from the default v1.2.0 mock
    await expect(contextDropdown.locator('.k8s-dropdown-item-name').first()).toContainText('minikube')

    // Close dropdown by pressing Escape and then open namespace selector
    await page.keyboard.press('Escape')
    await selectorBtns.nth(1).click()
    const nsDropdown = page.locator('.k8s-ns-dropdown').first()
    await expect(nsDropdown).toBeVisible()
    await expect(nsDropdown.locator('.k8s-dropdown-header')).toContainText('Namespaces')
    // First ns from mock: 'default'
    await expect(nsDropdown.locator('.k8s-dropdown-item-name').first()).toContainText('default')

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 2: 4 resource tabs (Pods, Deployments, Services, Config)
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 2: 4 resource tabs are rendered and switchable', async ({ page }) => {
    await page.goto('/kubernetes')
    await waitForApp(page)
    await expect(page.locator('.kubernetes-view')).toBeVisible()

    const tabBar = page.locator('.k8s-tab-bar')
    await expect(tabBar).toBeVisible()

    const tabs = page.locator('.k8s-tab')
    await expect(tabs).toHaveCount(4)
    await expect(tabs.nth(0)).toContainText('Pods')
    await expect(tabs.nth(1)).toContainText('Deployments')
    await expect(tabs.nth(2)).toContainText('Services')
    await expect(tabs.nth(3)).toContainText('Config')

    // Click Deployments tab and verify it becomes active
    await tabs.nth(1).click()
    await expect(tabs.nth(1)).toHaveClass(/active/)

    // Click Services tab
    await tabs.nth(2).click()
    await expect(tabs.nth(2)).toHaveClass(/active/)

    // Click Config tab (renders ConfigResourceTable)
    await tabs.nth(3).click()
    await expect(tabs.nth(3)).toHaveClass(/active/)

    // Return to Pods
    await tabs.nth(0).click()
    await expect(tabs.nth(0)).toHaveClass(/active/)

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 3: Click a pod row → K8sInspectorDrawer opens with Overview tab
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 3: Clicking a pod row opens inspector drawer with Overview tab', async ({ page }) => {
    await page.goto('/kubernetes')
    await waitForApp(page)
    await expect(page.locator('.kubernetes-view')).toBeVisible()

    // The default mock provides a pod named 'nginx-7c8d9f-abc12'
    const podRow = page.locator('text=nginx-7c8d9f-abc12').first()
    await expect(podRow).toBeVisible()
    await podRow.click()

    // Inspector drawer is opened
    const drawer = page.locator('.k8s-inspector-drawer').first()
    await expect(drawer).toBeVisible()

    // Header shows the pod name
    await expect(page.locator('.k8s-inspector-name').first()).toContainText('nginx-7c8d9f-abc12')

    // Overview tab is the default — it should be active
    const overviewTab = page.locator('.k8s-inspector-tab', { hasText: 'Overview' }).first()
    await expect(overviewTab).toHaveClass(/active/)

    // The Overview tab shows pod details (Status, Node, IP, etc.)
    await expect(drawer.locator('.k8s-info-label', { hasText: 'Status' }).first()).toBeVisible()
    await expect(drawer.locator('.k8s-info-label', { hasText: 'Node' }).first()).toBeVisible()
    await expect(drawer.locator('.k8s-info-value', { hasText: 'node-1' }).first()).toBeVisible()

    // Close drawer
    await page.locator('.k8s-inspector-close').first().click()
    await expect(drawer).toBeHidden()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 4: Secret redaction — values hidden by default, eye toggle reveals
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 4: Secret redaction hides values by default and reveals on toggle', async ({ page }) => {
    await page.goto('/kubernetes')
    await waitForApp(page)

    // Switch to Config tab
    const configTab = page.locator('.k8s-tab', { hasText: 'Config' })
    await configTab.click()

    // ConfigResourceTable has ConfigMap/Secret sub-tabs. Click the Secrets sub-tab.
    // The actual implementation switches sub-tab via the ConfigResourceTable internals.
    // We force the inspector open by switching to /kubernetes/secret/db-credentials route
    // OR by simply selecting a secret from the sub-tab. To keep the test deterministic,
    // we deep-link the secret via the route.
    await page.goto('/kubernetes/secret/db-credentials')
    await waitForApp(page)
    await expect(page.locator('.k8s-inspector-drawer')).toBeVisible()

    // Click the "Keys" tab to render SecretValueMask entries
    const keysTab = page.locator('.k8s-inspector-tab', { hasText: 'Keys' }).first()
    await keysTab.click()
    await expect(keysTab).toHaveClass(/active/)

    // At least one SecretValueMask is rendered
    const secretMasks = page.locator('.secret-mask')
    await expect(secretMasks.first()).toBeVisible()

    // Initially hidden — dots and "(hidden)" label are visible
    const firstMask = secretMasks.first()
    await expect(firstMask.locator('.secret-mask-dots')).toBeVisible()
    await expect(firstMask.locator('.secret-mask-label')).toContainText('hidden')

    // Click the eye toggle to reveal
    await firstMask.click()

    // The dots should disappear and the value be revealed
    await expect(firstMask.locator('.secret-mask-dots')).toHaveCount(0)
    // The icon should now be eye-slash (fa-eye-slash)
    await expect(firstMask.locator('i.fa-eye-slash')).toBeVisible()

    // Toggle back to hidden
    await firstMask.click()
    await expect(firstMask.locator('.secret-mask-dots')).toBeVisible()

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })

  // ─────────────────────────────────────────────────────────────────────
  // Scenario 5: No-kubectl state renders KubectlInstallGuide
  // ─────────────────────────────────────────────────────────────────────
  test('Scenario 5: Missing kubectl renders KubectlInstallGuide', async ({ page }) => {
    // Override detect_kubectl to report "not available"
    await setV12Override(page, 'detect_kubectl', {
      available: false,
      version: null,
      kubeconfig_path: null,
      kubeconfig_valid: false,
    })

    await page.goto('/kubernetes')
    await waitForApp(page)

    // The kubectl install guide is rendered
    const guide = page.locator('.kubectl-guide')
    await expect(guide).toBeVisible()
    await expect(page.locator('.kubectl-guide-title')).toContainText('kubectl Not Found')
    await expect(page.locator('.kubectl-guide-desc')).toBeVisible()

    // The guide shows install commands for the detected distro
    await expect(page.locator('.kubectl-guide-commands').first()).toBeVisible()
    await expect(page.locator('.kubectl-guide-command-line').first()).toBeVisible()

    // The retry button is present
    const retryBtn = page.locator('.kubectl-guide button.btn-primary', { hasText: 'Check Again' })
    await expect(retryBtn).toBeVisible()

    // The toolbar should NOT be rendered in no-kubectl state
    await expect(page.locator('.k8s-toolbar')).toHaveCount(0)

    expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  })
})
