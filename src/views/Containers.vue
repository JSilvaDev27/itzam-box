<!-- ItzamBox — Containers View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useDocker } from '../composables/useDocker'
import { useContextMenu, containerContextMenu } from '../composables/useContextMenu'
import { useNotifications } from '../composables/useNotifications'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const router = useRouter()
const { containers, fetchContainers, startContainer, stopContainer, restartContainer, pauseContainer, unpauseContainer, removeContainer, error: dockerError } = useDocker()
const { show } = useContextMenu()
const { success, error: notifyError } = useNotifications()
const loading = ref(false)
// Alias the reactive error ref from useDocker so the template's v-if="error" picks up errors from fetchContainers
const error = dockerError

onMounted(() => loadData())

async function loadData() {
  loading.value = true
  error.value = null
  try {
    await fetchContainers(true)
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

const running = computed(() => containers.value.filter(c => c.state === 'running'))
const paused = computed(() => containers.value.filter(c => c.state === 'paused'))
const stopped = computed(() => containers.value.filter(c => c.state === 'exited' || c.state === 'stopped'))
const isEmpty = computed(() => containers.value.length === 0 && !loading.value && !error.value)

const composeGroups = computed(() => {
  const groups: Record<string, typeof containers.value> = {}
  const standalone: typeof containers.value = []
  for (const c of containers.value) {
    if (c.compose_project) {
      if (!groups[c.compose_project]) groups[c.compose_project] = []
      groups[c.compose_project].push(c)
    } else {
      standalone.push(c)
    }
  }
  return { groups, standalone }
})

function goToDetail(id: string) {
  router.push('/containers/' + id)
}

function getContainerCallbacks(c: typeof containers.value[0]) {
  return {
    onStart: async () => {
      try { await startContainer(c.id); success('Container started', `${c.name} is now running.`); await fetchContainers(true) }
      catch (e: any) { notifyError('Failed to start', e.toString()) }
    },
    onStop: async () => {
      try { await stopContainer(c.id); success('Container stopped', `${c.name} has been stopped.`); await fetchContainers(true) }
      catch (e: any) { notifyError('Failed to stop', e.toString()) }
    },
    onRestart: async () => {
      try { await restartContainer(c.id); success('Container restarted', `${c.name} is restarting.`); await fetchContainers(true) }
      catch (e: any) { notifyError('Failed to restart', e.toString()) }
    },
    onPause: async () => {
      try { await pauseContainer(c.id); success('Container paused', `${c.name} is now paused.`); await fetchContainers(true) }
      catch (e: any) { notifyError('Failed to pause', e.toString()) }
    },
    onLogs: () => { router.push('/containers/' + c.id + '?tab=logs') },
    onTerminal: () => { router.push('/containers/' + c.id + '?tab=terminal') },
    onInspect: () => { router.push('/containers/' + c.id + '?tab=info') },
    onFiles: () => { router.push('/containers/' + c.id + '/files') },
    onRemove: async () => {
      if (!confirm(`Remove container "${c.name}"? This action cannot be undone.`)) return
      try { await removeContainer(c.id, true); success('Container removed', `${c.name} has been removed.`); await fetchContainers(true) }
      catch (e: any) { notifyError('Failed to remove', e.toString()) }
    },
  }
}

async function handleAction(id: string, action: string) {
  try {
    if (action === 'start') await startContainer(id)
    else if (action === 'stop') await stopContainer(id)
    else if (action === 'restart') await restartContainer(id)
    else if (action === 'pause') await pauseContainer(id)
    else if (action === 'unpause') await unpauseContainer(id)
    else if (action === 'remove') await removeContainer(id, true)
    await fetchContainers(true)
  } catch (e: any) { error.value = e.toString() }
}
</script>

<template>
  <div class="view-root">
  <div class="breadcrumb"><i class="fa-solid fa-house"></i> <span>Home</span> <i class="fa-solid fa-chevron-right"></i> <span class="current">Containers</span></div>
  
  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Containers</h1>
    <div style="display:flex;gap:8px">
      <button class="btn btn-secondary" @click="loadData" :disabled="loading">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> Refresh
      </button>
    </div>
  </div>

  <!-- Loading state (A.6.1) -->
  <SkeletonLoader v-if="loading && containers.length === 0" variant="table-row" :rows="5" />

  <!-- Error state (A.6.3) -->
  <ErrorState
    v-if="error"
    :message="'Error loading containers'"
    :suggestion="'Check that Docker is running and accessible.'"
    :detail="error"
    @retry="loadData"
  />

  <!-- Content (only when not loading and no error) -->
  <template v-if="!error && !loading">
    <!-- Empty state (A.6.2) -->
    <EmptyState
      v-if="isEmpty"
      icon="fa-solid fa-cubes"
      title="No containers found"
      description="Create your first container or pull an image to get started."
      action-label="Pull Image"
      secondary-label="View Images"
      @action="$router.push('/images')"
      @secondary="$router.push('/images')"
    />

    <template v-else>
      <div class="summary-bar">
        <div class="summary-chip all">All <span class="summary-count">{{ containers.length }}</span></div>
        <div class="summary-chip running"><i class="fa-solid fa-circle" style="font-size:7px;color:var(--accent-green)"></i> Running <span class="summary-count">{{ running.length }}</span></div>
        <div class="summary-chip paused"><i class="fa-solid fa-circle-pause" style="font-size:10px"></i> Paused <span class="summary-count">{{ paused.length }}</span></div>
        <div class="summary-chip stopped"><i class="fa-solid fa-circle-stop" style="font-size:10px"></i> Stopped <span class="summary-count">{{ stopped.length }}</span></div>
      </div>

      <!-- Compose groups -->
      <div v-for="(groupContainers, project) in composeGroups.groups" :key="project" class="compose-group">
        <div class="compose-group-header">
          <div class="compose-group-name"><i class="fa-solid fa-folder-tree"></i> {{ project }} <span class="tag compose">compose</span></div>
        </div>
        <div v-for="c in groupContainers" :key="c.id" class="data-row" @click="goToDetail(c.id)" @contextmenu="show($event, containerContextMenu(c, getContainerCallbacks(c)))">
          <span :class="['status-dot', c.state === 'running' ? 'status-dot--running' : c.state === 'paused' ? 'status-dot--paused' : 'status-dot--stopped']"></span>
          <div class="row-info"><div class="row-name">{{ c.name }}</div><div class="row-meta">{{ c.image }}</div></div>
          <span :class="['tag', c.state === 'running' ? 'tag running' : c.state === 'paused' ? 'tag paused' : 'tag stopped']">{{ c.state }}</span>
          <span style="font-size:12px;color:var(--text-muted)">{{ c.status }}</span>
          <div class="row-actions">
            <button v-if="c.state === 'stopped' || c.state === 'exited'" class="action-btn" @click.stop="handleAction(c.id, 'start')"><i class="fa-solid fa-play"></i></button>
            <button v-if="c.state === 'running'" class="action-btn" @click.stop="handleAction(c.id, 'stop')"><i class="fa-solid fa-stop"></i></button>
            <button v-if="c.state === 'running'" class="action-btn" @click.stop="handleAction(c.id, 'restart')"><i class="fa-solid fa-rotate-right"></i></button>
            <button v-if="c.state !== 'running'" class="action-btn" data-testid="remove-container" @click.stop="handleAction(c.id, 'remove')"><i class="fa-solid fa-trash-can"></i></button>
          </div>
        </div>
      </div>

      <!-- Standalone containers -->
      <div class="section">
        <div class="section-header"><span class="section-title">Standalone</span></div>
        <div v-if="composeGroups.standalone.length === 0 && Object.keys(composeGroups.groups).length === 0" style="padding:60px;text-align:center;color:var(--text-muted)">
          <i class="fa-solid fa-cubes" style="font-size:48px;margin-bottom:16px;opacity:0.3"></i>
          <p style="font-size:14px">No containers found</p>
        </div>
        <div v-for="c in composeGroups.standalone" :key="c.id" class="data-row" @click="goToDetail(c.id)" @contextmenu="show($event, containerContextMenu(c, getContainerCallbacks(c)))">
          <span :class="['status-dot', c.state === 'running' ? 'status-dot--running' : c.state === 'paused' ? 'status-dot--paused' : 'status-dot--stopped']"></span>
          <div class="row-info"><div class="row-name">{{ c.name }}</div><div class="row-meta">{{ c.image }}</div></div>
          <span :class="['tag', c.state === 'running' ? 'tag running' : c.state === 'paused' ? 'tag paused' : 'tag stopped']">{{ c.state }}</span>
          <span style="font-size:12px;color:var(--text-muted)">{{ c.status }}</span>
          <div class="row-actions">
            <button v-if="c.state === 'stopped' || c.state === 'exited'" class="action-btn" @click.stop="handleAction(c.id, 'start')"><i class="fa-solid fa-play"></i></button>
            <button v-if="c.state === 'running'" class="action-btn" @click.stop="handleAction(c.id, 'stop')"><i class="fa-solid fa-stop"></i></button>
            <button v-if="c.state === 'running'" class="action-btn" @click.stop="handleAction(c.id, 'restart')"><i class="fa-solid fa-rotate-right"></i></button>
            <button v-if="c.state !== 'running'" class="action-btn" data-testid="remove-container" @click.stop="handleAction(c.id, 'remove')"><i class="fa-solid fa-trash-can"></i></button>
          </div>
        </div>
      </div>
    </template>
  </template>
  </div>
</template>
