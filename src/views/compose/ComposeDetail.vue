<!-- ItzamBox — Docker Compose Project Detail View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  parseComposeFile, composeUp, composeDown, composeRestart, composeLogs, composePs,
  type ComposeFileInfo, type ComposeServiceStatus,
} from '../../composables/useDocker'
import SkeletonLoader from '../../components/shared/SkeletonLoader.vue'
import EmptyState from '../../components/shared/EmptyState.vue'
import ErrorState from '../../components/shared/ErrorState.vue'

const route = useRoute()
const router = useRouter()
const projectName = computed(() => route.params.name as string)
const projectPath = computed(() => (route.query.path as string) || '')

// ─── State ───
const fileInfo = ref<ComposeFileInfo | null>(null)
const services = ref<ComposeServiceStatus[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const activeTab = ref<'services' | 'volumes' | 'networks' | 'logs'>('services')
const actionLoading = ref<string | null>(null)

// ─── Logs state ───
const logs = ref('')
const logsLoading = ref(false)
const logsTail = ref(500)
const logContainerRef = ref<HTMLDivElement>()
const logAutoScroll = ref(true)

// ─── Lifecycle ───
onMounted(async () => {
  await loadData()
})

async function loadData() {
  if (!projectPath.value) {
    error.value = 'No project path specified.'
    loading.value = false
    return
  }
  loading.value = true
  error.value = null
  try {
    const [info, ps] = await Promise.all([
      parseComposeFile(projectPath.value),
      composePs(projectPath.value),
    ])
    fileInfo.value = info
    services.value = ps
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

// ─── Derived ───
const runningServices = computed(() => services.value.filter(s => s.state === 'running'))
const totalServices = computed(() => fileInfo.value?.services.length ?? 0)

// ─── Actions ───
async function handleAction(action: 'up' | 'down' | 'restart') {
  actionLoading.value = action
  try {
    if (action === 'up') {
      await composeUp(projectPath.value, true)
    } else if (action === 'down') {
      await composeDown(projectPath.value, false, false)
    } else if (action === 'restart') {
      await composeRestart(projectPath.value)
    }
    // Refresh status after action
    services.value = await composePs(projectPath.value)
  } catch (e: any) {
    error.value = e.toString()
  }
  actionLoading.value = null
}

async function handleServiceAction(serviceName: string, action: 'restart' | 'logs') {
  actionLoading.value = `svc-${serviceName}`
  try {
    if (action === 'restart') {
      await composeRestart(projectPath.value, [serviceName])
      services.value = await composePs(projectPath.value)
    } else if (action === 'logs') {
      logsTail.value = 100
      await fetchServiceLogs(serviceName)
      activeTab.value = 'logs'
    }
  } catch (e: any) {
    error.value = e.toString()
  }
  actionLoading.value = null
}

// ─── Logs ───
async function fetchServiceLogs(serviceName?: string) {
  logsLoading.value = true
  try {
    const svcs = serviceName ? [serviceName] : undefined
    const result = await composeLogs(projectPath.value, logsTail.value, svcs)
    logs.value = result || '(no logs)'
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
  if (logs.value.startsWith('Error:')) {
    return `<span style="color:var(--accent-red)">${logs.value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')}</span>`
  }
  let html = logs.value
    .replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
    .replace(/(\[error\]|error|ERROR)/gi, '<span style="color:var(--accent-red);font-weight:600">$1</span>')
    .replace(/(\[warn\]|warn|WARN)/gi, '<span style="color:var(--accent-yellow);font-weight:600">$1</span>')
    .replace(/(\[info\]|info|INFO)/gi, '<span style="color:var(--accent-blue);font-weight:600">$1</span>')
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

// ─── Service state helpers ───
function getStateClass(state: string): string {
  switch (state) {
    case 'running': return 'status-dot--running'
    case 'paused': return 'status-dot--paused'
    case 'exited':
    case 'stopped': return 'status-dot--stopped'
    default: return 'status-dot--stopped'
  }
}

function getServiceState(serviceName: string): string {
  const svc = services.value.find(s => s.name === serviceName)
  return svc?.state || 'unknown'
}

function getServiceStatus(serviceName: string): string {
  const svc = services.value.find(s => s.name === serviceName)
  return svc?.status || '—'
}

function getServicePorts(serviceName: string): string {
  const svc = services.value.find(s => s.name === serviceName)
  return svc?.ports || ''
}
</script>

<template>
  <!-- Breadcrumb -->
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span @click="router.push('/')" style="cursor:pointer">Home</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span @click="router.push('/compose')" style="cursor:pointer">Compose</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span class="current">{{ loading ? '...' : projectName }}</span>
  </div>

  <!-- Loading state -->
  <SkeletonLoader v-if="loading" variant="header" />
  <SkeletonLoader v-if="loading" variant="table-row" :rows="4" />

  <!-- Error state -->
  <ErrorState
    v-if="error && !loading"
    :message="'Failed to load compose project'"
    :suggestion="'The project directory may have changed or Docker may not be running.'"
    :detail="error"
    @retry="loadData"
  />

  <template v-if="fileInfo && !loading && !error">
    <!-- Detail Header -->
    <div class="detail-header">
      <div class="detail-id">
        <div style="
          width:40px;height:40px;border-radius:10px;
          background:rgba(168,85,247,0.1);color:var(--accent-purple);
          display:flex;align-items:center;justify-content:center;font-size:18px;
          flex-shrink:0
        ">
          <i class="fa-solid fa-layer-group"></i>
        </div>
        <div>
          <div class="detail-name">{{ projectName }}</div>
          <div class="detail-image" style="font-size:11px;font-family:var(--font-mono);color:var(--text-muted)">
            {{ projectPath }}
          </div>
        </div>
      </div>
      <div class="detail-actions">
        <button class="btn btn-primary" :disabled="actionLoading !== null" @click="handleAction('up')">
          <i class="fa-solid fa-play" :class="{ 'fa-spin': actionLoading === 'up' }"></i>
          {{ actionLoading === 'up' ? 'Starting...' : 'Up' }}
        </button>
        <button class="btn btn-secondary" :disabled="actionLoading !== null" @click="handleAction('down')">
          <i class="fa-solid fa-stop"></i>
          {{ actionLoading === 'down' ? 'Stopping...' : 'Down' }}
        </button>
        <button class="btn btn-secondary" :disabled="actionLoading !== null" @click="handleAction('restart')">
          <i class="fa-solid fa-rotate-right" :class="{ 'fa-spin': actionLoading === 'restart' }"></i>
          {{ actionLoading === 'restart' ? 'Restarting...' : 'Restart' }}
        </button>
        <button class="btn btn-secondary" @click="router.push({ name: 'ComposeEditor', params: { name: projectName }, query: { path: projectPath } })">
          <i class="fa-solid fa-pen-to-square"></i> Edit
        </button>
      </div>
    </div>

    <!-- Summary chips -->
    <div class="summary-bar">
      <div class="summary-chip all">Services <span class="summary-count">{{ totalServices }}</span></div>
      <div class="summary-chip running">
        <i class="fa-solid fa-circle" style="font-size:7px;color:var(--accent-green)"></i>
        Running <span class="summary-count">{{ runningServices.length }}</span>
      </div>
    </div>

    <!-- Tab Bar -->
    <div class="tabs">
      <button :class="['tab', { active: activeTab === 'services' }]" @click="activeTab = 'services'">
        <i class="fa-solid fa-cubes"></i> Services
      </button>
      <button :class="['tab', { active: activeTab === 'volumes' }]" @click="activeTab = 'volumes'">
        <i class="fa-solid fa-database"></i> Volumes ({{ fileInfo.volumes.length }})
      </button>
      <button :class="['tab', { active: activeTab === 'networks' }]" @click="activeTab = 'networks'">
        <i class="fa-solid fa-network-wired"></i> Networks ({{ fileInfo.networks.length }})
      </button>
      <button :class="['tab', { active: activeTab === 'logs' }]" @click="activeTab = 'logs'; fetchServiceLogs()">
        <i class="fa-solid fa-scroll"></i> Logs
      </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content">
      <!-- ════════ Services Tab ════════ -->
      <template v-if="activeTab === 'services'">
        <div v-if="fileInfo.services.length > 0" class="section">
          <div class="col-headers">
            <span class="col-header" style="flex:0 0 24px"></span>
            <span class="col-header" style="flex:0 0 160px">SERVICE</span>
            <span class="col-header" style="flex:1">IMAGE / BUILD</span>
            <span class="col-header" style="flex:0 0 90px">STATE</span>
            <span class="col-header" style="flex:0 0 100px">STATUS</span>
            <span class="col-header" style="flex:0 0 120px">PORTS</span>
            <span style="flex:0 0 80px;text-align:right">ACTIONS</span>
          </div>
          <div
            v-for="svc in fileInfo.services"
            :key="svc.name"
            class="data-row"
            style="cursor:default"
          >
            <span style="flex:0 0 24px">
              <span :class="['status-dot', getStateClass(getServiceState(svc.name))]"></span>
            </span>
            <div style="flex:0 0 160px;font-size:13px;font-weight:600">{{ svc.name }}</div>
            <div style="flex:1;min-width:0">
              <span v-if="svc.image" style="font-family:var(--font-mono);font-size:11px;color:var(--text-muted)">{{ svc.image }}</span>
              <span v-else-if="svc.build" style="font-family:var(--font-mono);font-size:11px;color:var(--accent-yellow)">
                <i class="fa-solid fa-hammer"></i> build: {{ svc.build }}
              </span>
              <span v-else style="color:var(--text-disabled);font-size:11px">—</span>
            </div>
            <div style="flex:0 0 90px">
              <span :class="['tag', getServiceState(svc.name) === 'running' ? 'running' : 'stopped']">
                {{ getServiceState(svc.name) }}
              </span>
            </div>
            <div style="flex:0 0 100px;font-size:11px;color:var(--text-muted);font-family:var(--font-mono);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
              {{ getServiceStatus(svc.name) }}
            </div>
            <div style="flex:0 0 120px;font-size:10px;color:var(--text-muted);font-family:var(--font-mono);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
              {{ getServicePorts(svc.name) || '—' }}
            </div>
            <div style="flex:0 0 80px;display:flex;gap:2px;justify-content:flex-end" @click.stop>
              <button
                class="action-btn"
                title="Restart service"
                :disabled="actionLoading === 'svc-' + svc.name"
                @click="handleServiceAction(svc.name, 'restart')"
              >
                <i class="fa-solid fa-rotate-right" :class="{ 'fa-spin': actionLoading === 'svc-' + svc.name }"></i>
              </button>
              <button
                class="action-btn"
                title="View logs"
                :disabled="actionLoading === 'svc-' + svc.name"
                @click="handleServiceAction(svc.name, 'logs')"
              >
                <i class="fa-solid fa-scroll"></i>
              </button>
            </div>
          </div>
        </div>
        <EmptyState v-else icon="fa-solid fa-cubes" title="No services defined" description="This compose file has no services." />
      </template>

      <!-- ════════ Volumes Tab ════════ -->
      <template v-if="activeTab === 'volumes'">
        <div v-if="fileInfo.volumes.length > 0" class="section">
          <div class="col-headers">
            <span class="col-header" style="flex:1">VOLUME NAME</span>
          </div>
          <div v-for="vol in fileInfo.volumes" :key="vol" class="data-row" style="cursor:default">
            <div style="flex:1;display:flex;align-items:center;gap:10px">
              <i class="fa-solid fa-database" style="color:var(--accent-cyan);font-size:12px"></i>
              <span style="font-family:var(--font-mono);font-size:12px">{{ vol }}</span>
            </div>
          </div>
        </div>
        <EmptyState v-else icon="fa-solid fa-database" title="No volumes" description="This compose file defines no named volumes." />
      </template>

      <!-- ════════ Networks Tab ════════ -->
      <template v-if="activeTab === 'networks'">
        <div v-if="fileInfo.networks.length > 0" class="section">
          <div class="col-headers">
            <span class="col-header" style="flex:1">NETWORK NAME</span>
          </div>
          <div v-for="net in fileInfo.networks" :key="net" class="data-row" style="cursor:default">
            <div style="flex:1;display:flex;align-items:center;gap:10px">
              <i class="fa-solid fa-network-wired" style="color:var(--accent-purple);font-size:12px"></i>
              <span style="font-family:var(--font-mono);font-size:12px">{{ net }}</span>
            </div>
          </div>
        </div>
        <EmptyState v-else icon="fa-solid fa-network-wired" title="No networks" description="This compose file defines no custom networks." />
      </template>

      <!-- ════════ Logs Tab ════════ -->
      <template v-if="activeTab === 'logs'">
        <div class="log-viewer">
          <div class="log-toolbar">
            <div class="log-filters">
              <span style="font-size:11px;color:var(--text-muted)">Tail:</span>
              <select v-model="logsTail" @change="fetchServiceLogs()" class="log-select">
                <option :value="50">50</option>
                <option :value="100">100</option>
                <option :value="500">500</option>
                <option :value="1000">1000</option>
                <option :value="5000">5000</option>
              </select>
            </div>
            <div class="log-controls">
              <button class="btn btn-ghost btn-sm" @click="fetchServiceLogs()" :disabled="logsLoading">
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
    </div>
  </template>
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
  padding: 20px; min-height: 250px;
}

/* ─── Log Viewer ─── */
.log-viewer { display: flex; flex-direction: column; }
.log-toolbar {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--border-light);
  gap: 8px; flex-wrap: wrap;
}
.log-filters { display: flex; gap: 12px; align-items: center; }
.log-controls { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; }
.log-select {
  background: var(--bg-tertiary); border: 1px solid var(--border-color);
  border-radius: 4px; color: var(--text-main); font-size: 11px;
  padding: 4px 8px; font-family: var(--font-sans);
}
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
</style>
