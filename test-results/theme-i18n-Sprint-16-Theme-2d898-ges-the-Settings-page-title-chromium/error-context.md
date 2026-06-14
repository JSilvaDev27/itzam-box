# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: theme-i18n.spec.ts >> Sprint 16: Theme Toggle & i18n E2E >> Scenario 4: Switching to English changes the Settings page title
- Location: e2e/theme-i18n.spec.ts:94:3

# Error details

```
Error: expect(locator).toBeVisible() failed

Locator: locator('h1').filter({ hasText: 'Settings' })
Expected: visible
Timeout: 5000ms
Error: element(s) not found

Call log:
  - Expect "toBeVisible" with timeout 5000ms
  - waiting for locator('h1').filter({ hasText: 'Settings' })

```

```yaml
- banner:
  - text: IB ItzamBox
  - textbox "Buscar..."
  - button "1"
  - button "Toggle theme"
  - button "Settings"
- complementary:
  - navigation:
    - link "Dashboard":
      - /url: /
    - link "Containers":
      - /url: /containers
    - link "Images":
      - /url: /images
    - link "Build":
      - /url: /build
    - link "Volumes":
      - /url: /volumes
    - link "Networks":
      - /url: /networks
    - link "Events":
      - /url: /events
    - link "Cleanup":
      - /url: /cleanup
    - link "Compose":
      - /url: /compose
    - link "Registries":
      - /url: /registries
    - link "Run Wizard":
      - /url: /run-wizard
    - link "Templates":
      - /url: /templates
    - link "Docker Setup":
      - /url: /installer
    - link "Export/Import":
      - /url: /export-import
    - link "Settings":
      - /url: /settings
    - link "Help":
      - /url: /help
  - text: CPU -- RAM --
- main:
  - text: Home Configuración
  - heading "Configuración" [level=1]
  - text: Tema Dark Light ✓ Active Idioma 🇪🇸 Español 🇺🇸 Inglés Versión ItzamBox v1.0.0 Tauri v2.11 License GNU GPL v3.0 © 2026 SodigTech
- text: Terminal Host Host
```

# Test source

