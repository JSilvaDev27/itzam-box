<!-- ItzamBox — Image Build View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, watch, nextTick, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { open } from '@tauri-apps/plugin-dialog'
import {
  buildImage,
  listenBuildLog,
  listenBuildComplete,
  type BuildLogLine,
  type BuildCompletePayload,
} from '../composables/useDocker'
import { useNotifications } from '../composables/useNotifications'

const router = useRouter()
const { success: notifySuccess, error: notifyError } = useNotifications()

// ─── Form State ──────────────────────────────────────────────────────────

const dockerfilePath = ref('')
const contextDir = ref('')
const tags = ref<string[]>([])
const newTag = ref('')
const buildArgs = ref<{ key: string; value: string }[]>([])
const noCache = ref(false)
const pullBase = ref(false)

// ─── Build State ─────────────────────────────────────────────────────────

type BuildStatus = 'idle' | 'building' | 'success' | 'error'
const status = ref<BuildStatus>('idle')
const logLines = ref<BuildLogLine[]>([])
const buildError = ref<string | null>(null)
const builtImageId = ref<string | null>(null)
const builtTags = ref<string[]>([])
const progressStep = ref(0)
const progressTotal = ref(0)
const building = computed(() => status.value === 'building')

// Unlisten functions (cleaned up on unmount)
let unlistenLog: (() => void) | null = null
let unlistenComplete: (() => void) | null = null

// ─── Scroll anchor for auto-scroll ───────────────────────────────────────

const logContainer = ref<HTMLElement | null>(null)

function scrollToBottom() {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}

watch(logLines, scrollToBottom, { deep: true })

// ─── Form validation ─────────────────────────────────────────────────────

const formValid = computed(() => {
  return dockerfilePath.value.trim().length > 0
    && contextDir.value.trim().length > 0
})

// ─── File/Directory dialogs ──────────────────────────────────────────────

async function browseDockerfile() {
  const selected = await open({
    title: 'Select Dockerfile',
    filters: [
      { name: 'Dockerfile', extensions: ['dockerfile', 'Dockerfile', ''] },
      { name: 'All files', extensions: ['*'] },
    ],
    multiple: false,
    directory: false,
  })
  if (selected) {
    dockerfilePath.value = selected as string
  }
}

async function browseContextDir() {
  const selected = await open({
    title: 'Select Build Context Directory',
    multiple: false,
    directory: true,
  })
  if (selected) {
    contextDir.value = selected as string
  }
}

// ─── Tags management ─────────────────────────────────────────────────────

function addTag() {
  const tag = newTag.value.trim()
  if (tag && !tags.value.includes(tag)) {
    tags.value.push(tag)
    newTag.value = ''
  }
}

function removeTag(index: number) {
  tags.value.splice(index, 1)
}

function handleTagKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault()
    addTag()
  }
}

// ─── Build Args management ───────────────────────────────────────────────

function addBuildArg() {
  buildArgs.value.push({ key: '', value: '' })
}

function removeBuildArg(index: number) {
  buildArgs.value.splice(index, 1)
}

// ─── Parse progress from log line ────────────────────────────────────────

function parseProgress(line: string) {
  // Match "Step X/N" pattern: "Step 1/10 : FROM node:18"
  const stepMatch = line.match(/^Step\s+(\d+)\/(\d+)\s*:/i)
  if (stepMatch) {
    progressStep.value = parseInt(stepMatch[1], 10)
    progressTotal.value = parseInt(stepMatch[2], 10)
  }
}

// ─── Line color helpers ──────────────────────────────────────────────────

type LineColor = 'default' | 'cyan' | 'green' | 'red' | 'yellow'

