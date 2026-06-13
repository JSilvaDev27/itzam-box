<!-- ItzamBox — Images View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { useDocker } from '../composables/useDocker'
import { useContextMenu, imageContextMenu } from '../composables/useContextMenu'
import { useNotifications } from '../composables/useNotifications'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const router = useRouter()
const { images, fetchImages, pullImage, removeImage } = useDocker()
const { show } = useContextMenu()
const { info, success, error: notifyError } = useNotifications()
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

function getImageCallbacks(img: { id: string; repository: string; tag: string; size_bytes: number }) {
  return {
    onRun: () => {
      info('Run Container', `Open Containers view to create a container from ${img.repository}:${img.tag}.`)
    },
    onPull: async () => {
      try {
        pullImageName.value = img.repository + (img.tag && img.tag !== 'latest' ? `:${img.tag}` : '')
        showPullModal.value = true
      } catch (e: any) { /* handled by modal */ }
    },
    onRemove: async () => {
      try {
        await removeImage(img.id, true)
        success('Image removed', `${img.repository}:${img.tag} removed successfully.`)
        await loadData()
      } catch (e: any) {
        notifyError('Failed to remove image', e.toString())
      }
    },
    onInspect: () => {
      info(
        `Image: ${img.repository}:${img.tag}`,
        `ID: ${img.id}\nSize: ${formatBytes(img.size_bytes)}\nRepository: ${img.repository}\nTag: ${img.tag}`
      )
    },
    onTag: async () => {
      try {
        await invoke('tag_image', { id: img.id, repo: img.repository, tag: img.tag })
        success('Image tagged', `${img.repository}:${img.tag} tagged.`)
        await loadData()
      } catch (e: any) {
        notifyError('Failed to tag image', e.toString())
      }
    },
  }
}

function navigateToLayers(img: { id: string }) {
  router.push(`/images/${img.id}/layers`)
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
      <div v-for="img in images" :key="img.id" class="data-row" @click="navigateToLayers(img)" @contextmenu="show($event, imageContextMenu(img, getImageCallbacks(img)))">
        <div class="row-info">
          <div class="row-name">{{ img.repository }}</div>
          <div class="row-meta">{{ img.tag }} · {{ formatBytes(img.size_bytes) }}</div>
        </div>
        <span class="port-tag">{{ img.tag }}</span>
        <div class="row-actions">
          <button class="action-btn" @click.stop="navigateToLayers(img)" title="View layers">
            <i class="fa-solid fa-layer-group"></i>
          </button>
          <button class="action-btn" @click.stop="removeImage(img.id, true)" title="Remove">
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
