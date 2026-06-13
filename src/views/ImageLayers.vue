<!-- ItzamBox — Image Layers View
     Copyright (C) 2026 SodigTech — GPL-3.0
     T-045: Vertical timeline of image layers with size analysis -->
<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useDocker, type ImageLayerInfo, type ImageInfo, formatFileSize } from '../composables/useDocker'
import { useNotifications } from '../composables/useNotifications'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const route = useRoute()
const router = useRouter()
const { success, error: notifyError } = useNotifications()
const { images, getImageHistory } = useDocker()

// ─── Route params ───
const imageId = computed(() => route.params.id as string)

// ─── Find image from store ───
const image = computed<ImageInfo | null>(() => {
  return images.value.find(img => img.id === imageId.value) ?? null
})

// ─── State ───
const layers = ref<ImageLayerInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const expandedLayerIndex = ref<number | null>(null)
const copiedIndex = ref<number | null>(null)
const dockerfileCopied = ref(false)
const historyCopied = ref(false)

// ─── Total size from layers (sum of all layer sizes) ───
const totalLayerSize = computed(() => {
  return layers.value.reduce((sum, l) => sum + l.size_bytes, 0)
})

// ─── Layer count ───
const layerCount = computed(() => layers.value.length)

// ─── Empty? ───
const isEmpty = computed(() => layers.value.length === 0 && !loading.value && !error.value)

// ─── Load data ───
onMounted(() => loadData())

