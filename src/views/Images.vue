<!-- ItzamBox — Images View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useDocker } from '../composables/useDocker'

const { images, loading, fetchImages, pullImage, removeImage } = useDocker()
const showPullModal = ref(false)
const pullImageName = ref('')
const pulling = ref(false)

onMounted(() => fetchImages())

async function handlePull() {
  if (!pullImageName.value) return
  pulling.value = true
  try {
    await pullImage(pullImageName.value)
    pullImageName.value = ''
    showPullModal.value = false
  } catch (e: any) { alert(e.toString()) }
  pulling.value = false
}

function formatBytes(b: number): string {
  if (b > 1e9) return (b / 1e9).toFixed(1) + ' GB'
  return Math.round(b / 1e6) + ' MB'
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i> <span class="current">Images</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">Images</h1>
    <div style="display:flex;gap:8px;">
      <button class="btn btn-secondary" @click="fetchImages">
        <i class="fa-solid fa-rotate"></i> Refresh
      </button>
      <button class="btn btn-primary" @click="showPullModal = true">
        <i class="fa-solid fa-cloud-arrow-down"></i> Pull Image
      </button>
    </div>
  </div>

  <div class="section">
    <div class="section-header">
      <span class="section-title">Local Images ({{ images.length }})</span>
    </div>
    <div v-if="images.length === 0" style="padding:60px;text-align:center;color:var(--text-muted)">
      <i class="fa-solid fa-layer-group" style="font-size:48px;margin-bottom:16px;opacity:0.3"></i>
      <p style="font-size:14px">No images found. Pull one to get started.</p>
    </div>
    <div v-for="img in images" :key="img.id" class="data-row">
      <div class="row-info">
        <div class="row-name">{{ img.repository }}</div>
        <div class="row-meta">{{ img.tag }} · {{ formatBytes(img.size_bytes) }}</div>
      </div>
      <span class="port-tag">{{ img.tag }}</span>
      <div class="row-actions">
        <button class="action-btn" @click="removeImage(img.id, true)" title="Remove">
          <i class="fa-solid fa-trash-can"></i>
        </button>
      </div>
    </div>
  </div>

  <!-- Pull Modal -->
  <div v-if="showPullModal" class="modal-backdrop" @click.self="showPullModal = false">
    <div class="modal-content">
      <div class="modal-header">
        <span class="modal-title"><i class="fa-solid fa-cloud-arrow-down"></i> Pull Image</span>
        <button class="header-btn" @click="showPullModal = false"><i class="fa-solid fa-xmark"></i></button>
      </div>
      <div class="modal-body">
        <div class="form-group">
          <label class="form-label">Image name</label>
          <input class="form-input mono" v-model="pullImageName" placeholder="nginx:latest" @keyup.enter="handlePull">
        </div>
      </div>
      <div class="modal-footer">
        <button class="btn btn-secondary" @click="showPullModal = false">Cancel</button>
        <button class="btn btn-primary" @click="handlePull" :disabled="pulling">
          <i class="fa-solid fa-cloud-arrow-down"></i> {{ pulling ? 'Pulling...' : 'Pull' }}
        </button>
      </div>
    </div>
  </div>
</template>
