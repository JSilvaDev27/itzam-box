<!-- ItzamBox — Kubernetes Service Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { K8sService } from '../../composables/useKubernetes'

const props = defineProps<{
  services: K8sService[]
  loading: boolean
}>()

const emit = defineEmits<{
  'inspect': [service: K8sService]
}>()

const sortColumn = ref<string>('name')
const sortDirection = ref<'asc' | 'desc'>('asc')
const filterQuery = ref('')

const sortedServices = computed(() => {
  let list = [...props.services]
  if (filterQuery.value) {
    const q = filterQuery.value.toLowerCase()
    list = list.filter(s => s.name.toLowerCase().includes(q) || s.namespace.toLowerCase().includes(q))
  }
  list.sort((a, b) => {
    let cmp = 0
    switch (sortColumn.value) {
      case 'name': cmp = a.name.localeCompare(b.name); break
      case 'namespace': cmp = a.namespace.localeCompare(b.namespace); break
      case 'type': cmp = a.service_type.localeCompare(b.service_type); break
      case 'cluster-ip': cmp = a.cluster_ip.localeCompare(b.cluster_ip); break
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

function typeClass(type: string): string {
  const t = type.toLowerCase()
  if (t === 'loadbalancer') return 'k8s-type--lb'
  if (t === 'nodeport') return 'k8s-type--np'
  if (t === 'clusterip') return 'k8s-type--cip'
  if (t === 'externalname') return 'k8s-type--ext'
  return 'k8s-type--default'
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
        <input v-model="filterQuery" placeholder="Filter services..." />
      </div>
      <span class="k8s-table-count">{{ sortedServices.length }} service{{ sortedServices.length !== 1 ? 's' : '' }}</span>
    </div>

    <div class="k8s-col-headers">
      <span class="k8s-col k8s-col--name" @click="toggleSort('name')">Name<span class="k8s-sort">{{ sortIndicator('name') }}</span></span>
      <span class="k8s-col k8s-col--ns" @click="toggleSort('namespace')">Namespace<span class="k8s-sort">{{ sortIndicator('namespace') }}</span></span>
      <span class="k8s-col k8s-col--type" @click="toggleSort('type')">Type<span class="k8s-sort">{{ sortIndicator('type') }}</span></span>
      <span class="k8s-col k8s-col--cip" @click="toggleSort('cluster-ip')">Cluster IP<span class="k8s-sort">{{ sortIndicator('cluster-ip') }}</span></span>
      <span class="k8s-col k8s-col--eip">External IP</span>
      <span class="k8s-col k8s-col--ports">Ports</span>
      <span class="k8s-col k8s-col--age" @click="toggleSort('age')">Age<span class="k8s-sort">{{ sortIndicator('age') }}</span></span>
    </div>

    <div class="k8s-table-body" v-if="!loading">
      <div v-for="svc in sortedServices" :key="svc.name + '/' + svc.namespace"
        class="k8s-data-row" @click="emit('inspect', svc)">
        <span class="k8s-cell k8s-cell--name"><i class="fa-solid fa-share-nodes k8s-cell-icon"></i>{{ svc.name }}</span>
        <span class="k8s-cell k8s-cell--ns">{{ svc.namespace }}</span>
        <span class="k8s-cell k8s-cell--type">
          <span :class="['k8s-type-badge', typeClass(svc.service_type)]">{{ svc.service_type }}</span>
        </span>
        <span class="k8s-cell k8s-cell--cip k8s-cell--mono">{{ svc.cluster_ip }}</span>
        <span class="k8s-cell k8s-cell--eip k8s-cell--mono">{{ svc.external_ip ?? '—' }}</span>
        <span class="k8s-cell k8s-cell--ports">
          <span v-for="p in svc.ports" :key="p" class="k8s-port-chip">{{ p }}</span>
        </span>
        <span class="k8s-cell k8s-cell--age">{{ svc.age }}</span>
      </div>
      <div v-if="sortedServices.length === 0" class="k8s-table-empty">
        <i class="fa-solid fa-share-nodes"></i>
        <p>No services found</p>
      </div>
    </div>

    <div v-else class="k8s-table-body">
      <div v-for="i in 5" :key="i" class="k8s-data-row k8s-skeleton-row">
        <span class="k8s-cell k8s-cell--name"><span class="skeleton skeleton-text-md" style="width:140px"></span></span>
        <span class="k8s-cell k8s-cell--ns"><span class="skeleton skeleton-text-sm" style="width:80px"></span></span>
        <span class="k8s-cell k8s-cell--type"><span class="skeleton skeleton-tag"></span></span>
        <span class="k8s-cell k8s-cell--cip"><span class="skeleton skeleton-text-sm" style="width:100px"></span></span>
        <span class="k8s-cell k8s-cell--eip"><span class="skeleton skeleton-text-sm" style="width:80px"></span></span>
        <span class="k8s-cell k8s-cell--ports"><span class="skeleton skeleton-tag"></span></span>
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
.k8s-col--type { flex: 0 0 100px; }
.k8s-col--cip { flex: 1; min-width: 0; }
.k8s-col--eip { flex: 1; min-width: 0; }
.k8s-col--ports { flex: 1.5; min-width: 0; overflow: visible; }
.k8s-col--age { flex: 0 0 80px; }
.k8s-table-body { flex: 1; overflow-y: auto; }
.k8s-data-row { display: flex; align-items: center; padding: 10px 16px; border-bottom: 1px solid var(--border-light); gap: 8px; cursor: pointer; transition: background var(--transition-fast); }
.k8s-data-row:hover { background: var(--bg-hover); }
.k8s-skeleton-row { cursor: default; pointer-events: none; }
.k8s-cell { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-main); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.k8s-cell--name { flex: 2; min-width: 0; }
.k8s-cell--ns { flex: 1; min-width: 0; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell--type { flex: 0 0 100px; }
.k8s-cell--cip { flex: 1; min-width: 0; }
.k8s-cell--eip { flex: 1; min-width: 0; }
.k8s-cell--ports { flex: 1.5; min-width: 0; overflow: visible; display: flex; flex-wrap: wrap; gap: 3px; }
.k8s-cell--age { flex: 0 0 80px; font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell--mono { font-family: var(--font-mono); font-size: 11px; color: var(--text-muted); }
.k8s-cell-icon { color: var(--accent-blue); font-size: 12px; flex-shrink: 0; }

.k8s-type-badge { padding: 2px 8px; border-radius: 10px; font-size: 10px; font-weight: 600; white-space: nowrap; }
.k8s-type--lb { background: rgba(168, 85, 247, 0.1); color: var(--accent-purple); border: 1px solid rgba(168, 85, 247, 0.2); }
.k8s-type--np { background: rgba(0, 229, 255, 0.08); color: var(--accent-cyan); border: 1px solid rgba(0, 229, 255, 0.15); }
.k8s-type--cip { background: rgba(59, 130, 246, 0.08); color: var(--accent-blue); border: 1px solid rgba(59, 130, 246, 0.15); }
.k8s-type--ext { background: var(--bg-tertiary); color: var(--text-muted); border: 1px solid var(--border-color); }
.k8s-type--default { background: var(--bg-tertiary); color: var(--text-muted); border: 1px solid var(--border-color); }

.k8s-port-chip { display: inline-block; font-family: var(--font-mono); font-size: 10px; padding: 1px 5px; background: var(--bg-tertiary); border: 1px solid var(--border-color); border-radius: 3px; color: var(--text-muted); white-space: nowrap; }

.k8s-table-empty { display: flex; flex-direction: column; align-items: center; padding: 48px 20px; color: var(--text-disabled); gap: 8px; }
.k8s-table-empty i { font-size: 28px; color: var(--text-disabled); }
.k8s-table-empty p { font-size: 13px; }
</style>
