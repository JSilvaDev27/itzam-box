<!-- ItzamBox — Networks View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { NetworkInfo } from '../composables/useDocker'

const networks = ref<NetworkInfo[]>([])
const showCreate = ref(false)
const netName = ref('')
const netDriver = ref('bridge')

onMounted(fetchNetworks)

async function fetchNetworks() {
  try { networks.value = await invoke<NetworkInfo[]>('list_networks') } catch(e: any) { alert(e.toString()) }
}
async function createNetwork() {
  if (!netName.value) return
  try { await invoke('create_network', { name: netName.value, driver: netDriver.value, subnet: null, gateway: null }); netName.value = ''; showCreate.value = false; await fetchNetworks() } catch(e: any) { alert(e.toString()) }
}
async function removeNetwork(id: string) {
  try { await invoke('remove_network', { id }); await fetchNetworks() } catch(e: any) { alert(e.toString()) }
}
</script>

<template>
  <div class="breadcrumb"><i class="fa-solid fa-house"></i> <span>Home</span> <i class="fa-solid fa-chevron-right"></i> <span class="current">Networks</span></div>
  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Networks</h1>
    <div style="display:flex;gap:8px">
      <button class="btn btn-secondary" @click="fetchNetworks"><i class="fa-solid fa-rotate"></i> Refresh</button>
      <button class="btn btn-primary" @click="showCreate = true"><i class="fa-solid fa-plus"></i> Create Network</button>
    </div>
  </div>
  <div class="section">
    <div class="section-header"><span class="section-title">Networks ({{ networks.length }})</span></div>
    <div v-if="networks.length === 0" style="padding:60px;text-align:center;color:var(--text-muted)">
      <i class="fa-solid fa-network-wired" style="font-size:48px;margin-bottom:16px;opacity:0.3"></i><p style="font-size:14px">No custom networks</p>
    </div>
    <div v-for="n in networks" :key="n.id" class="data-row">
      <div class="row-info"><div class="row-name">{{ n.name }}</div><div class="row-meta">Driver: {{ n.driver }} · Scope: {{ n.scope }}</div></div>
      <span class="port-tag">{{ n.driver }}</span>
      <div class="row-actions">
        <button class="action-btn" @click="removeNetwork(n.id)"><i class="fa-solid fa-trash-can"></i></button>
      </div>
    </div>
  </div>
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
