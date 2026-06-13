<!-- ItzamBox — Docker Compose Projects List View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { detectComposeProjects, composeUp, composeDown, composeRestart, type ComposeProject } from '../../composables/useDocker'
import SkeletonLoader from '../../components/shared/SkeletonLoader.vue'
import EmptyState from '../../components/shared/EmptyState.vue'
import ErrorState from '../../components/shared/ErrorState.vue'

const router = useRouter()
const projects = ref<ComposeProject[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')
const actionLoading = ref<string | null>(null)

onMounted(() => loadData())

async function loadData() {
  loading.value = true
  error.value = null
  try {
    projects.value = await detectComposeProjects()
  } catch (e: any) {
    error.value = e.toString()
  }
  loading.value = false
}

const filteredProjects = computed(() => {
  if (!searchQuery.value) return projects.value
  const q = searchQuery.value.toLowerCase()
  return projects.value.filter(p =>
    p.name.toLowerCase().includes(q) ||
    p.path.toLowerCase().includes(q) ||
    p.services.some(s => s.toLowerCase().includes(q))
  )
})

const isEmpty = computed(() => filteredProjects.value.length === 0 && !loading.value && !error.value)

function goToDetail(project: ComposeProject) {
  router.push({ name: 'ComposeDetail', params: { name: project.name }, query: { path: project.path } })
}

async function handleAction(project: ComposeProject, action: 'up' | 'down' | 'restart') {
  actionLoading.value = `${action}-${project.name}`
  try {
    if (action === 'up') {
      await composeUp(project.path, true)
    } else if (action === 'down') {
      await composeDown(project.path, false, false)
    } else if (action === 'restart') {
      await composeRestart(project.path)
    }
  } catch (e: any) {
    error.value = e.toString()
  }
  actionLoading.value = null
}

async function handleLogs(project: ComposeProject) {
  goToDetail(project)
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span @click="router.push('/')" style="cursor:pointer">Home</span>
    <i class="fa-solid fa-chevron-right"></i>
    <span class="current">Compose</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Compose Projects</h1>
    <div style="display:flex;gap:8px">
      <button class="btn btn-secondary" @click="loadData" :disabled="loading">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> Refresh
      </button>
    </div>
  </div>

  <!-- Search bar -->
  <div v-if="projects.length > 0" class="table-filter" style="align-self:flex-start">
    <i class="fa-solid fa-search"></i>
    <input v-model="searchQuery" placeholder="Search projects..." />
  </div>

  <!-- Loading state -->
  <SkeletonLoader v-if="loading && projects.length === 0" variant="card" :rows="4" />

  <!-- Error state -->
  <ErrorState
    v-if="error && !loading"
    :message="'Failed to detect Compose projects'"
    :suggestion="'Make sure Docker Compose is installed and accessible.'"
    :detail="error"
    @retry="loadData"
  />

  <!-- Empty state -->
  <EmptyState
    v-if="isEmpty"
    icon="fa-solid fa-layer-group"
    title="No Compose projects found"
    description="Create a docker-compose.yml file in any directory to get started. ItzamBox will detect it automatically."
    action-label="Refresh"
    @action="loadData"
  />

  <!-- Project list -->
  <template v-if="!error && !loading">
    <div v-if="filteredProjects.length > 0" class="section">
      <div class="col-headers">
        <span class="col-header" style="flex:0 0 200px">PROJECT</span>
        <span class="col-header" style="flex:1">PATH</span>
        <span class="col-header" style="flex:0 0 100px">SERVICES</span>
        <span class="col-header" style="flex:0 0 100px">STATUS</span>
        <span style="flex:0 0 160px;text-align:right">ACTIONS</span>
      </div>
      <div
        v-for="project in filteredProjects"
        :key="project.name + project.path"
        class="data-row"
        @click="goToDetail(project)"
      >
        <div style="flex:0 0 200px;display:flex;align-items:center;gap:10px">
          <div style="
            width:32px;height:32px;border-radius:8px;
            background:rgba(168,85,247,0.1);color:var(--accent-purple);
            display:flex;align-items:center;justify-content:center;font-size:14px
          ">
            <i class="fa-solid fa-layer-group"></i>
          </div>
          <div class="row-name" style="font-size:13px">{{ project.name }}</div>
        </div>
        <div style="flex:1;min-width:0">
          <div class="row-meta" style="font-size:11px;color:var(--text-muted);font-family:var(--font-mono);white-space:nowrap;overflow:hidden;text-overflow:ellipsis">
            {{ project.path }}
          </div>
        </div>
        <div style="flex:0 0 100px">
          <span class="tag compose">{{ project.services.length }} service{{ project.services.length !== 1 ? 's' : '' }}</span>
        </div>
        <div style="flex:0 0 100px">
          <span v-if="project.services.length > 0" class="tag stopped" style="color:var(--text-muted)">
            <i class="fa-solid fa-circle" style="font-size:6px;margin-right:4px;color:var(--status-stopped)"></i>
            Detected
          </span>
        </div>
        <div class="row-actions" style="flex:0 0 160px;display:flex;gap:2px;justify-content:flex-end;opacity:1" @click.stop>
          <button
            class="action-btn"
            title="Up (detached)"
            :disabled="actionLoading === 'up-' + project.name"
            @click="handleAction(project, 'up')"
          >
            <i class="fa-solid fa-play" :class="{ 'fa-spin': actionLoading === 'up-' + project.name }"></i>
          </button>
          <button
            class="action-btn"
            title="Down"
            :disabled="actionLoading === 'down-' + project.name"
            @click="handleAction(project, 'down')"
          >
            <i class="fa-solid fa-stop" :class="{ 'fa-spin': actionLoading === 'down-' + project.name }"></i>
          </button>
          <button
            class="action-btn"
            title="Restart"
            :disabled="actionLoading === 'restart-' + project.name"
            @click="handleAction(project, 'restart')"
          >
            <i class="fa-solid fa-rotate-right" :class="{ 'fa-spin': actionLoading === 'restart-' + project.name }"></i>
          </button>
          <button class="action-btn" title="View Logs" @click="handleLogs(project)">
            <i class="fa-solid fa-scroll"></i>
          </button>
        </div>
      </div>
    </div>

    <!-- No results after filtering -->
    <EmptyState
      v-if="searchQuery && filteredProjects.length === 0 && projects.length > 0"
      icon="fa-solid fa-search"
      title="No matching projects"
      :description="'No compose projects match &quot;' + searchQuery + '&quot;'"
    />
  </template>
</template>

<style scoped>
.table-filter {
  display: flex; align-items: center; background: var(--bg-tertiary);
  border: 1px solid var(--border-color); border-radius: 6px;
  padding: 6px 10px; font-size: 12px; color: var(--text-muted); gap: 6px;
  width: 300px; max-width: 100%;
}
.table-filter input {
  background: none; border: none; outline: none;
  color: var(--text-main); font-size: 12px; font-family: var(--font-sans); width: 100%;
}
.table-filter input::placeholder { color: var(--text-disabled); }
.table-filter:focus-within { border-color: var(--accent-cyan); }
</style>
