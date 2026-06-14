<!-- ItzamBox — Kubernetes Deployment Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { K8sDeployment } from '../../composables/useKubernetes'

const props = defineProps<{
  deployments: K8sDeployment[]
  loading: boolean
}>()

const emit = defineEmits<{
  'inspect': [deployment: K8sDeployment]
}>()

const sortColumn = ref<string>('name')
const sortDirection = ref<'asc' | 'desc'>('asc')
const filterQuery = ref('')

const sortedDeployments = computed(() => {
  let list = [...props.deployments]
  if (filterQuery.value) {
    const q = filterQuery.value.toLowerCase()
    list = list.filter(d => d.name.toLowerCase().includes(q) || d.namespace.toLowerCase().includes(q))
  }
  list.sort((a, b) => {
    let cmp = 0
    switch (sortColumn.value) {
      case 'name': cmp = a.name.localeCompare(b.name); break
      case 'namespace': cmp = a.namespace.localeCompare(b.namespace); break
      case 'ready': cmp = a.ready.localeCompare(b.ready); break
      case 'up-to-date': cmp = a.up_to_date - b.up_to_date; break
      case 'available': cmp = a.available - b.available; break
      case 'age': cmp = a.age.localeCompare(b.age); break
      default: cmp = 0
    }
    return sortDirection.value === 'asc' ? cmp : -cmp
  })
  return list
})

function toggleSort(col: string) {
  if (sortColumn.value === col) sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
  else { sortColumn.value = col; sortDirection.value = 'asc' }
}

function readyState(ready: string): 'full' | 'partial' | 'none' {
  const [readyCount, totalCount] = ready.split('/').map(Number)
  if (readyCount === 0) return 'none'
  if (readyCount >= totalCount) return 'full'
  return 'partial'
}

const sortIndicator = (col: string): string => {
  if (sortColumn.value !== col) return ''
  return sortDirection.value === 'asc' ? ' ▲' : ' ▼'
}
</script>

<template>
  <div class="k8s-table-wrapper">
    <div class="k8s-table-toolbar">
      <div class="k8s-table-filter">
        <i class="fa-solid fa-search"></i>
        <input v-model="filterQuery" placeholder="Filter deployments..." />
      </div>
      <span class="k8s-table-count">{{ sortedDeployments.length }} deployment{{ sortedDeployments.length !== 1 ? 's' : '' }}</span>
    </div>

    <div class="k8s-col-headers">
      <span class="k8s-col k8s-col--name" @click="toggleSort('name')">Name<span class="k8s-sort">{{ sortIndicator('name') }}</span></span>
      <span class="k8s-col k8s-col--ns" @click="toggleSort('namespace')">Namespace<span class="k8s-sort">{{ sortIndicator('namespace') }}</span></span>
      <span class="k8s-col k8s-col--ready" @click="toggleSort('ready')">Ready<span class="k8s-sort">{{ sortIndicator('ready') }}</span></span>
      <span class="k8s-col k8s-col--num" @click="toggleSort('up-to-date')">Up-to-Date<span class="k8s-sort">{{ sortIndicator('up-to-date') }}</span></span>
      <span class="k8s-col k8s-col--num" @click="toggleSort('available')">Available<span class="k8s-sort">{{ sortIndicator('available') }}</span></span>
      <span class="k8s-col k8s-col--age" @click="toggleSort('age')">Age<span class="k8s-sort">{{ sortIndicator('age') }}</span></span>
    </div>

    <div class="k8s-table-body" v-if="!loading">
      <div v-for="dep in sortedDeployments" :key="dep.name + '/' + dep.namespace"
        class="k8s-data-row" @click="emit('inspect', dep)">
        <span class="k8s-cell k8s-cell--name"><i class="fa-solid fa-layer-group k8s-cell-icon"></i>{{ dep.name }}</span>
        <span class="k8s-cell k8s-cell--ns">{{ dep.namespace }}</span>
        <span class="k8s-cell k8s-cell--ready">
          <span :class="['k8s-ready-badge', `k8s-ready--${readyState(dep.ready)}`]">{{ dep.ready }}</span>
        </span>
        <span class="k8s-cell k8s-cell--num">{{ dep.up_to_date }}</span>
        <span class="k8s-cell k8s-cell--num">{{ dep.available }}</span>
        <span class="k8s-cell k8s-cell--age">{{ dep.age }}</span>
      </div>
      <div v-if="sortedDeployments.length === 0" class="k8s-table-empty">
        <i class="fa-solid fa-layer-group"></i>
        <p>No deployments found</p>
      </div>
    </div>

    <div v-else class="k8s-table-body">
      <div v-for="i in 5" :key="i" class="k8s-data-row k8s-skeleton-row">
        <span class="k8s-cell k8s-cell--name"><span class="skeleton skeleton-text-md" style="width:160px"></span></span>
        <span class="k8s-cell k8s-cell--ns"><span class="skeleton skeleton-text-sm" style="width:80px"></span></span>
        <span class="k8s-cell k8s-cell--ready"><span class="skeleton skeleton-tag"></span></span>
        <span class="k8s-cell k8s-cell--num"><span class="skeleton skeleton-text-xs"></span></span>
        <span class="k8s-cell k8s-cell--num"><span class="skeleton skeleton-text-xs"></span></span>
        <span class="k8s-cell k8s-cell--age"><span class="skeleton skeleton-text-xs"></span></span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.k8s-table-wrapper { display: flex; flex-direction: column; overflow: hidden; }
