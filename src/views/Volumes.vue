<!-- ItzamBox — Volumes View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useDocker } from '../composables/useDocker'
import { invoke } from '@tauri-apps/api/core'

const { volumes, fetchVolumes } = useDocker()
const showCreate = ref(false)
const volName = ref('')
const volDriver = ref('')

onMounted(() => fetchVolumes())

async function createVolume() {
  if (!volName.value) return
  try {
    await invoke('create_volume', { name: volName.value, driver: volDriver.value || null })
    volName.value = ''; volDriver.value = ''; showCreate.value = false
    await fetchVolumes()
  } catch (e: any) { alert(e.toString()) }
}

async function removeVolume(name: string) {
  if (!confirm(`Delete volume "${name}"?`)) return
  try {
    await invoke('remove_volume', { name, force: true })
    await fetchVolumes()
  } catch (e: any) { alert(e.toString()) }
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Volumes</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Volumes</h1>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary" @click="fetchVolumes"><i class="fa-solid fa-rotate"></i> Refresh</button>
      <button class="btn btn-primary" @click="showCreate = true"><i class="fa-solid fa-plus"></i> Create Volume</button>
    </div>
  </div>

  <div class="section">
    <div class="section-header"><span class="section-title">Volumes ({{ volumes.length }})</span></div>
    <div v-if="volumes.length === 0" style="padding:60px;text-align:center;color:var(--text-muted)">
      <i class="fa-solid fa-database" style="font-size:48px;margin-bottom:16px;opacity:0.3"></i>
      <p style="font-size:14px">No volumes created.</p>
    </div>
    <div v-for="v in volumes" :key="v.name" class="data-row">
      <div class="row-info">
        <div class="row-name">{{ v.name }}</div>
        <div class="row-meta">Driver: {{ v.driver }} · {{ v.mountpoint }}</div>
      </div>
      <div class="row-actions">
        <button class="action-btn" @click="removeVolume(v.name)" title="Remove">
          <i class="fa-solid fa-trash-can"></i>
        </button>
      </div>
    </div>
  </div>

  <!-- Create Modal -->
  <div v-if="showCreate" class="modal-backdrop" @click.self="showCreate = false">
    <div class="modal-content">
      <div class="modal-header">
        <span class="modal-title"><i class="fa-solid fa-database"></i> Create Volume</span>
        <button class="header-btn" @click="showCreate = false"><i class="fa-solid fa-xmark"></i></button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label class="form-label">Volume name</label>
          <input class="form-input mono" v-model="volName" placeholder="my-volume" @keyup.enter="createVolume">
        </div>
        <div class="form-group">
          <label class="form-label">Driver (optional)</label>
          <input class="form-input mono" v-model="volDriver" placeholder="local">
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="showCreate = false">Cancel</button>
        <button class="btn btn-primary" @click="createVolume"><i class="fa-solid fa-plus"></i> Create</button>
      </div>
    </div>
  </div>
</template>