async function loadData() {
  loading.value = true
  error.value = null
  try {
    layers.value = await getImageHistory(imageId.value)
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

// ─── Layer size percentage ───
function layerPercentage(layer: ImageLayerInfo): number {
  const total = totalLayerSize.value
  if (total === 0) return 0
  return (layer.size_bytes / total) * 100
}

// ─── Format layer size ───
function formatLayerSize(bytes: number): string {
  return formatFileSize(bytes)
}

// ─── Layer size category ───
type SizeCategory = 'small' | 'medium' | 'large' | 'very-large'
function sizeCategory(bytes: number): SizeCategory {
  if (bytes > 500_000_000) return 'very-large'
  if (bytes > 100_000_000) return 'large'
  if (bytes > 10_000_000) return 'medium'
  return 'small'
}

// ─── Format digest (truncated) ───
function truncateDigest(digest: string): string {
  if (!digest) return '<missing>'
  if (digest.startsWith('sha256:')) {
    return digest.slice(0, 19) + '…'
  }
  return digest.slice(0, 12) + '…'
}

// ─── Format date from unix epoch ───
function formatDate(unixSeconds: number): string {
  if (!unixSeconds) return '—'
  const d = new Date(unixSeconds * 1000)
  return d.toLocaleDateString(undefined, {
    year: 'numeric', month: 'short', day: 'numeric',
    hour: '2-digit', minute: '2-digit',
  })
}

// ─── Toggle command expansion ───
function toggleExpand(index: number) {
  expandedLayerIndex.value = expandedLayerIndex.value === index ? null : index
}

// ─── Truncate command to 80 chars ───
function truncateCommand(cmd: string, expanded: boolean): string {
  if (!cmd) return '(empty)'
  if (expanded) return cmd
  if (cmd.length > 80) return cmd.slice(0, 80) + '…'
  return cmd
}

// ─── Copy digest to clipboard ───
async function copyDigest(digest: string, index: number) {
  try {
    await navigator.clipboard.writeText(digest)
    copiedIndex.value = index
    setTimeout(() => { copiedIndex.value = index; setTimeout(() => { copiedIndex.value = null }, 300) }, 1500)
    success('Copied', 'Layer digest copied to clipboard.')
  } catch {
    notifyError('Copy failed', 'Could not copy to clipboard.')
  }
}

// ─── Copy Dockerfile ───
async function copyDockerfile() {
  const lines = layers.value
    .map(l => l.command)
    .filter(cmd => cmd.trim().length > 0)
    .reverse() // Dockerfile is built bottom-up; history is top-down most recent first
  
  // Reconstruct approximate Dockerfile
  const dockerfile = lines.join('\n')

  try {
    await navigator.clipboard.writeText(dockerfile)
    dockerfileCopied.value = true
    setTimeout(() => { dockerfileCopied.value = false }, 2000)
    success('Dockerfile copied', 'Approximate Dockerfile reconstructed from layer commands.')
  } catch {
    notifyError('Copy failed', 'Could not copy Dockerfile to clipboard.')
  }
}

// ─── Export History ───
async function exportHistory() {
  const lines = layers.value.map((l, i) => {
    const size = formatLayerSize(l.size_bytes)
    const date = formatDate(l.created_at)
    return `Layer ${i + 1}: ${size} | ${l.digest.slice(0, 19)}… | ${date}\n  Command: ${l.command || '(empty)'}`
  })
  const header = `Image History: ${image.value?.repository ?? 'unknown'}:${image.value?.tag ?? 'unknown'} (${imageId.value})\nTotal layers: ${layerCount.value}\nTotal size: ${formatLayerSize(totalLayerSize.value)}\n${'─'.repeat(60)}\n\n`
  const text = header + lines.join('\n\n')

  try {
    await navigator.clipboard.writeText(text)
    historyCopied.value = true
    setTimeout(() => { historyCopied.value = false }, 2000)
    success('History exported', 'Full layer history copied to clipboard.')
  } catch {
    notifyError('Copy failed', 'Could not copy history to clipboard.')
  }
}
</script>

<template>
  <!-- Breadcrumb -->
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i>
    <router-link to="/images" style="color:var(--text-muted);text-decoration:none;">Images</router-link>
    <i class="fa-solid fa-chevron-right"></i>
    <span class="current">Layers</span>
  </div>

  <!-- Loading state (A.6.1) -->
  <SkeletonLoader v-if="loading && layers.length === 0" variant="card" :rows="6" />

  <!-- Error state (A.6.3) -->
  <ErrorState
    v-if="error"
    :message="'Failed to load image layers'"
    :suggestion="'Verify the image ID is correct and Docker daemon is running.'"
    :detail="error"
    @retry="loadData"
  />

  <!-- Content -->
  <template v-if="!loading && !error">
    <!-- Empty state (A.6.2) -->
    <EmptyState
      v-if="isEmpty"
      icon="fa-solid fa-layer-group"
      title="No layers found"
      description="This image has no layers or the history data is unavailable."
      action-label="Back to Images"
      @action="router.push('/images')"
    />

    <template v-else>
      <!-- ─── Header ─── -->
      <div class="layers-header">
        <div class="layers-header-top">
          <div>
            <h1 class="text-h1" style="margin-bottom:4px;">
              {{ image?.repository ?? 'Unknown' }}
              <span class="layers-tag">{{ image?.tag ?? '?' }}</span>
            </h1>
            <div class="layers-meta">
              <span class="text-mono" style="font-size:11px;color:var(--text-muted);">
                <i class="fa-solid fa-fingerprint"></i> {{ image?.id ?? imageId }}
              </span>
            </div>
          </div>
          <div class="layers-stats">
            <div class="layer-stat">
              <span class="layer-stat-value">{{ formatLayerSize(image?.size_bytes ?? totalLayerSize) }}</span>
              <span class="layer-stat-label">Total Size</span>
            </div>
            <div class="layer-stat">
              <span class="layer-stat-value">{{ layerCount }}</span>
              <span class="layer-stat-label">Layers</span>
            </div>
          </div>
        </div>
      </div>

      <!-- ─── Action Bar ─── -->
      <div class="layers-actions">
        <button class="btn btn-secondary" @click="copyDockerfile" :disabled="layerCount === 0">
          <i :class="dockerfileCopied ? 'fa-solid fa-check' : 'fa-solid fa-file-lines'"></i>
          {{ dockerfileCopied ? 'Copied!' : 'Copy Dockerfile' }}
        </button>
        <button class="btn btn-secondary" @click="exportHistory" :disabled="layerCount === 0">
          <i :class="historyCopied ? 'fa-solid fa-check' : 'fa-solid fa-clipboard-list'"></i>
          {{ historyCopied ? 'Copied!' : 'Export History' }}
        </button>
      </div>

      <!-- ─── Layer Timeline ─── -->
      <div class="section">
        <div class="section-header">
          <span class="section-title">
            <i class="fa-solid fa-clock-rotate-left" style="margin-right:6px;color:var(--accent-cyan);"></i>
            Layer History ({{ layerCount }})
          </span>
        </div>

        <div class="layers-timeline">
          <div
            v-for="(layer, index) in layers"
            :key="layer.digest + index"
            class="layer-row"
          >
            <!-- Timeline connector -->
            <div class="timeline-connector">
              <div class="timeline-dot"></div>
              <div v-if="index < layers.length - 1" class="timeline-line"></div>
            </div>

            <!-- Layer content -->
            <div class="layer-content">
              <div class="layer-header-row">
                <div class="layer-size-section">
                  <span class="layer-badge" :class="{
                    'badge-sm': true,
                    'badge-large': sizeCategory(layer.size_bytes) === 'large',
                    'badge-very-large': sizeCategory(layer.size_bytes) === 'very-large',
                  }">
                    {{ formatLayerSize(layer.size_bytes) }}
                  </span>

                  <!-- Size warning badges -->
                  <span v-if="sizeCategory(layer.size_bytes) === 'large'" class="badge-warning">
                    <i class="fa-solid fa-triangle-exclamation"></i> Large layer
                  </span>
                  <span v-if="sizeCategory(layer.size_bytes) === 'very-large'" class="badge-danger">
                    <i class="fa-solid fa-circle-exclamation"></i> Very large layer
                  </span>
                </div>

                <span class="layer-date">{{ formatDate(layer.created_at) }}</span>
              </div>

              <!-- Proportion bar -->
              <div class="proportion-bar-track">
                <div
                  class="proportion-bar-fill"
                  :style="{ width: layerPercentage(layer) + '%' }"
                  :class="{
                    'fill-large': sizeCategory(layer.size_bytes) === 'large',
                    'fill-very-large': sizeCategory(layer.size_bytes) === 'very-large',
                  }"
                ></div>
              </div>
              <div class="proportion-label">
                {{ layerPercentage(layer).toFixed(1) }}% of total image size
              </div>

              <!-- Command -->
              <div class="layer-command" @click="toggleExpand(index)">
                <i class="fa-solid fa-terminal" style="margin-right:6px;color:var(--text-disabled);font-size:11px;"></i>
                <code class="command-text">{{ truncateCommand(layer.command, expandedLayerIndex === index) }}</code>
                <button v-if="layer.command.length > 80" class="expand-btn" @click.stop="toggleExpand(index)">
                  <i :class="expandedLayerIndex === index ? 'fa-solid fa-chevron-up' : 'fa-solid fa-chevron-down'" style="font-size:10px;"></i>
                </button>
              </div>

              <!-- Digest -->
              <div class="layer-digest-row">
                <span class="digest-label">Digest:</span>
                <code class="digest-hash" :title="layer.digest">{{ truncateDigest(layer.digest) }}</code>
                <button class="copy-btn" @click="copyDigest(layer.digest, index)" :title="'Copy full digest: ' + layer.digest">
                  <i :class="copiedIndex === index ? 'fa-solid fa-check' : 'fa-solid fa-copy'"></i>
                </button>
              </div>

              <!-- Layer number -->
              <div class="layer-number">Layer {{ layers.length - index }}</div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </template>
