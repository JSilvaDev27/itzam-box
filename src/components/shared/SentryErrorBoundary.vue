<!-- ItzamBox — Sentry Error Boundary
     Copyright (C) 2026 SodigTech — GPL-3.0
     Wraps app content and captures unhandled errors to Sentry
     in production mode. Falls back to a user-friendly error UI. -->
<script setup lang="ts">
import { ref, onErrorCaptured, type Ref } from 'vue'
import * as Sentry from '@sentry/vue'

interface CapturedError {
  message: string
  timestamp: string
  eventId?: string
}

const hasError: Ref<boolean> = ref(false)
const error: Ref<CapturedError | null> = ref(null)

onErrorCaptured((err: unknown, instance: unknown, info: string): boolean => {
  const isProd = import.meta.env.PROD
  const message = err instanceof Error ? err.message : String(err)

  error.value = {
    message,
    timestamp: new Date().toISOString(),
  }

  if (isProd) {
    const eventId = Sentry.captureException(err, {
      contexts: {
        vue: {
          componentName: (instance as any)?.$options?.name ?? 'Unknown',
          info,
        },
      },
    })
    error.value.eventId = eventId
  } else {
    console.error('[ErrorBoundary] Caught:', err, 'Info:', info)
  }

  hasError.value = true

  // Prevent the error from propagating further
  return false
})

function resetError(): void {
  hasError.value = false
  error.value = null
}
</script>

<template>
  <div v-if="hasError" class="sentry-error-boundary">
    <div class="sentry-error-boundary-card">
      <div class="sentry-error-boundary-icon">
        <svg width="56" height="56" viewBox="0 0 56 56" fill="none">
          <circle cx="28" cy="28" r="24" stroke="currentColor" stroke-width="1.5" opacity="0.3" />
          <path
            d="M28 18v14M28 36.5v.01"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
          />
        </svg>
      </div>
      <h2 class="sentry-error-boundary-title">Something went wrong</h2>
      <p class="sentry-error-boundary-message">
        An unexpected error occurred. Our team has been notified.
      </p>
      <p v-if="error?.eventId" class="sentry-error-boundary-id">
        Error ID: <code>{{ error.eventId }}</code>
      </p>
      <div class="sentry-error-boundary-actions">
        <button class="btn btn-primary" @click="resetError">
          <i class="fa-solid fa-rotate"></i> Try Again
        </button>
        <button class="btn btn-ghost" @click="resetError">
          <i class="fa-solid fa-arrow-left"></i> Go Back
        </button>
      </div>
    </div>
  </div>
  <slot v-else />
</template>

<style scoped>
.sentry-error-boundary {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  padding: 2rem;
}

.sentry-error-boundary-card {
  text-align: center;
  max-width: 400px;
}

.sentry-error-boundary-icon {
  margin-bottom: 1.5rem;
  color: var(--color-error, #e74c3c);
  opacity: 0.8;
}

.sentry-error-boundary-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 0.75rem;
  color: var(--color-text-primary, #1a1a2e);
}

.sentry-error-boundary-message {
  font-size: 0.95rem;
  color: var(--color-text-secondary, #6b7280);
  margin-bottom: 1rem;
  line-height: 1.5;
}

.sentry-error-boundary-id {
  font-size: 0.8rem;
  color: var(--color-text-tertiary, #9ca3af);
  margin-bottom: 1.5rem;
}

.sentry-error-boundary-id code {
  background: var(--color-surface-secondary, #f3f4f6);
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  font-family: 'SF Mono', 'Fira Code', monospace;
}

.sentry-error-boundary-actions {
  display: flex;
  gap: 0.75rem;
  justify-content: center;
}
</style>
