<!-- ItzamBox — Swarm Join Modal
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useSwarm } from '../../composables/useSwarm'

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'done'): void
}>()

const { joinSwarm, terminalOutput, listenSwarmOutput, stopListeningSwarmOutput, clearTerminalOutput } = useSwarm()

const joinToken = ref('')
const managerAddr = ref('')
const joinRole = ref<'worker' | 'manager'>('worker')
const submitting = ref(false)
const success = ref(false)
const localError = ref<string | null>(null)

onMounted(async () => {
  clearTerminalOutput()
  await listenSwarmOutput()
})

onBeforeUnmount(() => {
  stopListeningSwarmOutput()
})

function validateForm(): boolean {
  localError.value = null
  if (!joinToken.value.trim()) {
    localError.value = 'Join token is required'
    return false
  }
  if (!joinToken.value.trim().startsWith('SWMTKN-')) {
    localError.value = 'Invalid join token format (must start with SWMTKN-)'
    return false
  }
  if (!managerAddr.value.trim()) {
    localError.value = 'Manager address is required'
    return false
  }
  const addrPattern = /^(\d{1,3}\.){3}\d{1,3}(:\d{1,5})?$/
  if (!addrPattern.test(managerAddr.value.trim())) {
    localError.value = 'Invalid address format (e.g. 192.168.1.100:2377)'
    return false
  }
  return true
}

async function handleJoin() {
  if (!validateForm()) return
  submitting.value = true
  localError.value = null
  try {
    await joinSwarm(joinToken.value.trim(), managerAddr.value.trim())
    success.value = true
  } catch (e: any) {
    localError.value = e?.toString?.() ?? String(e)
  } finally {
    submitting.value = false
  }
}

function handleDone() {
  clearTerminalOutput()
  emit('done')
}

function handleClose() {
  if (!submitting.value) {
    clearTerminalOutput()
    emit('close')
  }
}
</script>

<template>
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Join Existing Swarm"
    @click.self="handleClose"
  >
    <div class="modal-content" style="max-width: 520px">
      <div class="modal-header">
        <h2 class="modal-title">
          <i class="fa-solid fa-right-to-bracket" style="color: var(--accent-purple); margin-right: 8px"></i>
          Join Existing Swarm
        </h2>
        <button class="action-btn" aria-label="Close" @click="handleClose">
          <i class="fa-solid fa-xmark"></i>
        </button>
      </div>

      <div class="modal-body">
        <!-- Error alert -->
        <div v-if="localError" class="join-error" role="alert" style="margin-bottom: 16px">
          <i class="fa-solid fa-circle-exclamation" style="margin-right: 8px"></i>
          {{ localError }}
        </div>

        <!-- Form (hidden after success) -->
        <template v-if="!success">
          <div class="form-group">
            <label class="form-label" for="joinToken">Join Token *</label>
            <input
              id="joinToken"
              v-model="joinToken"
              class="form-input mono"
              type="text"
              placeholder="SWMTKN-1-..."
              aria-required="true"
              :disabled="submitting"
              @keyup.enter="handleJoin"
            />
            <span class="form-hint">Paste the token from the manager's <code>docker swarm join</code> output</span>
          </div>

          <div class="form-group">
            <label class="form-label" for="managerAddr">Manager Address *</label>
            <input
              id="managerAddr"
              v-model="managerAddr"
              class="form-input mono"
              type="text"
              placeholder="192.168.1.100:2377"
              aria-required="true"
              :disabled="submitting"
              @keyup.enter="handleJoin"
            />
            <span class="form-hint">IP and port of the swarm manager node</span>
          </div>

          <div class="form-group">
            <label class="form-label" for="joinRole">Role</label>
            <select
              id="joinRole"
              v-model="joinRole"
              class="form-select"
              :disabled="submitting"
            >
              <option value="worker">Worker</option>
              <option value="manager">Manager</option>
            </select>
            <span class="form-hint">Worker nodes run tasks; manager nodes manage the swarm</span>
          </div>
        </template>

        <!-- Success message -->
        <div v-if="success" class="join-success" role="status">
          <i class="fa-solid fa-circle-check" style="font-size: 36px; color: var(--accent-green)"></i>
          <p style="font-size: 15px; font-weight: 600; margin: 12px 0 4px">
            Successfully joined swarm as {{ joinRole }}
          </p>
          <p class="text-muted" style="font-size: 12px">
            This node is now part of the cluster
          </p>
        </div>

        <!-- Terminal Output -->
        <div v-if="terminalOutput.length > 0" class="terminal-panel" style="margin-top: 12px">
          <div class="terminal-panel__header">
            <span><i class="fa-solid fa-terminal" style="margin-right: 4px"></i> Output</span>
            <span style="font-size: 10px; color: var(--text-disabled)">docker swarm join</span>
          </div>
          <div class="terminal-panel__body">
            <div v-for="(line, i) in terminalOutput" :key="i" class="terminal-line">{{ line }}</div>
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
          @click="handleJoin"
        >
          <i v-if="submitting" class="fa-solid fa-spinner fa-spin" aria-hidden="true"></i>
          <i v-else class="fa-solid fa-right-to-bracket" aria-hidden="true"></i>
          {{ submitting ? 'Joining...' : 'Join' }}
        </button>
        <button v-if="success" class="btn btn-primary" @click="handleDone">
          <i class="fa-solid fa-check"></i> Done
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.join-error {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: var(--radius-md);
  color: var(--accent-red);
  font-size: 13px;
}

.join-success {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 24px;
  text-align: center;
}

.form-hint {
  display: block;
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-disabled);
}

.form-hint code {
  font-family: var(--font-mono);
  font-size: 10px;
  background: var(--bg-tertiary);
  padding: 1px 4px;
  border-radius: 3px;
}

.form-select {
  width: 100%;
  padding: 8px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  color: var(--text-main);
  font-size: 13px;
  font-family: var(--font-sans);
  cursor: pointer;
}

.form-select:focus {
  border-color: var(--accent-cyan);
  outline: none;
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
</style>