function getLineColor(line: string): LineColor {
  const upper = line.toUpperCase()
  if (/^(STEP\s+\d+\/\d+|#\d+\s+)/.test(upper)) return 'cyan'
  if (/^(SUCCESS|SUCCESSFULLY|BUILD SUCCESS)/.test(upper)) return 'green'
  if (/^(ERROR|FAILED|FAILURE)/.test(upper)) return 'red'
  if (/^(WARN|WARNING)/.test(upper)) return 'yellow'
  // Check for ANSI-like color codes embedded in Docker output
  if (line.includes('[A;') || line.includes('1;32m')) return 'green'
  if (line.includes('1;31m') || line.includes('0;31m')) return 'red'
  if (line.includes('1;33m') || line.includes('0;33m')) return 'yellow'
  if (line.includes('1;36m') || line.includes('0;36m')) return 'cyan'
  return 'default'
}

// ─── Build execution ─────────────────────────────────────────────────────

async function handleBuild() {
  if (!formValid.value || building.value) return

  // Reset state
  status.value = 'building'
  logLines.value = []
  buildError.value = null
  builtImageId.value = null
  builtTags.value = []
  progressStep.value = 0
  progressTotal.value = 0

  // Prepare build args as KEY=VALUE strings
  const argStrings = buildArgs.value
    .filter(a => a.key.trim())
    .map(a => `${a.key.trim()}=${a.value.trim()}`)

  // Subscribe to events
  try {
    unlistenLog = await listenBuildLog((log: BuildLogLine) => {
      logLines.value.push(log)
      parseProgress(log.line)
    })

    unlistenComplete = await listenBuildComplete((payload: BuildCompletePayload) => {
      if (payload.success) {
        status.value = 'success'
        builtImageId.value = payload.image_id
        builtTags.value = payload.tags
        notifySuccess('Image built', `Image ${payload.image_id ?? ''} built successfully.`)
      } else {
        status.value = 'error'
        buildError.value = payload.error ?? 'Unknown build error'
        notifyError('Build failed', buildError.value)
      }
    })

    // Invoke the build command
    const imageId = await buildImage({
      dockerfilePath: dockerfilePath.value,
      contextDir: contextDir.value,
      tags: tags.value,
      buildArgs: argStrings,
      noCache: noCache.value,
      pullBase: pullBase.value,
    })

    // If we got here without build-complete, set success
    if (status.value === 'building') {
      status.value = 'success'
      builtImageId.value = imageId
      builtTags.value = tags.value
      notifySuccess('Image built', `Image ${imageId} built successfully.`)
    }
  } catch (e: any) {
    // Error already handled via build-complete or invoke rejection
    if (status.value === 'building') {
      status.value = 'error'
      buildError.value = e?.toString() ?? 'Build failed'
      notifyError('Build failed', buildError.value)
    }
  }
}

function handleRetry() {
  status.value = 'idle'
}

function goToImages() {
  router.push('/images')
}

// ─── Cleanup ──────────────────────────────────────────────────────────────

onUnmounted(() => {
  unlistenLog?.()
  unlistenComplete?.()
})
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span>Images</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Build</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px;">
    <h1 class="text-h1">Build Image</h1>
  </div>

  <!-- Split Layout -->
  <div class="build-split">
    <!-- Left Panel: Configuration Form -->
    <div class="build-config">
      <div class="section">
        <div class="section-header">
          <span class="section-title">Build Configuration</span>
        </div>
        <div class="build-config-body">
          <!-- Dockerfile -->
          <div class="form-group">
            <label class="form-label">Dockerfile</label>
            <div class="input-with-btn">
              <input
                class="form-input mono"
                v-model="dockerfilePath"
                placeholder="/path/to/Dockerfile"
                :disabled="building"
              />
              <button class="btn btn-secondary btn-sm" @click="browseDockerfile" :disabled="building">
                <i class="fa-solid fa-folder-open"></i> Browse
              </button>
            </div>
          </div>

          <!-- Context Directory -->
          <div class="form-group">
            <label class="form-label">Build Context</label>
            <div class="input-with-btn">
              <input
                class="form-input mono"
                v-model="contextDir"
                placeholder="/path/to/context"
                :disabled="building"
              />
              <button class="btn btn-secondary btn-sm" @click="browseContextDir" :disabled="building">
                <i class="fa-solid fa-folder-open"></i> Browse
              </button>
            </div>
          </div>

          <!-- Tags -->
          <div class="form-group">
            <label class="form-label">Tags</label>
            <div class="tags-input-row">
              <input
                class="form-input mono"
                v-model="newTag"
                placeholder="myimage:latest"
                @keydown="handleTagKeydown"
                :disabled="building"
              />
              <button class="btn btn-secondary btn-sm" @click="addTag" :disabled="building || !newTag.trim()">
                <i class="fa-solid fa-plus"></i>
              </button>
            </div>
            <div v-if="tags.length > 0" class="tag-list">
              <span v-for="(tag, i) in tags" :key="i" class="build-tag">
                <span>{{ tag }}</span>
                <button class="tag-remove" @click="removeTag(i)" :disabled="building" title="Remove tag">
                  <i class="fa-solid fa-xmark"></i>
                </button>
              </span>
            </div>
            <div v-else class="form-hint">
              Optional. Leave empty for untagged build.
            </div>
          </div>

          <!-- Build Args -->
          <div class="form-group">
            <label class="form-label">Build Arguments</label>
            <div v-for="(arg, i) in buildArgs" :key="i" class="build-arg-row">
              <input
                class="form-input mono arg-key"
                v-model="arg.key"
                placeholder="KEY"
                :disabled="building"
              />
              <span class="arg-eq">=</span>
              <input
                class="form-input mono arg-val"
                v-model="arg.value"
                placeholder="value"
                :disabled="building"
              />
              <button class="btn btn-ghost btn-sm" @click="removeBuildArg(i)" :disabled="building">
                <i class="fa-solid fa-trash-can"></i>
              </button>
            </div>
            <button class="btn btn-secondary btn-sm" @click="addBuildArg" :disabled="building">
              <i class="fa-solid fa-plus"></i> Add Argument
            </button>
          </div>

          <!-- Options -->
          <div class="form-group">
            <label class="form-label">Options</label>
            <div class="build-options">
              <label class="checkbox-label">
                <input type="checkbox" v-model="noCache" :disabled="building" />
                <span>No cache</span>
              </label>
              <label class="checkbox-label">
                <input type="checkbox" v-model="pullBase" :disabled="building" />
                <span>Pull base image</span>
              </label>
            </div>
          </div>

          <!-- Build Button -->
          <div class="build-actions">
            <button
              v-if="status !== 'building'"
              class="btn btn-primary build-btn"
              @click="handleBuild"
              :disabled="!formValid"
            >
              <i class="fa-solid fa-hammer"></i> Build
            </button>
            <button
              v-if="status === 'error'"
              class="btn btn-secondary"
              @click="handleRetry"
            >
              <i class="fa-solid fa-rotate"></i> Reset
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Right Panel: Build Log Terminal -->
    <div class="build-terminal-panel">
      <div class="section terminal-section">
        <div class="section-header terminal-header">
          <span class="section-title">Build Log</span>
          <div class="terminal-status-badge">
            <span v-if="status === 'idle'" class="status-label idle">Ready</span>
            <span v-else-if="status === 'building'" class="status-label building">
              <i class="fa-solid fa-spinner fa-spin"></i> Building...
            </span>
            <span v-else-if="status === 'success'" class="status-label success">
              <i class="fa-solid fa-circle-check"></i> Success
            </span>
            <span v-else-if="status === 'error'" class="status-label error">
              <i class="fa-solid fa-circle-xmark"></i> Failed
            </span>
          </div>
        </div>

        <!-- Progress Bar -->
        <div v-if="progressTotal > 0" class="build-progress">
          <div class="progress-bar-bg">
            <div
              class="progress-bar-fill"
              :style="{ width: (progressStep / progressTotal * 100) + '%' }"
            ></div>
          </div>
          <span class="progress-text">Step {{ progressStep }}/{{ progressTotal }}</span>
        </div>

        <!-- Terminal Log -->
        <div ref="logContainer" class="terminal-log">
          <!-- Idle state -->
          <div v-if="logLines.length === 0 && status === 'idle'" class="terminal-placeholder">
            <i class="fa-solid fa-hammer"></i>
            <p>Configure your build options and click <strong>Build</strong> to start.</p>
            <p class="placeholder-hint">The build log will appear here in real time.</p>
          </div>

          <!-- Log lines -->
          <div
            v-for="(log, i) in logLines"
            :key="i"
            :class="['log-line', 'log-' + getLineColor(log.line), log.stream === 'stderr' ? 'log-stderr' : '']"
          >
            <span class="log-text">{{ log.line }}</span>
          </div>

          <!-- Building indicator -->
          <div v-if="status === 'building'" class="log-line building-indicator">
            <span class="log-cursor"></span>
          </div>

          <!-- Success state -->
          <div v-if="status === 'success'" class="log-result success-result">
            <div class="result-icon"><i class="fa-solid fa-circle-check"></i></div>
            <div class="result-info">
              <p class="result-title">Build Successful</p>
              <p v-if="builtImageId" class="result-id mono">Image ID: {{ builtImageId }}</p>
              <div v-if="builtTags.length > 0" class="result-tags">
                <span v-for="tag in builtTags" :key="tag" class="build-tag result-tag">{{ tag }}</span>
              </div>
              <button class="btn btn-primary btn-sm" @click="goToImages" style="margin-top:12px;">
                <i class="fa-solid fa-eye"></i> View Images
              </button>
            </div>
          </div>

          <!-- Error state -->
          <div v-if="status === 'error'" class="log-result error-result">
            <div class="result-icon error"><i class="fa-solid fa-circle-xmark"></i></div>
            <div class="result-info">
              <p class="result-title">Build Failed</p>
              <p v-if="buildError" class="result-error-detail">{{ buildError }}</p>
              <button class="btn btn-secondary btn-sm" @click="handleRetry" style="margin-top:12px;">
                <i class="fa-solid fa-rotate"></i> Retry
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ─── Split Layout ─── */
.build-split {
  display: flex;
  gap: 16px;
  flex: 1;
  min-height: 0;
  height: calc(100vh - var(--header-height) - 100px);
}

.build-config {
  flex: 0 0 40%;
  max-width: 40%;
  overflow-y: auto;
}

.build-terminal-panel {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

/* ─── Config Form Body ─── */
.build-config-body {
  padding: 20px;
}

/* ─── Input with button row ─── */
.input-with-btn {
  display: flex;
  gap: 8px;
  align-items: center;
}
.input-with-btn .form-input {
  flex: 1;
}

/* ─── Tags input row ─── */
.tags-input-row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.tags-input-row .form-input {
  flex: 1;
}

/* ─── Tag list ─── */
.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}
.build-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  background: rgba(0, 229, 255, 0.08);
  border: 1px solid rgba(0, 229, 255, 0.15);
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--accent-cyan);
}
.tag-remove {
  background: none;
  border: none;
  color: var(--text-disabled);
  cursor: pointer;
  font-size: 12px;
  padding: 0 2px;
  display: flex;
  align-items: center;
  transition: color var(--transition-fast);
}
.tag-remove:hover {
  color: var(--accent-red);
}

