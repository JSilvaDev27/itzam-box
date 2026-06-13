<!-- ItzamBox — Images View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useDocker } from '../composables/useDocker'
import { useContextMenu, imageContextMenu } from '../composables/useContextMenu'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const { images, fetchImages, pullImage, removeImage } = useDocker()
const { show } = useContextMenu()
const loading = ref(false)
const error = ref<string | null>(null)
const showPullModal = ref(false)
const pullImageName = ref('')
const pulling = ref(false)

onMounted(() => loadData())

async function loadData() {
  loading.value = true
  error.value = null
  try { await fetchImages() } catch (e: any) { error.value = e.toString() }
  loading.value = false
}

const isEmpty = computed(() => images.value.length === 0 && !loading.value && !error.value)

async function handlePull() {
  if (!pullImageName.value) return
  pulling.value = true
  try {
    await pullImage(pullImageName.value)
    pullImageName.value = ''
    showPullModal.value = false
    await loadData()
  } catch (e: any) { error.value = e.toString() }
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
      <button class="btn btn-secondary" @click="loadData" :disabled="loading">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i> Refresh
      </button>
      <button class="btn btn-primary" @click="showPullModal = true">
        <i class="fa-solid fa-cloud-arrow-down"></i> Pull Image
      </button>
    </div>
  </div>

  <!-- Loading state (A.6.1) -->
  <SkeletonLoader v-if="loading && images.length === 0" variant="table-row" :rows="4" />

  <!-- Error state (A.6.3) -->
  <ErrorState
    v-if="error"
    :message="'Error loading images'"
    :suggestion="'Verify Docker daemon connectivity and try again.'"
    :detail="error"
    @retry="loadData"
  />

  <!-- Content -->
  <template v-if="!loading && !error">
    <!-- Empty state (A.6.2) -->
    <EmptyState
      v-if="isEmpty"
      icon="fa-solid fa-layer-group"
      title="No images found"
      description="Pull your first image from Docker Hub to start building containers."
      action-label="Pull Image"
      @action="showPullModal = true"
    />

    <div v-else class="section">
      <div class="section-header">
        <span class="section-title">Local Images ({{ images.length }})</span>
      </div>
      <div v-for="img in images" :key="img.id" class="data-row" @contextmenu="show($event, imageContextMenu(img))">
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
  </template>

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