</template>

<style scoped>
/* ─── Layout ─── */
.layers-header {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 24px;
}

.layers-header-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 20px;
}

.layers-tag {
  display: inline-block;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  padding: 2px 8px;
  font-family: var(--font-mono);
  font-size: 13px;
  color: var(--accent-cyan);
  vertical-align: middle;
  margin-left: 8px;
}

.layers-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 4px;
}

/* ─── Stat pills ─── */
.layers-stats {
  display: flex;
  gap: 16px;
  flex-shrink: 0;
}

.layer-stat {
  text-align: center;
  padding: 12px 20px;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  min-width: 100px;
}

.layer-stat-value {
  display: block;
  font-size: 1.5rem;
  font-weight: 700;
  font-family: var(--font-mono);
  color: var(--accent-cyan);
  letter-spacing: -0.03em;
  line-height: 1.2;
}

.layer-stat-label {
  display: block;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-top: 4px;
}

/* ─── Actions ─── */
.layers-actions {
  display: flex;
  gap: 8px;
}

/* ─── Timeline ─── */
.layers-timeline {
  padding: 8px 0;
}

.layer-row {
  display: flex;
  gap: 16px;
  padding: 0 20px;
  transition: background var(--transition-fast);
}

.layer-row:hover {
  background: var(--bg-hover);
}