/* ─── Build Args ─── */
.build-arg-row {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 8px;
}
.arg-key {
  flex: 0 0 120px;
  font-family: var(--font-mono);
  text-transform: uppercase;
}
.arg-eq {
  color: var(--text-muted);
  font-family: var(--font-mono);
  font-size: 13px;
}
.arg-val {
  flex: 1;
  font-family: var(--font-mono);
}

/* ─── Checkboxes ─── */
.build-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text-main);
  cursor: pointer;
}
.checkbox-label input[type="checkbox"] {
  accent-color: var(--accent-cyan);
}

/* ─── Hint ─── */
.form-hint {
  font-size: 11px;
  color: var(--text-disabled);
  margin-top: 4px;
}

/* ─── Build Actions ─── */
.build-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}
.build-btn {
  min-width: 120px;
  justify-content: center;
}

/* ─── Terminal Section ─── */
.terminal-section {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}
.terminal-header {
  flex-shrink: 0;
}

/* ─── Status badges ─── */
.terminal-status-badge {
  display: flex;
  align-items: center;
}
.status-label {
  font-size: 12px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  gap: 5px;
}
.status-label.idle {
  background: var(--bg-tertiary);
  color: var(--text-muted);
}
.status-label.building {
  background: rgba(0, 229, 255, 0.08);
  color: var(--accent-cyan);
}
.status-label.success {
  background: rgba(16, 185, 129, 0.08);
  color: var(--accent-green);
}
.status-label.error {
  background: rgba(239, 68, 68, 0.08);
  color: var(--accent-red);
}

