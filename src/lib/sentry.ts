/* ═══════════════════════════════════════════
   ItzamBox — Sentry Initialization Module
   Copyright (C) 2026 SodigTech — GPL-3.0
   Configures error tracking, performance monitoring,
   and session replay for production builds.
   ═══════════════════════════════════════════ */

import * as Sentry from '@sentry/vue'
import type { App } from 'vue'
import type { Router } from 'vue-router'

/**
 * Initialize Sentry error tracking and performance monitoring.
 * Only activates in production (VITE_SENTRY_DSN must be set).
 *
 * Integrations:
 *  - BrowserTracing: automatic route-level performance traces
 *  - Replay: session recording for error reproduction
 *
 * @param app - The Vue application instance
 * @param router - The Vue Router instance (for tracing)
 */
export function initSentry(app: App, router: Router): void {
  const dsn = import.meta.env.VITE_SENTRY_DSN

  if (!import.meta.env.PROD || !dsn) {
    if (import.meta.env.DEV) {
      console.log('[Sentry] Skipped (DSN not configured or non-production mode)')
    }
    return
  }

  Sentry.init({
    app,
    dsn,
    environment: import.meta.env.MODE,
    release: `itzam-box@${import.meta.env.VITE_APP_VERSION || '1.1.0'}`,

    integrations: [
      Sentry.browserTracingIntegration({ router }),
      Sentry.replayIntegration(),
    ],

    // Performance Monitoring — sample 20% of transactions
    tracesSampleRate: 0.2,

    // Session Replay — sample 10% of sessions, 50% on error
    replaysSessionSampleRate: 0.1,
    replaysOnErrorSampleRate: 0.5,
  })

  console.log('[Sentry] Initialized (environment:', import.meta.env.MODE, ')')
}
