<!-- ItzamBox — Swarm Init Modal
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useSwarm } from '../../composables/useSwarm'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'done'): void
}>()

const { initSwarm, terminalOutput, initTokens, listenSwarmOutput, stopListeningSwarmOutput, clearTerminalOutput } = useSwarm()

const advertiseAddr = ref('')
const submitting = ref(false)
const success = ref(false)
const localError = ref<string | null>(null)
const showTokens = ref(false)

onMounted(async () => {
  clearTerminalOutput()
  // Attempt to detect default IP
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    // The backend exposes a `detect_advertise_addr` helper via swarm_status
    const hostname = await invoke<{ ip: string }>('get_host_metrics')
    // Fallback: try docker info
    advertiseAddr.value = '192.168.1.100' // sensible fallback
  } catch {
    advertiseAddr.value = ''
  }
  await listenSwarmOutput()
})

onBeforeUnmount(() => {
  stopListeningSwarmOutput()
})

function validateForm(): boolean {
  localError.value = null
  const ipPattern = /^(\d{1,3}\.){3}\d{1,3}$/
  if (!advertiseAddr.value.trim()) {
    localError.value = 'Advertise address is required'
    return false
  }
  if (!ipPattern.test(advertiseAddr.value.trim())) {
    localError.value = 'Invalid IP address format'
    return false
  }
  return true
}

async function handleInit() {
  if (!validateForm()) return
  submitting.value = true
  localError.value = null
  try {
    await initSwarm(advertiseAddr.value.trim())
    success.value = true
    showTokens.value = true
  } catch (e: any) {
    localError.value = e.toString()
  } finally {
    submitting.value = false
  }
}

function handleDone() {
  emit('done')
}

function handleClose() {
  if (!submitting.value) {
    clearTerminalOutput()
    emit('close')
  }
}

function copyToken(token: string) {
  navigator.clipboard.writeText(token).catch(() => {
    // Fallback: select text manually
  })
}
</script>

<template>
  <div
    class="modal-backdrop swarm-modal"
    role="dialog"
    aria-modal="true"
    aria-label="Initialize Swarm"
    @click.self="handleClose"
  >
    <div class="modal-content" style="max-width: 600px">
      <div class="modal-header">
        <h2 class="modal-title">
          <i class="fa-solid fa-play" style="color: var(--accent-cyan); margin-right: 8px"></i>
          Initialize Swarm
        </h2>
        <button class="action-btn" aria-label="Close" @click="handleClose">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <div class="modal-body">
        <!-- Error alert -->
        <div v-if="localError" class="error-banner" role="alert" style="margin-bottom: 16px">
          <i class="fa-solid fa-circle-exclamation" style="margin-right: 8px"></i>
          {{ localError }}
        </div>

        <!-- Form -->
        <div class="form-group" v-if="!success">
          <label class="form-label" for="advertiseAddr">Advertise Address *</label>
          <div style="display: flex; gap: 8px">
            <input
              id="advertiseAddr"
              v-model="advertiseAddr"
              class="form-input mono"
              type="text"
              placeholder="192.168.1.100"
              aria-required="true"
              :disabled="submitting"
              @keyup.enter="handleInit"
            />
          </div>
          <span class="form-hint">The IP address other nodes will use to reach this manager</span>
        </div>

        <!-- Terminal Output -->
        <div v-if="terminalOutput.length > 0" class="terminal-panel" style="margin-top: 12px">
          <div class="terminal-panel__header">
            <span><i class="fa-solid fa-terminal" style="margin-right: 4px"></i> Output</span>
            <span style="font-size: 10px; color: var(--text-disabled)">docker swarm init</span>
          </div>
          <div class="terminal-panel__body" ref="terminalBody">
            <div v-for="(line, i) in terminalOutput" :key="i" class="terminal-line">{{ line }}</div>
          </div>
        </div>

        <!-- Join Tokens (init success) -->
        <div v-if="showTokens && initTokens.length > 0" style="margin-top: 16px">
          <div class="swarm-token-section">
            <i class="fa-solid fa-key" style="color: var(--accent-cyan); margin-right: 6px"></i>
            <strong>Join Tokens</strong>
            <span class="text-muted" style="font-size: 11px; margin-left: 8px">
              Copy these to add nodes to the swarm
            </span>
          </div>
          <div v-for="(token, i) in initTokens" :key="i" class="swarm-token-row">
            <code class="swarm-token-code">{{ token }}</code>
            <button
              class="btn btn-ghost btn-sm"
              :aria-label="'Copy token ' + (i + 1)"
              title="Copy to clipboard"
              @click="copyToken(token)"
            >
              <i class="fa-solid fa-copy"></i>
            </button>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button v-if="!success" class="btn btn-ghost" :disabled="submitting" @click="handleClose">
          Cancel
        </button>
        <button
          v-if="!success"
          class="btn btn-primary"
          :disabled="submitting"
          @click="handleInit"
        >
          <i v-if="submitting" class="fa-solid fa-spinner fa-spin" aria-hidden="true"></i>
          <i v-else class="fa-solid fa-play" aria-hidden="true"></i>
          {{ submitting ? 'Initializing...' : 'Initialize' }}
        </button>
        <button v-if="success" class="btn btn-primary" @click="handleDone">
          <i class="fa-solid fa-check"></i> Done
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.error-banner {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--radius-md);
  color: var(--accent-red);
  font-size: 13px;
}

.form-hint {
  display: block;
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-disabled);
}

.terminal-panel {
  background: var(--terminal-bg);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.terminal-panel__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.03);
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-muted);
}

.terminal-panel__body {
  padding: 12px;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--terminal-fg);
  line-height: 1.7;
  max-height: 200px;
  overflow-y: auto;
  white-space: pre-wrap;
}

.terminal-line {
  word-break: break-all;
}

.swarm-token-section {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
  font-size: 13px;
}

.swarm-token-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 6px;
  padding: 8px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
}

.swarm-token-code {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--accent-green);
  word-break: break-all;
  line-height: 1.5;
}
</style>