/* ─── Progress Bar ─── */
.build-progress {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 20px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
}
.progress-bar-bg {
  flex: 1;
  height: 4px;
  background: var(--bg-hover);
  border-radius: 2px;
  overflow: hidden;
}
.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent-cyan), var(--accent-purple));
  border-radius: 2px;
  transition: width 0.3s ease;
}
.progress-text {
  font-family: var(--font-mono);
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
}

/* ─── Terminal Log ─── */
.terminal-log {
  flex: 1;
  overflow-y: auto;
  background: var(--terminal-bg);
  padding: 16px;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.6;
  min-height: 300px;
}

/* ─── Placeholder ─── */
.terminal-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 200px;
  color: var(--text-disabled);
  text-align: center;
  gap: 8px;
}
.terminal-placeholder i {
  font-size: 32px;
  opacity: 0.3;
  margin-bottom: 8px;
}
.terminal-placeholder p {
  font-size: 13px;
  font-family: var(--font-sans);
  color: var(--text-disabled);
}
.placeholder-hint {
  font-size: 11px !important;
  opacity: 0.6;
}

/* ─── Log Lines ─── */
.log-line {
  white-space: pre-wrap;
  word-break: break-all;
  padding: 0;
  line-height: 1.5;
}
.log-stderr {
  opacity: 0.85;
}
.log-default { color: #e0e0e0; }
.log-cyan { color: #00e5ff; }
.log-green { color: #10b981; }
.log-red { color: #ef4444; }
.log-yellow { color: #f59e0b; }

/* ─── Building Cursor ─── */
.building-indicator {
  display: flex;
  align-items: center;
  padding: 4px 0;
}
.log-cursor {
  display: inline-block;
  width: 8px;
  height: 14px;
  background: var(--accent-cyan);
  animation: blink 1s step-end infinite;
}
@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0; }
}

/* ─── Result Blocks ─── */
.log-result {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  margin-top: 16px;
  padding: 16px;
  border-radius: var(--radius-md);
}
.success-result {
  background: rgba(16, 185, 129, 0.06);
  border: 1px solid rgba(16, 185, 129, 0.15);
}
.error-result {
  background: rgba(239, 68, 68, 0.06);
  border: 1px solid rgba(239, 68, 68, 0.15);
}
.result-icon {
  font-size: 28px;
  color: var(--accent-green);
  flex-shrink: 0;
}
.result-icon.error {
  color: var(--accent-red);
}
.result-info {
  flex: 1;
}
.result-title {
  font-family: var(--font-sans);
  font-size: 15px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 4px;
}
.result-id {
  font-size: 12px;
  color: var(--accent-cyan);
  word-break: break-all;
}
.result-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}
.result-tag {
  font-size: 10px;
  padding: 2px 6px;
}
.result-error-detail {
  font-size: 12px;
  color: var(--accent-red);
  font-family: var(--font-mono);
  word-break: break-all;
  margin-top: 4px;
}
</style>