```ts
  11  |   let consoleErrors: string[] = []
  12  | 
  13  |   test.beforeEach(async ({ page }) => {
  14  |     consoleErrors = []
  15  |     page.on('console', (msg: ConsoleMessage) => {
  16  |       if (msg.type() === 'error') consoleErrors.push(msg.text())
  17  |     })
  18  |     page.on('pageerror', err => consoleErrors.push(err.message))
  19  | 
  20  |     await mockDockerResponses(page)
  21  |   })
  22  | 
  23  |   // ─────────────────────────────────────────────────────────────────────
  24  |   // Scenario 1: Theme attribute is set on <html> on initial load
  25  |   //             (the useTheme composable applies data-theme to
  26  |   //             documentElement, not body)
  27  |   // ─────────────────────────────────────────────────────────────────────
  28  |   test('Scenario 1: Initial theme is dark (data-theme=dark on <html>)', async ({ page }) => {
  29  |     await page.goto('/')
  30  |     await waitForApp(page)
  31  | 
  32  |     // The mock returns 'dark' for get_config({key:'theme'})
  33  |     const theme = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
  34  |     expect(theme).toBe('dark')
  35  |   })
  36  | 
  37  |   // ─────────────────────────────────────────────────────────────────────
  38  |   // Scenario 2: Clicking the toggle theme button flips the theme
  39  |   // ─────────────────────────────────────────────────────────────────────
  40  |   test('Scenario 2: Clicking the theme toggle switches between dark and light', async ({ page }) => {
  41  |     await page.goto('/')
  42  |     await waitForApp(page)
  43  | 
  44  |     const themeBtn = page.locator('button[title="Toggle theme"]').first()
  45  |     await expect(themeBtn).toBeVisible()
  46  | 
  47  |     // Click once → light
  48  |     await themeBtn.click()
  49  |     await page.waitForTimeout(150)
  50  |     const afterClick1 = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
  51  |     expect(afterClick1).toBe('light')
  52  | 
  53  |     // Click again → dark
  54  |     await themeBtn.click()
  55  |     await page.waitForTimeout(150)
  56  |     const afterClick2 = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
  57  |     expect(afterClick2).toBe('dark')
  58  | 
  59  |     expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  60  |   })
  61  | 
  62  |   // ─────────────────────────────────────────────────────────────────────
  63  |   // Scenario 3: Settings page exposes theme selection cards (dark / light)
  64  |   // ─────────────────────────────────────────────────────────────────────
  65  |   test('Scenario 3: Settings page exposes Dark and Light theme cards', async ({ page }) => {
  66  |     await page.goto('/settings')
  67  |     await waitForApp(page)
  68  | 
  69  |     // The Settings view renders two clickable theme cards
  70  |     await expect(page.locator('text=Dark').first()).toBeVisible()
  71  |     await expect(page.locator('text=Light').first()).toBeVisible()
  72  | 
  73  |     // Click the Light card → theme becomes light
  74  |     const lightCard = page.locator('div:has-text("Light")').first()
  75  |     await lightCard.click()
  76  |     await page.waitForTimeout(150)
  77  |     const theme = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
  78  |     expect(theme).toBe('light')
  79  | 
  80  |     // Toggle back to dark
  81  |     const darkCard = page.locator('div:has-text("Dark")').first()
  82  |     await darkCard.click()
  83  |     await page.waitForTimeout(150)
  84  |     const theme2 = await page.evaluate(() => document.documentElement.getAttribute('data-theme'))
  85  |     expect(theme2).toBe('dark')
  86  | 
  87  |     expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  88  |   })
  89  | 
  90  |   // ─────────────────────────────────────────────────────────────────────
  91  |   // Scenario 4: i18n — switch from ES to EN and verify a key label
  92  |   //              changes (the Settings view re-renders t.settings.* keys)
  93  |   // ─────────────────────────────────────────────────────────────────────
  94  |   test('Scenario 4: Switching to English changes the Settings page title', async ({ page }) => {
  95  |     // Mock returns lang = 'es' by default → Settings should show "Configuración"
  96  |     await page.goto('/settings')
  97  |     await waitForApp(page)
  98  | 
  99  |     // The H1 should read "Configuración" in Spanish
  100 |     await expect(page.locator('h1', { hasText: 'Configuración' })).toBeVisible()
  101 | 
  102 |     // Click the English flag (locale='en' card)
  103 |     // The Settings view has two divs with @click="setLocale('es')" and ('en')
  104 |     // We click the one containing the English text "Inglés" (the ES label).
  105 |     await page.locator('div:has-text("Inglés")').first().click()
  106 | 
  107 |     // Allow Vue to re-render
  108 |     await page.waitForTimeout(200)
  109 | 
  110 |     // Now the H1 should be "Settings"
> 111 |     await expect(page.locator('h1', { hasText: 'Settings' })).toBeVisible()
      |                                                               ^ Error: expect(locator).toBeVisible() failed
  112 | 
  113 |     expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  114 |   })
  115 | 
  116 |   // ─────────────────────────────────────────────────────────────────────
  117 |   // Scenario 5: i18n — switch back from EN to ES
  118 |   // ─────────────────────────────────────────────────────────────────────
  119 |   test('Scenario 5: Switching back to Spanish restores the Configuración label', async ({ page }) => {
  120 |     await page.goto('/settings')
  121 |     await waitForApp(page)
  122 | 
  123 |     // Switch to EN first
  124 |     await page.locator('div:has-text("Inglés")').first().click()
  125 |     await page.waitForTimeout(200)
  126 |     await expect(page.locator('h1', { hasText: 'Settings' })).toBeVisible()
  127 | 
  128 |     // Now switch back to ES (the card showing "Español" label)
  129 |     await page.locator('div:has-text("Español")').first().click()
  130 |     await page.waitForTimeout(200)
  131 |     await expect(page.locator('h1', { hasText: 'Configuración' })).toBeVisible()
  132 | 
  133 |     expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  134 |   })
  135 | 
  136 |   // ─────────────────────────────────────────────────────────────────────
  137 |   // Scenario 6: Initial lang is 'es' (mock returns 'es' for get_config)
  138 |   // ─────────────────────────────────────────────────────────────────────
  139 |   test('Scenario 6: Default language is Spanish', async ({ page }) => {
  140 |     await page.goto('/settings')
  141 |     await waitForApp(page)
  142 | 
  143 |     // The H1 in ES is "Configuración"
  144 |     await expect(page.locator('h1.text-h1', { hasText: 'Configuración' })).toBeVisible()
  145 | 
  146 |     expect(consoleErrors, `Unexpected console errors: ${consoleErrors.join('\n')}`).toHaveLength(0)
  147 |   })
  148 | })
  149 | 
```