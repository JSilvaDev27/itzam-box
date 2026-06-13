<!-- ItzamBox — Dashboard View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref } from 'vue'
import { useDocker } from '../composables/useDocker'
import { useContextMenu, containerContextMenu } from '../composables/useContextMenu'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const { containers, images, volumes, hostMetrics, loading, error, refreshAll,
        startContainer, stopContainer, restartContainer, removeContainer } = useDocker()
const { show } = useContextMenu()

const firstLoadDone = ref(false)

let interval: number | undefined

onMounted(async () => {
  await refreshAll()
  firstLoadDone.value = true
  interval = window.setInterval(() => refreshAll(), 5000)
})

onUnmounted(() => {
  if (interval) clearInterval(interval)
})

const runningContainers = computed(() => containers.value.filter(c => c.state === 'running'))
const stoppedContainers = computed(() => containers.value.filter(c => c.state === 'exited' || c.state === 'stopped'))
const memoryPercent = computed(() => {
  if (!hostMetrics.value) return 0
  return (hostMetrics.value.memory_used_bytes / hostMetrics.value.memory_total_bytes * 100).toFixed(0)
})
const uptimeDays = computed(() => {
  if (!hostMetrics.value) return '--'
  const days = Math.floor(hostMetrics.value.uptime_seconds / 86400)
  const hours = Math.floor((hostMetrics.value.uptime_seconds % 86400) / 3600)
  return `${days}d ${hours}h`
})
const isEmpty = computed(() => containers.value.length === 0 && !loading.value && !error.value)

function formatBytes(bytes: number): string {
  if (bytes > 1e9) return (bytes / 1e9).toFixed(1) + ' GB'
  if (bytes > 1e6) return Math.round(bytes / 1e6) + ' MB'
  return Math.round(bytes / 1e3) + ' KB'
}