/* ─── Timeline Connector ─── */
.timeline-connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  min-width: 20px;
  padding-top: 20px;
}

.timeline-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--accent-cyan);
  border: 2px solid var(--bg-secondary);
  box-shadow: 0 0 0 2px var(--accent-cyan);
  flex-shrink: 0;
  z-index: 1;
}

.timeline-line {
  width: 2px;
  flex: 1;
  min-height: 24px;
  background: linear-gradient(180deg, var(--border-color), var(--border-light));
  margin: 2px 0;
}

/* ─── Layer Content ─── */
.layer-content {
  flex: 1;
  min-width: 0;
  padding: 14px 0;
  border-bottom: 1px solid var(--border-light);
  position: relative;
}

.layer-row:last-child .layer-content {
  border-bottom: none;
}

.layer-header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 8px;
}

.layer-size-section {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.layer-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 4px;
  font-family: var(--font-mono);
  font-size: 12px;
  font-weight: 600;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  color: var(--text-main);
}

.badge-large {
  background: rgba(245, 158, 11, 0.1);
  border-color: rgba(245, 158, 11, 0.3);
  color: var(--accent-yellow);
}

.badge-very-large {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: var(--accent-red);
}

.badge-warning {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  background: rgba(245, 158, 11, 0.08);
  border: 1px solid rgba(245, 158, 11, 0.2);
  color: var(--accent-yellow);
}

.badge-danger {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: var(--accent-red);
}

.layer-date {
  font-size: 11px;
  color: var(--text-muted);
  white-space: nowrap;
  font-family: var(--font-mono);
}

/* ─── Proportion Bar ─── */
.proportion-bar-track {
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 2px;
}

.proportion-bar-fill {
  height: 100%;
  border-radius: 3px;
  background: var(--accent-cyan);
  opacity: 0.6;
  transition: width 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.proportion-bar-fill.fill-large {
  background: var(--accent-yellow);
  opacity: 0.7;
}

.proportion-bar-fill.fill-very-large {
  background: var(--accent-red);
  opacity: 0.7;
}

.proportion-label {
  font-size: 10px;
  color: var(--text-disabled);
  margin-bottom: 8px;
  font-family: var(--font-mono);
}

/* ─── Command ─── */
.layer-command {
  display: flex;
  align-items: flex-start;
  gap: 4px;
  padding: 8px 10px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  margin-bottom: 8px;
  cursor: pointer;
  transition: border-color var(--transition-fast);
}

.layer-command:hover {
  border-color: var(--accent-cyan);
}

.command-text {
  flex: 1;
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
  word-break: break-all;
  white-space: pre-wrap;
}

.expand-btn {
  background: none;
  border: none;
  color: var(--text-disabled);
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 3px;
  flex-shrink: 0;
  margin-top: 2px;
  transition: all var(--transition-fast);
}

.expand-btn:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

/* ─── Digest ─── */
.layer-digest-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
}

.digest-label {
  color: var(--text-disabled);
  font-weight: 500;
}

.digest-hash {
  font-family: var(--font-mono);
  color: var(--text-muted);
  font-size: 11px;
}

.copy-btn {
  background: none;
  border: none;
  color: var(--text-disabled);
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 11px;
  transition: all var(--transition-fast);
}

.copy-btn:hover {
  background: var(--bg-hover);
  color: var(--accent-cyan);
}

/* ─── Layer Number (right side, faint) ─── */
.layer-number {
  position: absolute;
  right: 20px;
  font-size: 10px;
  color: var(--text-disabled);
  font-family: var(--font-mono);
  opacity: 0.4;
  margin-top: -4px;
}

/* ─── Responsive helpers ─── */
@media (max-width: 768px) {
  .layers-header-top {
    flex-direction: column;
  }

  .layers-stats {
    width: 100%;
    justify-content: stretch;
  }

  .layer-stat {
    flex: 1;
  }
}
</style>
