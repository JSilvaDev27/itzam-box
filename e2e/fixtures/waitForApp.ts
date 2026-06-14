import { Page } from '@playwright/test'

export async function waitForApp(page: Page) {
  // Wait for the main app container to mount and render
  await page.waitForSelector('#app', { state: 'attached' })
  // Wait for the header to be visible which signals that loading is complete
  await page.waitForSelector('.header-title', { state: 'visible' })
}