function onContainerContextMenu(e: MouseEvent, c: typeof containers.value[0]) {
  show(e, containerContextMenu(c))
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Dashboard</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Dashboard</h1>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary" @click="refreshAll" :disabled="loading">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> {{ loading ? 'Loading...' : 'Refresh' }}
      </button>
    </div>
  </div>

  <!-- Loading state (A.6.1) — only on first load, not on refresh -->
  <SkeletonLoader v-if="loading && !firstLoadDone" variant="metric-grid" :count="4" />
  <SkeletonLoader v-if="loading && !firstLoadDone" variant="table-row" :rows="3" />

  <!-- Error state (A.6.3) — only if we've loaded at least once -->
  <ErrorState
    v-if="error && firstLoadDone"
    :message="'Error connecting to Docker'"
    :suggestion="'Make sure Docker is running. Try: sudo systemctl start docker'"
    :detail="error"
    detail-label="Error details"
    icon="fa-solid fa-circle-exclamation"
    @retry="refreshAll"
  />

  <!-- Metric Cards — keep visible during refresh, fade slightly -->
  <div v-show="firstLoadDone && !error" class="metrics-grid" :style="{ opacity: loading ? 0.6 : 1, transition: 'opacity 0.2s' }">
    <div class="metric-card">
      <div class="metric-icon green"><i class="fa-solid fa-cubes"></i></div>
      <div class="metric-label">Active Containers</div>
      <div class="metric-value" style="color:var(--accent-green)">{{ runningContainers.length }}</div>
      <div class="metric-delta">{{ containers.length }} total</div>
    </div>
    <div class="metric-card">
      <div class="metric-icon cyan"><i class="fa-solid fa-layer-group"></i></div>
      <div class="metric-label">Local Images</div>
      <div class="metric-value">{{ images.length }}</div>
      <div class="metric-delta">{{ volumes.length }} volumes</div>
    </div>
    <div class="metric-card">
      <div class="metric-icon purple"><i class="fa-solid fa-microchip"></i></div>
      <div class="metric-label">CPU Usage</div>
      <div class="metric-value" :style="{ color: (hostMetrics?.cpu_usage_percent ?? 0) > 80 ? 'var(--accent-red)' : 'var(--accent-green)' }">
        {{ hostMetrics?.cpu_usage_percent.toFixed(1) ?? '--' }}%
      </div>
      <div class="metric-delta">{{ hostMetrics?.cpu_cores ?? '--' }} cores</div>
    </div>
    <div class="metric-card">
      <div class="metric-icon amber"><i class="fa-solid fa-memory"></i></div>
      <div class="metric-label">Memory</div>
      <div class="metric-value">{{ memoryPercent }}%</div>
      <div class="metric-delta" v-if="hostMetrics">
        {{ formatBytes(hostMetrics.memory_used_bytes) }} / {{ formatBytes(hostMetrics.memory_total_bytes) }}
      </div>
    </div>
  </div>

  <!-- Host Info — keep visible during refresh -->
  <div v-show="hostMetrics && firstLoadDone && !error" style="display:grid;grid-template-columns:1fr 1fr;gap:16px;" :style="{ opacity: loading ? 0.6 : 1, transition: 'opacity 0.2s' }">
    <div class="section">
      <div class="section-header"><span class="section-title"><i class="fa-solid fa-server" style="color:var(--accent-cyan);margin-right:8px"></i> Host</span></div>
      <div style="padding:16px;font-size:13px;display:flex;flex-direction:column;gap:6px;">
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Hostname</span><span style="font-family:var(--font-mono)">{{ hostMetrics.hostname }}</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">OS</span><span>{{ hostMetrics.os_name }}</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Kernel</span><span style="font-family:var(--font-mono)">{{ hostMetrics.kernel_version }}</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Uptime</span><span style="font-family:var(--font-mono)">{{ uptimeDays }}</span></div>
      </div>
    </div>
    <div class="section">
      <div class="section-header"><span class="section-title"><i class="fa-solid fa-docker" style="color:var(--accent-cyan);margin-right:8px"></i> Docker</span></div>
      <div style="padding:16px;font-size:13px;display:flex;flex-direction:column;gap:6px;">
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Containers</span><span>{{ runningContainers.length }} running, {{ stoppedContainers.length }} stopped</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Images</span><span>{{ images.length }}</span></div>
        <div style="display:flex;justify-content:space-between"><span style="color:var(--text-muted)">Volumes</span><span>{{ volumes.length }}</span></div>
      </div>
    </div>
  </div>

  <!-- Recent Containers — keep visible during refresh -->
  <div v-show="containers.length > 0 && firstLoadDone && !error" class="section" :style="{ opacity: loading ? 0.6 : 1, transition: 'opacity 0.2s' }">
    <div class="section-header">
      <span class="section-title">Containers</span>
      <div class="table-toolbar">
        <span style="font-size:12px;color:var(--text-muted)">{{ containers.length }} total</span>
      </div>
    </div>
    <div v-for="c in containers.slice(0, 10)" :key="c.id" class="data-row" @contextmenu="onContainerContextMenu($event, c)">
      <span :class="['status-dot', c.state === 'running' ? 'status-dot--running' : c.state === 'paused' ? 'status-dot--paused' : 'status-dot--stopped']"></span>
      <div class="row-info">
        <div class="row-name">{{ c.name }}</div>
        <div class="row-meta">{{ c.image }} · {{ c.status }}</div>
      </div>
      <span :class="['tag', c.state === 'running' ? 'tag running' : c.state === 'paused' ? 'tag paused' : 'tag stopped']">{{ c.state }}</span>
      <div class="row-actions">
        <button v-if="c.state === 'stopped' || c.state === 'exited'" class="action-btn" @click.stop="startContainer(c.id)" title="Start"><i class="fa-solid fa-play"></i></button>
        <button v-if="c.state === 'running'" class="action-btn" @click.stop="stopContainer(c.id)" title="Stop"><i class="fa-solid fa-stop"></i></button>
        <button v-if="c.state === 'running'" class="action-btn" @click.stop="restartContainer(c.id)" title="Restart"><i class="fa-solid fa-rotate-right"></i></button>
        <button v-if="c.state !== 'running'" class="action-btn" @click.stop="removeContainer(c.id, true)" title="Remove"><i class="fa-solid fa-trash-can"></i></button>
      </div>
    </div>
  </div>

  <!-- Empty state (A.6.2) — only if fully loaded and truly empty -->
  <EmptyState
    v-if="isEmpty && firstLoadDone"
    icon="fa-solid fa-docker"
    title="No containers running"
    description="Pull an image and create your first container to get started."
    action-label="Pull Image"
    secondary-label="View Images"
    @action="$router.push('/images')"
    @secondary="$router.push('/images')"
  />
</template>
