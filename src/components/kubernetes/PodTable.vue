<!-- ItzamBox — Kubernetes Pod Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { K8sPod } from '../../composables/useKubernetes'

const props = defineProps<{
  pods: K8sPod[]
  loading: boolean
}>()

const emit = defineEmits<{
  'inspect': [pod: K8sPod]
}>()

const sortColumn = ref<string>('name')
const sortDirection = ref<'asc' | 'desc'>('asc')
const filterQuery = ref('')

const sortedPods = computed(() => {
  let list = [...props.pods]

  // Filter
  if (filterQuery.value) {
    const q = filterQuery.value.toLowerCase()
    list = list.filter(p => p.name.toLowerCase().includes(q) || p.namespace.toLowerCase().includes(q))
  }

  // Sort
  list.sort((a, b) => {
    let cmp = 0
    switch (sortColumn.value) {
      case 'name': cmp = a.name.localeCompare(b.name); break
      case 'namespace': cmp = a.namespace.localeCompare(b.namespace); break
      case 'status': cmp = a.status.localeCompare(b.status); break
      case 'restarts': cmp = a.restarts - b.restarts; break
      case 'age': cmp = a.age.localeCompare(b.age); break
      case 'node': cmp = a.node.localeCompare(b.node); break
      default: cmp = 0
    }
    return sortDirection.value === 'asc' ? cmp : -cmp
  })

  return list
})

function toggleSort(col: string) {
  if (sortColumn.value === col) {
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortColumn.value = col
    sortDirection.value = 'asc'
  }
}

function statusClass(status: string): string {
  const s = status.toLowerCase()
  if (s === 'running') return 'k8s-pill--running'
  if (s === 'pending') return 'k8s-pill--pending'
  if (s === 'succeeded' || s === 'completed') return 'k8s-pill--succeeded'
  if (s === 'failed') return 'k8s-pill--failed'
  if (s.includes('crash') || s.includes('error') || s === 'unknown') return 'k8s-pill--error'
  return 'k8s-pill--default'
}

const statusIcon = (status: string): string => {
  const s = status.toLowerCase()
  if (s === 'running') return 'fa-solid fa-play'
  if (s === 'pending') return 'fa-solid fa-hourglass-half'
  if (s === 'succeeded') return 'fa-solid fa-check'
  if (s === 'failed') return 'fa-solid fa-xmark'
  if (s.includes('crash')) return 'fa-solid fa-bug'
  return 'fa-solid fa-circle'
}

function isCrashLoop(status: string): boolean {
  return status.toLowerCase().includes('crashloopbackoff')
}

const sortIndicator = (col: string): string => {
  if (sortColumn.value !== col) return ''
  return sortDirection.value === 'asc' ? ' ▲' : ' ▼'
}
</script>

<template>
  <div class="k8s-table-wrapper">
    <!-- Filter -->
    <div class="k8s-table-toolbar">
      <div class="k8s-table-filter">
        <i class="fa-solid fa-search"></i>
        <input v-model="filterQuery" placeholder="Filter pods by name..." />
      </div>
      <span class="k8s-table-count">{{ sortedPods.length }} pod{{ sortedPods.length !== 1 ? 's' : '' }}</span>
    </div>

    <!-- Column Headers -->
    <div class="k8s-col-headers">
      <span class="k8s-col k8s-col--name" @click="toggleSort('name')">Name<span class="k8s-sort">{{ sortIndicator('name') }}</span></span>
      <span class="k8s-col k8s-col--ns" @click="toggleSort('namespace')">Namespace<span class="k8s-sort">{{ sortIndicator('namespace') }}</span></span>
      <span class="k8s-col k8s-col--status" @click="toggleSort('status')">Status<span class="k8s-sort">{{ sortIndicator('status') }}</span></span>
      <span class="k8s-col k8s-col--restarts" @click="toggleSort('restarts')">Restarts<span class="k8s-sort">{{ sortIndicator('restarts') }}</span></span>
      <span class="k8s-col k8s-col--age" @click="toggleSort('age')">Age<span class="k8s-sort">{{ sortIndicator('age') }}</span></span>
      <span class="k8s-col k8s-col--node" @click="toggleSort('node')">Node<span class="k8s-sort">{{ sortIndicator('node') }}</span></span>
    </div>

    <!-- Body -->
    <div class="k8s-table-body" v-if="!loading">
      <div
        v-for="pod in sortedPods"
        :key="pod.name + '/' + pod.namespace"
        class="k8s-data-row"
        @click="emit('inspect', pod)"
      >
        <span class="k8s-cell k8s-cell--name">
          <i class="fa-solid fa-cube k8s-cell-icon"></i>
          <span class="k8s-cell-value">{{ pod.name }}</span>
        </span>
        <span class="k8s-cell k8s-cell--ns">{{ pod.namespace }}</span>
        <span class="k8s-cell k8s-cell--status">
          <span :class="['k8s-pill', statusClass(pod.status)]">
            <i :class="statusIcon(pod.status)" class="k8s-pill-icon"></i>
            {{ pod.status }}
          </span>
        </span>
        <span class="k8s-cell k8s-cell--restarts" :class="{ 'k8s-cell--danger': isCrashLoop(pod.status) }">
          {{ pod.restarts }}
        </span>
        <span class="k8s-cell k8s-cell--age">{{ pod.age }}</span>
        <span class="k8s-cell k8s-cell--node">
          <i class="fa-solid fa-server k8s-cell-icon-sub"></i>
          {{ pod.node }}
        </span>
      </div>
      <div v-if="sortedPods.length === 0" class="k8s-table-empty">
        <i class="fa-solid fa-cubes"></i>
        <p>No pods found</p>
      </div>
    </div>

    <!-- Loading skeleton -->
    <div v-else class="k8s-table-body">
      <div v-for="i in 8" :key="i" class="k8s-data-row k8s-skeleton-row">
        <span class="k8s-cell k8s-cell--name"><span class="skeleton skeleton-text-md" style="width:140px"></span></span>
        <span class="k8s-cell k8s-cell--ns"><span class="skeleton skeleton-text-sm" style="width:80px"></span></span>
        <span class="k8s-cell k8s-cell--status"><span class="skeleton skeleton-tag"></span></span>
        <span class="k8s-cell k8s-cell--restarts"><span class="skeleton skeleton-text-xs"></span></span>
        <span class="k8s-cell k8s-cell--age"><span class="skeleton skeleton-text-xs"></span></span>
        <span class="k8s-cell k8s-cell--node"><span class="skeleton skeleton-text-sm" style="width:100px"></span></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.k8s-table-wrapper {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.k8s-table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--border-light);
}

