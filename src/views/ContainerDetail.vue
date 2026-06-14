<!-- ItzamBox — Container Detail Inspection View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useDocker, type ContainerStats } from '../composables/useDocker'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const route = useRoute()
const router = useRouter()
const containerId = computed(() => route.params.id as string)
const envFilter = ref('')

// Read initial tab from query param (e.g., ?tab=logs, ?tab=info)
const initialTab = (route.query.tab as string) || 'info'
const validTabs = ['info', 'env', 'volumes', 'network', 'health', 'labels', 'logs', 'stats'] as const
type ValidTab = typeof validTabs[number]
const activeTab = ref<ValidTab>(validTabs.includes(initialTab as ValidTab) ? (initialTab as ValidTab) : 'info')
const {
  getContainerLogs, getContainerStats, inspectContainer,
  startContainer, stopContainer, restartContainer, pauseContainer, unpauseContainer, removeContainer,
} = useDocker()

// ─── General state ───
const inspectData = ref<Record<string, any> | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const actionLoading = ref<string | null>(null)

// ─── Logs state ───
const logs = ref('')
const logsLoading = ref(false)
const logsTail = ref(500)
const logsTimestamps = ref(true)
const showStdout = ref(true)
const showStderr = ref(true)
const logsFilter = ref('')
const logContainerRef = ref<HTMLDivElement>()
const logAutoScroll = ref(true)

// ─── Stats state ───
const statsData = ref<ContainerStats | null>(null)
const statsLoading = ref(false)
let statsInterval: number | undefined

// ─── Derived data from inspect ───
const containerName = computed(() => {
  if (!inspectData.value) return 'Loading...'
  return (inspectData.value.Name || '').replace(/^\//, '')
})

const containerState = computed(() => {
  if (!inspectData.value?.State) return 'unknown'
  return inspectData.value.State.Status || 'unknown'
})

const isRunning = computed(() => containerState.value === 'running')
const isPaused = computed(() => containerState.value === 'paused')
const isStopped = computed(() => ['exited', 'stopped', 'dead'].includes(containerState.value))
const isStarting = computed(() => ['created', 'restarting'].includes(containerState.value))

const filteredEnvVars = computed(() => {
  if (!envFilter.value) return envVars.value
  const q = envFilter.value.toLowerCase()
  return envVars.value.filter(e => e.key.toLowerCase().includes(q) || e.value.toLowerCase().includes(q))
})

const envVars = computed(() => {
  if (!inspectData.value?.Config?.Env) return []
  const raw: string[] = inspectData.value.Config.Env
  return raw.map((entry: string) => {
    const idx = entry.indexOf('=')
    if (idx === -1) return { key: entry, value: '' }
    return { key: entry.slice(0, idx), value: entry.slice(idx + 1) }
  })
})

const mounts = computed(() => {
  return inspectData.value?.Mounts || []
})

const networks = computed(() => {
  return inspectData.value?.NetworkSettings?.Networks || {}
})

const labels = computed(() => {
  return inspectData.value?.Config?.Labels || {}
})

const labelsArray = computed(() => {
  return Object.entries(labels.value).map(([key, value]) => ({ key, value: String(value) }))
})

const healthData = computed(() => {
  return inspectData.value?.State?.Health || null
})

const healthStatus = computed(() => {
  if (!healthData.value) return 'none'
  return healthData.value.Status || 'none'
})

const healthLogs = computed(() => {
  return healthData.value?.Log || []
})

const ports = computed(() => {
  return inspectData.value?.NetworkSettings?.Ports || {}
})

const portsArray = computed(() => {
  const result: Array<{ containerPort: string; protocol: string; hostIp: string; hostPort: string }> = []
  for (const [key, bindings] of Object.entries(ports.value)) {
    const [containerPort, protocol] = key.split('/')
    if (Array.isArray(bindings) && bindings.length > 0) {
      for (const b of bindings) {
        result.push({
          containerPort,
          protocol,
          hostIp: (b as any).HostIp || '',
          hostPort: (b as any).HostPort || '',
        })
      }
    } else {
      result.push({ containerPort, protocol, hostIp: '', hostPort: '' })
    }
  }
  return result
})

// ─── Helpers ───
function formatDateTime(iso: string): string {
  if (!iso) return '--'
  try {
    const d = new Date(iso)
    return d.toLocaleString(undefined, {
      year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit', second: '2-digit',
    })
  } catch { return iso }
}

function formatBytes(bytes: number | undefined | null): string {
  if (bytes == null || isNaN(Number(bytes))) return '0 B'
  const b = Number(bytes)
  if (b === 0) return '0 B'
  if (b > 1e12) return (b / 1e12).toFixed(2) + ' TB'
  if (b > 1e9) return (b / 1e9).toFixed(2) + ' GB'
  if (b > 1e6) return (b / 1e6).toFixed(1) + ' MB'
  if (b > 1e3) return (b / 1e3).toFixed(1) + ' KB'
  return b + ' B'
}

function formatPct(val: number | undefined | null): string {
  if (val == null || isNaN(Number(val))) return '0.0%'
  return Number(val).toFixed(1) + '%'
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    const ta = document.createElement('textarea')
    ta.value = text; ta.style.position = 'fixed'; ta.style.opacity = '0'
    document.body.appendChild(ta); ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
  }
}