.k8s-table-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; border-bottom: 1px solid var(--border-light); }
.k8s-table-filter { display: flex; align-items: center; gap: 6px; background: var(--bg-tertiary); border: 1px solid var(--border-color); border-radius: 6px; padding: 5px 10px; }
.k8s-table-filter i { color: var(--text-disabled); font-size: 12px; }
.k8s-table-filter input { background: none; border: none; outline: none; color: var(--text-main); font-size: 12px; font-family: var(--font-sans); width: 200px; }
.k8s-table-filter input::placeholder { color: var(--text-disabled); }
.k8s-table-count { font-size: 11px; color: var(--text-muted); font-family: var(--font-mono); }
.k8s-col-headers { display: flex; align-items: center; padding: 6px 16px; font-size: 10px; font-weight: 600; color: var(--text-disabled); text-transform: uppercase; letter-spacing: 0.04em; background: var(--bg-tertiary); border-bottom: 1px solid var(--border-light); gap: 8px; }
.k8s-col { cursor: pointer; display: flex; align-items: center; gap: 3px; user-select: none; }
.k8s-col:hover { color: var(--text-muted); }
.k8s-sort { font-size: 8px; opacity: 0.4; font-family: var(--font-mono); }
.k8s-col--name { flex: 2; min-width: 0; }
.k8s-col--ns { flex: 1; min-width: 0; }
.k8s-col--ready { flex: 0 0 90px; }
.k8s-col--num { flex: 0 0 80px; justify-content: center; }
.k8s-col--age { flex: 0 0 80px; }
.k8s-table-body { flex: 1; overflow-y: auto; }
.k8s-data-row { display: flex; align-items: center; padding: 10px 16px; border-bottom: 1px solid var(--border-light); gap: 8px; cursor: pointer; transition: background var(--transition-fast); }
.k8s-data-row:hover { background: var(--bg-hover); }
.k8s-skeleton-row { cursor: default; pointer-events: none; }
.k8s-cell { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-main); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.k8s-cell--name { flex: 2; min-width: 0; }
.k8s-cell--ns { flex: 1; min-width: 0; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell--ready { flex: 0 0 90px; }
.k8s-cell--num { flex: 0 0 80px; justify-content: center; font-family: var(--font-mono); font-size: 12px; }
.k8s-cell--age { flex: 0 0 80px; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell-icon { color: var(--accent-purple); font-size: 12px; flex-shrink: 0; }

.k8s-ready-badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; font-family: var(--font-mono); }
.k8s-ready--full { background: rgba(16, 185, 129, 0.1); color: var(--accent-green); border: 1px solid rgba(16, 185, 129, 0.2); }
.k8s-ready--partial { background: rgba(245, 158, 11, 0.08); color: var(--accent-yellow); border: 1px solid rgba(245, 158, 11, 0.15); }
.k8s-ready--none { background: rgba(239, 68, 68, 0.08); color: var(--accent-red); border: 1px solid rgba(239, 68, 68, 0.2); }

.k8s-table-empty { display: flex; flex-direction: column; align-items: center; padding: 48px 20px; color: var(--text-disabled); gap: 8px; }
.k8s-table-empty i { font-size: 28px; color: var(--text-disabled); }
.k8s-table-empty p { font-size: 13px; }
</style>