.k8s-table-filter {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 5px 10px;
}

.k8s-table-filter i {
  color: var(--text-disabled);
  font-size: 12px;
}

.k8s-table-filter input {
  background: none;
  border: none;
  outline: none;
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
  width: 200px;
}

.k8s-table-filter input::placeholder {
  color: var(--text-disabled);
}

.k8s-table-count {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.k8s-col-headers {
  display: flex;
  align-items: center;
  padding: 6px 16px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
  gap: 8px;
}

.k8s-col {
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 3px;
  user-select: none;
  transition: color var(--transition-fast);
}

.k8s-col:hover {
  color: var(--text-muted);
}

.k8s-sort {
  font-size: 8px;
  opacity: 0.4;
  font-family: var(--font-mono);
}

.k8s-col:hover .k8s-sort {
  opacity: 0.8;
}

.k8s-col--name { flex: 2; min-width: 0; }
.k8s-col--ns { flex: 1; min-width: 0; }
.k8s-col--status { flex: 1; min-width: 0; }
.k8s-col--restarts { flex: 0 0 70px; justify-content: center; }
.k8s-col--age { flex: 0 0 80px; }
.k8s-col--node { flex: 1.5; min-width: 0; }

.k8s-table-body {
  flex: 1;
  overflow-y: auto;
}

.k8s-data-row {
  display: flex;
  align-items: center;
  padding: 10px 16px;
  border-bottom: 1px solid var(--border-light);
  gap: 8px;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.k8s-data-row:hover {
  background: var(--bg-hover);
}

.k8s-skeleton-row {
  cursor: default;
  pointer-events: none;
}

.k8s-cell {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-main);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.k8s-cell--name { flex: 2; min-width: 0; }
.k8s-cell--ns { flex: 1; min-width: 0; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell--status { flex: 1; min-width: 0; }
.k8s-cell--restarts { flex: 0 0 70px; justify-content: center; font-family: var(--font-mono); font-size: 12px; }
.k8s-cell--age { flex: 0 0 80px; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell--node { flex: 1.5; min-width: 0; font-size: 11px; color: var(--text-muted); font-family: var(--font-mono); }

.k8s-cell--danger {
  color: var(--accent-red);
  font-weight: 700;
}

.k8s-cell-icon {
  color: var(--accent-cyan);
  font-size: 12px;
  flex-shrink: 0;
}

.k8s-cell-icon-sub {
  color: var(--text-disabled);
  font-size: 10px;
  flex-shrink: 0;
}

.k8s-cell-value {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Status Pills ── */
.k8s-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 600;
}

.k8s-pill-icon {
  font-size: 7px;
}

.k8s-pill--running {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.k8s-pill--pending {
  background: rgba(245, 158, 11, 0.08);
  color: var(--accent-yellow);
  border: 1px solid rgba(245, 158, 11, 0.15);
}

.k8s-pill--succeeded {
  background: rgba(59, 130, 246, 0.1);
  color: var(--accent-blue);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.k8s-pill--failed {
  background: rgba(239, 68, 68, 0.08);
  color: var(--accent-red);
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.k8s-pill--error {
  background: rgba(239, 68, 68, 0.1);
  color: var(--accent-red);
  border: 1px solid rgba(239, 68, 68, 0.3);
  animation: k8s-pulse-danger 1.5s infinite;
}

.k8s-pill--default {
  background: var(--bg-tertiary);
  color: var(--text-muted);
  border: 1px solid var(--border-color);
}

@keyframes k8s-pulse-danger {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}

.k8s-table-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 48px 20px;
  color: var(--text-disabled);
  gap: 8px;
}

.k8s-table-empty i {
  font-size: 28px;
  color: var(--text-disabled);
}

.k8s-table-empty p {
  font-size: 13px;
}
</style>