// ─── Load inspect data ───
async function loadInspect() {
  loading.value = true
  error.value = null
  try {
    inspectData.value = await inspectContainer(containerId.value)
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

// ─── Action handlers ───
async function handleAction(action: string) {
  actionLoading.value = action
  try {
    switch (action) {
      case 'start': await startContainer(containerId.value); break
      case 'stop': await stopContainer(containerId.value); break
      case 'restart': await restartContainer(containerId.value); break
      case 'pause': await pauseContainer(containerId.value); break
      case 'unpause': await unpauseContainer(containerId.value); break
      case 'remove':
        await removeContainer(containerId.value, true)
        router.push('/containers')
        return
    }
    await loadInspect()
  } catch (e: any) {
    error.value = e.toString()
  }
  actionLoading.value = null
}

// ─── Logs ───
async function fetchLogs() {
  if (!containerId.value) return
  logsLoading.value = true
  try {
    const result = await getContainerLogs(containerId.value, logsTail.value, logsTimestamps.value)
    let filtered = result
    if (!showStdout.value) filtered = filtered.split('\n').filter(l => l.includes('[error]') || l.includes('[warn]')).join('\n')
    if (!showStderr.value) filtered = filtered.split('\n').filter(l => !l.includes('[error]')).join('\n')
    if (logsFilter.value) {
      try {
        const re = new RegExp(logsFilter.value, 'gi')
        filtered = filtered.split('\n').filter(l => re.test(l)).join('\n')
      } catch { /* ignore invalid regex */ }
    }
    logs.value = filtered || 'No logs found'
  } catch (e: any) {
    logs.value = `Error: ${e.toString()}`
  }
  logsLoading.value = false
  await nextTick()
  if (logAutoScroll.value && logContainerRef.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
  }
}

const coloredLogs = computed(() => {
  if (!logs.value) return '<span style="color:var(--text-disabled)">No logs available</span>'
  if (logs.value.startsWith('Error:')) return `<span style="color:var(--accent-red)">${logs.value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')}</span>`
  let html = logs.value
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/(\[error\])/gi, '<span style="color:var(--accent-red);font-weight:600">$1</span>')
    .replace(/(\[warn\])/gi, '<span style="color:var(--accent-yellow);font-weight:600">$1</span>')
    .replace(/(\[info\])/gi, '<span style="color:var(--accent-blue);font-weight:600">$1</span>')
    .replace(/(\[debug\])/gi, '<span style="color:var(--text-disabled);font-weight:600">$1</span>')
  // Colour stderr lines in red/amber
  html = html.split('\n').map(line => {
    // Docker log format: timestamp + line. stderr often has no marker so we check for common patterns
    if (/^[0-9]{4}-[0-9]{2}-[0-9]{2}T/.test(line) || /^[0-9]{4}\/[0-9]{2}\/[0-9]{2}/.test(line)) {
      // Has timestamp — check for error indicators
      if (/error|fail|exception|traceback|stderr/i.test(line) && !/\[info\]|\[debug\]/i.test(line)) {
        return `<span style="color:var(--accent-red)">${line}</span>`
      }
      return line
    }
    return line
  }).join('\n')
  return html
})

function scrollLogsToBottom() {
  logAutoScroll.value = true
  if (logContainerRef.value) {
    logContainerRef.value.scrollTop = logContainerRef.value.scrollHeight
  }
}

function handleLogScroll() {
  if (!logContainerRef.value) return
  const el = logContainerRef.value
  const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50
  logAutoScroll.value = atBottom
}

function copyLogs() {
  navigator.clipboard.writeText(logs.value)
}

// ─── Stats ───
async function fetchStats() {
  if (!containerId.value || !isRunning.value) return
  statsLoading.value = true
  try {
    statsData.value = await getContainerStats(containerId.value)
  } catch {
    // Silently fail on stats polling
  }
  statsLoading.value = false
}

function startStatsPolling() {
  stopStatsPolling()
  if (!isRunning.value) return
  fetchStats()
  statsInterval = window.setInterval(fetchStats, 3000)
}

function stopStatsPolling() {
  if (statsInterval != null) {
    clearInterval(statsInterval)
    statsInterval = undefined
  }
}

// ─── Tab change watcher ───
watch(activeTab, (tab) => {
  if (tab === 'logs') {
    fetchLogs()
  } else if (tab === 'stats') {
    startStatsPolling()
  } else {
    stopStatsPolling()
  }
})

// ─── Lifecycle ───
onMounted(async () => {
  await loadInspect()
  if (activeTab.value === 'logs') {
    await fetchLogs()
  } else if (activeTab.value === 'stats' && isRunning.value) {
    startStatsPolling()
  }
})

onUnmounted(() => {
  stopStatsPolling()
})
</script>

<template>
  <div class="view-root">
  <!-- Breadcrumb -->
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span @click="router.push('/')" style="cursor:pointer">Home</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span @click="router.push('/containers')" style="cursor:pointer">Containers</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span class="current">{{ loading ? '...' : containerName }}</span>
  </div>

  <!-- Loading state (A.6.1) -->
  <SkeletonLoader v-if="loading" variant="header" />
  <SkeletonLoader v-if="loading" variant="table-row" :rows="6" />

  <!-- Error state (A.6.3) -->
  <ErrorState
    v-if="error && !loading"
    :message="'Failed to inspect container'"
    :suggestion="'The container may have been removed or Docker may not be running.'"
    :detail="error"
    @retry="loadInspect"
  />

  <template v-if="inspectData && !loading && !error">
    <!-- Detail Header -->
    <div class="detail-header">
      <div class="detail-id">
        <span :class="['status-dot', isRunning ? 'status-dot--running' : isPaused ? 'status-dot--paused' : 'status-dot--stopped']"></span>
        <div>
          <div class="detail-name">{{ containerName }}</div>
          <div class="detail-image">
            {{ inspectData.Config?.Image || inspectData.Image || 'unknown' }}
            · ID: {{ (inspectData.Id || '').slice(0, 12) }}
          </div>
        </div>
      </div>
      <div class="detail-actions">
        <button
          v-if="isStopped || isStarting"
          class="btn btn-primary"
          :disabled="actionLoading !== null"
          @click="handleAction('start')"
        >
          <i class="fa-solid fa-play" :class="{ 'fa-spin': actionLoading === 'start' }"></i>
          {{ actionLoading === 'start' ? 'Starting...' : 'Start' }}
        </button>
        <button
          v-if="isRunning"
          class="btn btn-secondary"
          :disabled="actionLoading !== null"
          @click="handleAction('stop')"
        >
          <i class="fa-solid fa-stop"></i>
          {{ actionLoading === 'stop' ? 'Stopping...' : 'Stop' }}
        </button>
        <button
          v-if="isRunning && !isPaused"
          class="btn btn-secondary"
          :disabled="actionLoading !== null"
          @click="handleAction('pause')"
        >
          <i class="fa-solid fa-pause"></i>
          {{ actionLoading === 'pause' ? 'Pausing...' : 'Pause' }}
        </button>
        <button
          v-if="isPaused"
          class="btn btn-secondary"
          :disabled="actionLoading !== null"
          @click="handleAction('unpause')"
        >
          <i class="fa-solid fa-play"></i>
          {{ actionLoading === 'unpause' ? 'Resuming...' : 'Unpause' }}
        </button>
        <button
          v-if="isRunning"
          class="btn btn-secondary"
          :disabled="actionLoading !== null"
          @click="handleAction('restart')"
        >
          <i class="fa-solid fa-rotate-right" :class="{ 'fa-spin': actionLoading === 'restart' }"></i>
          {{ actionLoading === 'restart' ? 'Restarting...' : 'Restart' }}
        </button>
        <button
          v-if="!isRunning"
          class="btn btn-danger"
          :disabled="actionLoading !== null"
          @click="handleAction('remove')"
        >
          <i class="fa-solid fa-trash-can"></i>
          {{ actionLoading === 'remove' ? 'Removing...' : 'Remove' }}
        </button>
      </div>
    </div>

    <!-- Tab Bar -->
    <div class="tabs">
      <button :class="['tab', { active: activeTab === 'info' }]" @click="activeTab = 'info'">
        <i class="fa-solid fa-circle-info"></i> Info
      </button>
      <button :class="['tab', { active: activeTab === 'env' }]" @click="activeTab = 'env'">
        <i class="fa-solid fa-list"></i> Env Vars
      </button>
      <button :class="['tab', { active: activeTab === 'volumes' }]" @click="activeTab = 'volumes'">
        <i class="fa-solid fa-database"></i> Volumes
      </button>
      <button :class="['tab', { active: activeTab === 'network' }]" @click="activeTab = 'network'">
        <i class="fa-solid fa-network-wired"></i> Network
      </button>
      <button :class="['tab', { active: activeTab === 'health' }]" @click="activeTab = 'health'">
        <i class="fa-solid fa-heart-pulse"></i> Health
      </button>
      <button :class="['tab', { active: activeTab === 'labels' }]" @click="activeTab = 'labels'">
        <i class="fa-solid fa-tags"></i> Labels
      </button>
      <button :class="['tab', { active: activeTab === 'logs' }]" @click="activeTab = 'logs'">
        <i class="fa-solid fa-scroll"></i> Logs
      </button>
      <button :class="['tab', { active: activeTab === 'stats' }]" @click="activeTab = 'stats'">
        <i class="fa-solid fa-chart-line"></i> Stats
      </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content" :key="containerId">
      <!-- ════════ Info Tab ════════ -->
      <template v-if="activeTab === 'info'">
        <div class="inspect-grid">
          <div class="inspect-section">
            <div class="inspect-section-title"><i class="fa-solid fa-box"></i> General</div>
            <div class="inspect-row">
              <span class="inspect-label">Container ID</span>
              <span class="inspect-value" style="display:flex;align-items:center;gap:6px">
                <span style="font-size:10px">{{ (inspectData.Id || '') }}</span>
                <button class="btn btn-ghost btn-sm" @click="copyToClipboard(inspectData.Id || '')" title="Copy ID">
                  <i class="fa-solid fa-copy"></i>
                </button>
              </span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Image</span>
              <span class="inspect-value">{{ inspectData.Config?.Image || inspectData.Image || '--' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Created</span>
              <span class="inspect-value">{{ formatDateTime(inspectData.Created) }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">State</span>
              <span class="inspect-value" :class="{
                green: containerState === 'running',
                red: containerState === 'exited' || containerState === 'dead',
              }">
                <span :class="['status-dot', isRunning ? 'status-dot--running' : isPaused ? 'status-dot--paused' : 'status-dot--stopped']" style="margin-right:6px;vertical-align:middle"></span>
                {{ containerState }}
              </span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Status</span>
              <span class="inspect-value">{{ inspectData.State?.Status || '--' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Exit Code</span>
              <span class="inspect-value">{{ inspectData.State?.ExitCode ?? '--' }}</span>
            </div>
          </div>

          <div class="inspect-section">
            <div class="inspect-section-title"><i class="fa-solid fa-gear"></i> Configuration</div>
            <div class="inspect-row">
              <span class="inspect-label">Command</span>
              <span class="inspect-value">{{ Array.isArray(inspectData.Config?.Cmd) ? inspectData.Config.Cmd.join(' ') : inspectData.Config?.Cmd || '--' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Entrypoint</span>
              <span class="inspect-value">{{ Array.isArray(inspectData.Config?.Entrypoint) ? inspectData.Config.Entrypoint.join(' ') : inspectData.Config?.Entrypoint || '(none)' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Platform</span>
              <span class="inspect-value">{{ inspectData.Platform || 'linux' }}/{{ inspectData.Architecture || 'amd64' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Driver</span>
              <span class="inspect-value">{{ inspectData.Driver || '--' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Restart Policy</span>
              <span class="inspect-value green">{{ inspectData.HostConfig?.RestartPolicy?.Name || 'none' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Restart Count</span>
              <span class="inspect-value">{{ inspectData.RestartCount ?? 0 }}</span>
            </div>
          </div>

          <div class="inspect-section">
            <div class="inspect-section-title"><i class="fa-solid fa-plug"></i> Port Bindings</div>
            <template v-if="portsArray.length > 0">
              <div v-for="p in portsArray" :key="p.containerPort + '/' + p.protocol + p.hostPort" class="inspect-row">
                <span class="inspect-label">{{ p.containerPort }}/{{ p.protocol }}</span>
                <span class="inspect-value">{{ p.hostIp }}:{{ p.hostPort || '(not published)' }}</span>
              </div>
            </template>
            <EmptyState v-else icon="fa-solid fa-plug" title="No port bindings" description="This container has no published ports." />
          </div>

          <div class="inspect-section">
            <div class="inspect-section-title"><i class="fa-solid fa-clock"></i> Timing</div>
            <div class="inspect-row">
              <span class="inspect-label">Started At</span>
              <span class="inspect-value">{{ formatDateTime(inspectData.State?.StartedAt) }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Finished At</span>
              <span class="inspect-value">{{ formatDateTime(inspectData.State?.FinishedAt) }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Path</span>
              <span class="inspect-value" style="font-size:10px">{{ inspectData.LogPath || '--' }}</span>
            </div>
          </div>
        </div>
      </template>

      <!-- ════════ Env Vars Tab ════════ -->
      <template v-if="activeTab === 'env'">
        <div v-if="envVars.length > 0" class="section">
          <div class="section-header">
            <span class="section-title">Environment Variables ({{ envVars.length }})</span>
            <div class="table-filter">
              <i class="fa-solid fa-search"></i>
              <input v-model="envFilter" placeholder="Filter variables..." />
            </div>
          </div>
          <div style="max-height:420px;overflow-y:auto">
            <div class="col-headers">
              <span class="col-header" style="flex:0 0 220px">KEY</span>
              <span class="col-header" style="flex:1">VALUE</span>
              <span style="width:40px"></span>
            </div>
            <div v-for="(env, idx) in filteredEnvVars" :key="idx" class="data-row" style="cursor:default;padding:8px 20px">
              <span style="flex:0 0 220px;font-family:var(--font-mono);font-size:11px;color:var(--accent-purple);word-break:break-all">{{ env.key }}</span>
              <span style="flex:1;font-family:var(--font-mono);font-size:11px;color:var(--text-muted);word-break:break-all">{{ env.value }}</span>
              <button class="action-btn" @click="copyToClipboard(env.key + '=' + env.value)" title="Copy">
                <i class="fa-solid fa-copy"></i>
              </button>
            </div>
          </div>
        </div>
        <EmptyState v-else icon="fa-solid fa-list" title="No environment variables" description="This container has no custom environment variables set." />
      </template>

      <!-- ════════ Volumes Tab ════════ -->
      <template v-if="activeTab === 'volumes'">
        <div v-if="mounts.length > 0" class="section">
          <div class="section-header">
            <span class="section-title">Mounts ({{ mounts.length }})</span>
          </div>
          <div class="col-headers">
            <span class="col-header" style="flex:0 0 70px">TYPE</span>
            <span class="col-header" style="flex:1">SOURCE</span>
            <span class="col-header" style="flex:1">DESTINATION</span>
            <span class="col-header" style="flex:0 0 60px">MODE</span>
            <span class="col-header" style="flex:0 0 40px">RW</span>
          </div>
          <div v-for="(m, idx) in mounts" :key="idx" class="data-row" style="cursor:default">
            <span style="flex:0 0 70px"><span :class="['tag', m.Type === 'bind' ? 'tag running' : 'tag paused']">{{ m.Type }}</span></span>
            <span style="flex:1;font-family:var(--font-mono);font-size:11px;color:var(--text-muted);word-break:break-all">{{ m.Source }}</span>
            <span style="flex:1;font-family:var(--font-mono);font-size:11px;word-break:break-all">{{ m.Destination }}</span>
            <span style="flex:0 0 60px;font-family:var(--font-mono);font-size:11px;color:var(--text-muted)">{{ m.Mode || 'default' }}</span>
            <span style="flex:0 0 40px;font-family:var(--font-mono);font-size:11px">
              <span v-if="m.RW" style="color:var(--accent-green)">✓</span>
              <span v-else style="color:var(--accent-red)">✗</span>
            </span>
          </div>
        </div>
        <EmptyState v-else icon="fa-solid fa-database" title="No volume mounts" description="This container has no volume mounts configured." />
      </template>

      <!-- ════════ Network Tab ════════ -->
      <template v-if="activeTab === 'network'">
        <div v-if="Object.keys(networks).length > 0" class="inspect-grid">
          <div v-for="(net, netName) in networks" :key="netName" class="inspect-section">
            <div class="inspect-section-title" style="color:var(--accent-cyan)!important">
              <i class="fa-solid fa-network-wired"></i> {{ netName }}
            </div>
            <div class="inspect-row">
              <span class="inspect-label">IP Address</span>
              <span class="inspect-value" data-testid="network-ip">{{ net.IPAddress || '--' }}{{ net.IPPrefixLen ? '/' + net.IPPrefixLen : '' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Gateway</span>
              <span class="inspect-value" data-testid="network-gateway">{{ net.Gateway || '--' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">MAC Address</span>
              <span class="inspect-value">{{ net.MacAddress || '--' }}</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Network ID</span>
              <span class="inspect-value" style="font-size:9px">{{ (net.NetworkID || '').slice(0, 24) }}...</span>
            </div>
            <div class="inspect-row">
              <span class="inspect-label">Endpoint ID</span>
              <span class="inspect-value" style="font-size:9px">{{ (net.EndpointID || '').slice(0, 24) }}...</span>
            </div>
          </div>
          <div class="inspect-section">
            <div class="inspect-section-title"><i class="fa-solid fa-plug"></i> Published Ports</div>
            <template v-if="portsArray.length > 0">
              <div v-for="p in portsArray" :key="p.containerPort + '/' + p.protocol + p.hostPort" class="inspect-row">
                <span class="inspect-label">{{ p.containerPort }}/{{ p.protocol }}</span>
                <span class="inspect-value">{{ p.hostIp && p.hostIp !== '0.0.0.0' ? p.hostIp + ':' : '' }}{{ p.hostPort || 'not published' }}</span>
              </div>
            </template>
            <EmptyState v-else icon="fa-solid fa-plug" title="No published ports" description="No ports are published for this container." />
          </div>
        </div>
        <EmptyState v-else icon="fa-solid fa-network-wired" title="No networks" description="This container is not connected to any networks." />
      </template>

      <!-- ════════ Health Tab ════════ -->
      <template v-if="activeTab === 'health'">
        <template v-if="healthData">
          <div style="display:flex;align-items:center;gap:12px;margin-bottom:20px;padding:16px;background:var(--bg-tertiary);border-radius:var(--radius-md);border:1px solid var(--border-light)">
            <div :style="{
              width: '40px', height: '40px', borderRadius: '12px',
              background: healthStatus === 'healthy' ? 'rgba(16,185,129,0.1)' : healthStatus === 'unhealthy' ? 'rgba(239,68,68,0.1)' : 'rgba(245,158,11,0.1)',
              display: 'flex', alignItems: 'center', justifyContent: 'center',
              color: healthStatus === 'healthy' ? 'var(--accent-green)' : healthStatus === 'unhealthy' ? 'var(--accent-red)' : 'var(--accent-yellow)',
              fontSize: '18px'
            }">
              <i :class="healthStatus === 'healthy' ? 'fa-solid fa-circle-check' : healthStatus === 'unhealthy' ? 'fa-solid fa-circle-exclamation' : 'fa-solid fa-hourglass-half'"></i>
            </div>
            <div>
              <div style="font-size:16px;font-weight:600;text-transform:capitalize">{{ healthStatus }}</div>
              <div style="font-size:12px;color:var(--text-muted)">
                Failing streak: {{ healthData.FailingStreak ?? 0 }}
              </div>
            </div>
          </div>

          <div v-if="healthLogs.length > 0">
            <div class="section" v-for="(log, idx) in healthLogs.slice(-5).reverse()" :key="idx" style="margin-bottom:8px">
              <div class="section-header" style="padding:10px 16px">
                <div style="display:flex;align-items:center;gap:8px">
                  <span :style="{ color: log.ExitCode === 0 ? 'var(--accent-green)' : 'var(--accent-red)' }">
                    <i :class="log.ExitCode === 0 ? 'fa-solid fa-circle-check' : 'fa-solid fa-circle-xmark'"></i>
                  </span>
                  <span class="section-title" style="font-size:12px">Exit: {{ log.ExitCode }}</span>
                  <span style="font-size:11px;color:var(--text-muted)">{{ formatDateTime(log.Start) }}</span>
                </div>
              </div>
              <pre style="padding:12px 16px;font-family:var(--font-mono);font-size:11px;color:var(--text-muted);white-space:pre-wrap;word-break:break-all;max-height:120px;overflow-y:auto;background:var(--bg-tertiary);margin:0"><code>{{ log.Output || '(no output)' }}</code></pre>
            </div>
          </div>
        </template>
        <EmptyState v-else icon="fa-solid fa-heart-pulse" title="No health check configured" description="This container does not have a health check. Add HEALTHCHECK to the Dockerfile or use `--health-cmd` when running." />
      </template>

      <!-- ════════ Labels Tab ════════ -->
      <template v-if="activeTab === 'labels'">
        <div v-if="labelsArray.length > 0" class="section">
          <div class="section-header">
            <span class="section-title">Labels ({{ labelsArray.length }})</span>
          </div>
          <div style="max-height:420px;overflow-y:auto">
            <div class="col-headers">
              <span class="col-header" style="flex:0 0 220px">KEY</span>
              <span class="col-header" style="flex:1">VALUE</span>
              <span style="width:40px"></span>
            </div>
            <div v-for="(label, idx) in labelsArray" :key="idx" class="data-row" style="cursor:default;padding:8px 20px">
              <span style="flex:0 0 220px;font-family:var(--font-mono);font-size:11px;color:var(--accent-cyan);word-break:break-all" data-testid="label-key">{{ label.key }}</span>
              <span style="flex:1;font-family:var(--font-mono);font-size:11px;color:var(--text-muted);word-break:break-all" data-testid="label-value">{{ label.value }}</span>
              <button class="action-btn" @click="copyToClipboard(label.key + '=' + label.value)" title="Copy">
                <i class="fa-solid fa-copy"></i>
              </button>
            </div>
          </div>
        </div>
        <EmptyState v-else icon="fa-solid fa-tags" title="No labels" description="This container has no labels set." />
      </template>

      <!-- ════════ Logs Tab ════════ -->
      <template v-if="activeTab === 'logs'">
        <div class="log-viewer">
          <div class="log-toolbar">
            <div class="log-filters">
              <label class="filter-toggle"><input type="checkbox" v-model="showStdout" @change="fetchLogs"> STDOUT</label>
              <label class="filter-toggle"><input type="checkbox" v-model="showStderr" @change="fetchLogs"> STDERR</label>
              <label class="filter-toggle"><input type="checkbox" v-model="logsTimestamps" @change="fetchLogs"> TS</label>
            </div>
            <div class="log-controls">
              <input v-model="logsFilter" placeholder="Search..." @keyup.enter="fetchLogs" class="log-search" />
              <select v-model="logsTail" @change="fetchLogs" class="log-select">
                <option :value="100">100</option>
                <option :value="500">500</option>
                <option :value="1000">1000</option>
                <option :value="5000">5000</option>
              </select>
              <button class="btn btn-ghost btn-sm" @click="fetchLogs" :disabled="logsLoading">
                <i class="fa-solid fa-rotate" :class="{ 'fa-spin': logsLoading }"></i> Refresh
              </button>
              <button class="btn btn-ghost btn-sm" @click="copyLogs">
                <i class="fa-solid fa-copy"></i>
              </button>
            </div>
          </div>
          <div
            ref="logContainerRef"
            class="log-content"
            @scroll="handleLogScroll"
            v-html="coloredLogs"
          ></div>
          <div v-if="!logAutoScroll" class="log-scroll-hint" @click="scrollLogsToBottom">
            <i class="fa-solid fa-arrow-down"></i> New logs — click to scroll to bottom
          </div>
        </div>
      </template>

      <!-- ════════ Stats Tab ════════ -->
      <template v-if="activeTab === 'stats'">
        <template v-if="isRunning">
          <div data-testid="stats-polling-indicator" style="margin-bottom:12px;display:flex;align-items:center;gap:8px">
            <span style="font-size:12px;color:var(--text-muted)">
              <i class="fa-solid fa-circle" style="color:var(--accent-green);font-size:8px;margin-right:4px"></i>
              Live (polling every 3s)
            </span>
            <button class="btn btn-ghost btn-sm" @click="fetchStats" :disabled="statsLoading">
              <i class="fa-solid fa-rotate" :class="{ 'fa-spin': statsLoading }"></i> Refresh
            </button>
          </div>
          <div class="metrics-grid">
            <div class="metric-card">
              <div class="metric-icon cyan"><i class="fa-solid fa-microchip"></i></div>
              <div class="metric-label">CPU</div>
              <div class="metric-value" style="font-size:1.5rem">{{ formatPct(statsData?.cpu_percentage) }}</div>
              <div class="host-widget-bar" style="margin-top:8px">
                <div class="host-widget-bar-fill" :style="{
                  width: Math.min((statsData?.cpu_percentage ?? 0), 100) + '%',
                  background: (statsData?.cpu_percentage ?? 0) > 80 ? 'var(--accent-red)' : (statsData?.cpu_percentage ?? 0) > 50 ? 'var(--accent-yellow)' : 'var(--accent-green)'
                }"></div>
              </div>
            </div>
            <div class="metric-card">
              <div class="metric-icon purple"><i class="fa-solid fa-memory"></i></div>
              <div class="metric-label">Memory</div>
              <div class="metric-value" style="font-size:1.5rem">{{ formatPct(statsData?.memory_percentage) }}</div>
              <div style="display:flex;justify-content:space-between;margin-top:4px">
                <span style="font-size:11px;color:var(--text-muted)">{{ formatBytes(statsData?.memory_usage_bytes) }}</span>
                <span style="font-size:11px;color:var(--text-disabled)">{{ formatBytes(statsData?.memory_limit_bytes) }}</span>
              </div>
              <div class="host-widget-bar" style="margin-top:4px">
                <div class="host-widget-bar-fill" :style="{
                  width: Math.min((statsData?.memory_percentage ?? 0), 100) + '%',
                  background: (statsData?.memory_percentage ?? 0) > 80 ? 'var(--accent-red)' : (statsData?.memory_percentage ?? 0) > 50 ? 'var(--accent-yellow)' : 'var(--accent-green)'
                }"></div>
              </div>
            </div>
            <div class="metric-card">
              <div class="metric-icon green"><i class="fa-solid fa-wifi"></i></div>
              <div class="metric-label">Network</div>
              <div class="metric-value" style="font-size:1.2rem;font-family:var(--font-mono)">RX</div>
              <div style="font-size:13px;font-weight:600;font-family:var(--font-mono)">{{ formatBytes(statsData?.network_rx_bytes) }}</div>
              <div style="font-size:12px;color:var(--text-muted);margin-top:2px">TX: {{ formatBytes(statsData?.network_tx_bytes) }}</div>
            </div>
            <div class="metric-card">
              <div class="metric-icon amber"><i class="fa-solid fa-hard-drive"></i></div>
              <div class="metric-label">Block I/O</div>
              <div class="metric-value" style="font-size:1.2rem;font-family:var(--font-mono)">Read</div>
              <div style="font-size:13px;font-weight:600;font-family:var(--font-mono)">{{ formatBytes(statsData?.block_read_bytes) }}</div>
              <div style="font-size:12px;color:var(--text-muted);margin-top:2px">Write: {{ formatBytes(statsData?.block_write_bytes) }}</div>
            </div>
          </div>
          <div style="margin-top:16px;display:flex;gap:16px;align-items:center;padding:12px 16px;background:var(--bg-tertiary);border-radius:var(--radius-md);border:1px solid var(--border-light)">
            <div style="display:flex;align-items:center;gap:8px">
              <i class="fa-solid fa-diagram-project" style="color:var(--accent-cyan)"></i>
              <span style="font-size:12px;color:var(--text-muted)">PIDs:</span>
              <span style="font-size:16px;font-weight:700;font-family:var(--font-mono)">{{ statsData?.pids ?? '--' }}</span>
            </div>
            <div style="display:flex;align-items:center;gap:8px">
              <i class="fa-solid fa-clock" style="color:var(--text-disabled)"></i>
              <span style="font-size:12px;color:var(--text-muted)">Last updated:</span>
              <span style="font-size:12px;font-family:var(--font-mono);color:var(--text-muted)">
                {{ statsData?.timestamp ? new Date(statsData.timestamp * 1000).toLocaleTimeString() : '--' }}
              </span>
            </div>
          </div>
        </template>
        <EmptyState v-else icon="fa-solid fa-chart-line" title="Container not running" description="Start the container to view live resource usage statistics." />
      </template>
    </div>
  </template>
  </div>
</template>

<style scoped>
/* ─── Detail Header ─── */
.detail-header {
  display: flex; align-items: center; justify-content: space-between;
  background: var(--bg-secondary); border: 1px solid var(--border-color);
  border-radius: var(--radius-lg); padding: 16px 20px;
}
.detail-id { display: flex; align-items: center; gap: 12px; }
.detail-name { font-size: 1.25rem; font-weight: 700; letter-spacing: -0.01em; }
.detail-image { font-size: 12px; color: var(--text-muted); font-family: var(--font-mono); margin-top: 2px; }
.detail-actions { display: flex; gap: 6px; }

/* ─── Tabs ─── */
.tabs {
  display: flex; gap: 0; border-bottom: 1px solid var(--border-color);
  background: var(--bg-secondary);
  border-radius: var(--radius-lg) var(--radius-lg) 0 0; overflow: hidden;
}
.tab {
  padding: 12px 16px; font-size: 12px; font-weight: 500; color: var(--text-muted);
  cursor: pointer; border: none; background: none; font-family: var(--font-sans);
  border-bottom: 2px solid transparent; transition: all var(--transition-fast);
  display: flex; align-items: center; gap: 6px; white-space: nowrap;
}
.tab:hover { color: var(--text-main); background: var(--bg-hover); }
.tab.active { color: var(--accent-cyan); border-bottom-color: var(--accent-cyan); background: var(--bg-tertiary); }
.tab i { font-size: 12px; }

/* ─── Tab Content ─── */
.tab-content {
  background: var(--bg-secondary); border: 1px solid var(--border-color);
  border-top: none; border-radius: 0 0 var(--radius-lg) var(--radius-lg);
  padding: 20px; min-height: 350px;
}

/* ─── Inspect Grid ─── */
.inspect-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.inspect-section {
  background: var(--bg-tertiary); border-radius: var(--radius-md);
  padding: 16px; border: 1px solid var(--border-light);
}
.inspect-section-title {
  font-size: 11px; font-weight: 600; color: var(--text-disabled);
  text-transform: uppercase; letter-spacing: .04em; margin-bottom: 12px;
}
.inspect-section-title i { margin-right: 6px; }
.inspect-row {
  display: flex; justify-content: space-between;
  padding: 6px 0; font-size: 12px; border-bottom: 1px solid var(--border-light);
}
.inspect-row:last-child { border-bottom: none; }
.inspect-label { color: var(--text-muted); }
.inspect-value { font-weight: 500; font-family: var(--font-mono); font-size: 11px; text-align: right; max-width: 60%; word-break: break-all; }
.inspect-value.green { color: var(--accent-green); }
.inspect-value.red { color: var(--accent-red); }

/* ─── Log Viewer ─── */
.log-viewer { display: flex; flex-direction: column; }
.log-toolbar {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--border-light);
  gap: 8px; flex-wrap: wrap;
}
.log-filters { display: flex; gap: 12px; }
.filter-toggle {
  display: flex; align-items: center; gap: 6px;
  font-size: 11px; color: var(--text-muted); cursor: pointer;
}
.filter-toggle input[type="checkbox"] { accent-color: var(--accent-cyan); }
.log-controls { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
.log-select {
  background: var(--bg-tertiary); border: 1px solid var(--border-color);
  border-radius: 4px; color: var(--text-main); font-size: 11px;
  padding: 4px 8px; font-family: var(--font-sans);
}
.log-search {
  width: 120px; background: var(--bg-tertiary);
  border: 1px solid var(--border-color); border-radius: 4px;
  padding: 4px 8px; font-size: 11px; color: var(--text-main);
  font-family: var(--font-sans); outline: none;
}
.log-search:focus { border-color: var(--accent-cyan); }
.log-content {
  background: #000; border-radius: var(--radius-md); padding: 14px;
  font-family: var(--font-mono); font-size: 11px; line-height: 1.6;
  max-height: 380px; overflow-y: auto; white-space: pre-wrap; word-break: break-all;
  color: #e0e0e0;
}
.log-scroll-hint {
  text-align: center; padding: 8px; font-size: 11px; color: var(--accent-cyan);
  cursor: pointer; background: rgba(0, 229, 255, 0.05);
  border-radius: 0 0 var(--radius-md) var(--radius-md);
}
.log-scroll-hint:hover { background: rgba(0, 229, 255, 0.1); }

/* ─── Env Filter ─── */
.env-filter {
  display: flex; align-items: center; background: var(--bg-tertiary);
  border: 1px solid var(--border-color); border-radius: 6px;
  padding: 6px 10px; font-size: 12px; color: var(--text-muted); gap: 6px;
}
.env-filter input {
  background: none; border: none; outline: none;
  color: var(--text-main); font-size: 12px; font-family: var(--font-sans); width: 180px;
}
.env-filter input::placeholder { color: var(--text-disabled); }

/* ─── Empty State override inside tab ─── */
.tab-content :deep(.empty-state) {
  padding: 40px 20px;
}
.tab-content :deep(.empty-state-icon) {
  width: 60px; height: 60px; font-size: 24px;
}
</style>
