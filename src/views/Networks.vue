<!-- ItzamBox — Networks View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { NetworkInfo } from '../composables/useDocker'
import { useContextMenu, networkContextMenu } from '../composables/useContextMenu'
import { useNotifications } from '../composables/useNotifications'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const networks = ref<NetworkInfo[]>([])
const { show } = useContextMenu()
const { info, success, error: notifyError } = useNotifications()
const loading = ref(false)
const error = ref<string | null>(null)
const showCreate = ref(false)
const netName = ref('')
const netDriver = ref('bridge')

onMounted(() => loadData())

async function loadData() {
  loading.value = true
  error.value = null
  try { networks.value = await invoke<NetworkInfo[]>('list_networks') } catch(e: any) { error.value = e.toString() }
  loading.value = false
}

const isEmpty = computed(() => networks.value.length === 0 && !loading.value && !error.value)

async function createNetwork() {
  if (!netName.value) return
  try {
    await invoke('create_network', { name: netName.value, driver: netDriver.value, subnet: null, gateway: null })
    netName.value = ''; showCreate.value = false
    await loadData()
  } catch(e: any) { error.value = e.toString() }
}

async function removeNetwork(id: string) {
  if (!confirm('Delete this network?')) return
  try { await invoke('remove_network', { id }); await loadData() } catch(e: any) { error.value = e.toString() }
}

function getNetworkCallbacks(n: NetworkInfo) {
  return {
    onInspect: () => {
      info(
        `Network: ${n.name}`,
        `ID: ${n.id}\nDriver: ${n.driver}\nScope: ${n.scope}`
      )
    },
    onRemove: async () => {
      try {
        await removeNetwork(n.id)
        success('Network removed', `${n.name} removed successfully.`)
      } catch (e: any) {
        notifyError('Failed to remove network', e.toString())
      }
    },
  }
}
</script>

<template>
  <div class="breadcrumb"><i class="fa-solid fa-house"></i> <span>Home</span> <i class="fa-solid fa-chevron-right"></i> <span class="current">Networks</span></div>
  
  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Networks</h1>
    <div style="display:flex;gap:8px">
      <button class="btn btn-secondary" @click="loadData" :disabled="loading">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> Refresh
      </button>
      <button class="btn btn-primary" @click="showCreate = true"><i class="fa-solid fa-plus"></i> Create Network</button>
    </div>
  </div>

  <!-- Loading state (A.6.1) -->
  <SkeletonLoader v-if="loading && networks.length === 0" variant="table-row" :rows="3" />

  <!-- Error state (A.6.3) -->
  <ErrorState
    v-if="error"
    :message="'Error loading networks'"
    :suggestion="'Docker networking service may not be available.'"
    :detail="error"
    @retry="loadData"
  />

  <!-- Content -->
  <template v-if="!loading && !error">
    <!-- Empty state (A.6.2) -->
    <EmptyState
      v-if="isEmpty"
      icon="fa-solid fa-network-wired"
      title="No custom networks"
      description="Create a network to allow containers to communicate with each other."
      action-label="Create Network"
      @action="showCreate = true"
    />

    <div v-else class="section">
      <div class="section-header"><span class="section-title">Networks ({{ networks.length }})</span></div>
      <div v-for="n in networks" :key="n.id" class="data-row" @contextmenu="show($event, networkContextMenu(n, getNetworkCallbacks(n)))">
        <div class="row-info"><div class="row-name">{{ n.name }}</div><div class="row-meta">Driver: {{ n.driver }} · Scope: {{ n.scope }}</div></div>
        <span class="port-tag">{{ n.driver }}</span>
        <div class="row-actions">
          <button class="action-btn" @click="removeNetwork(n.id)"><i class="fa-solid fa-trash-can"></i></button>
        </div>
      </div>
    </div>
  </template>

  <!-- Create Modal -->
  <div v-if="showCreate" class="modal-backdrop" @click.self="showCreate = false">
    <div class="modal-content">
      <div class="modal-header"><span class="modal-title"><i class="fa-solid fa-network-wired"></i> Create Network</span><button class="header-btn" @click="showCreate = false"><i class="fa-solid fa-xmark"></i></button></div>
      <div class="modal-body">
        <div class="form-group"><label class="form-label">Name</label><input class="form-input mono" v-model="netName" placeholder="my-network"></div>
        <div class="form-group"><label class="form-label">Driver</label><input class="form-input mono" v-model="netDriver" placeholder="bridge"></div>
      </div>
      <div class="modal-footer"><button class="btn btn-secondary" @click="showCreate = false">Cancel</button><button class="btn btn-primary" @click="createNetwork"><i class="fa-solid fa-plus"></i> Create</button></div>
    </div>
  </div>
</template>
