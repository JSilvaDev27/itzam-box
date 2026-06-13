<!-- ItzamBox — Export / Import Images & Containers
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { useDocker, type ContainerInfo, type ImageInfo } from '../composables/useDocker'
import { useNotifications } from '../composables/useNotifications'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const {
  containers, images, fetchContainers, fetchImages,
  exportContainer, commitContainer, saveImage, loadImage,
} = useDocker()
const { success, error: notifyError, info } = useNotifications()

// ─── State ───────────────────────────────────────────────────────────────
const activeTab = ref<'export' | 'import'>('export')
const loadingContainers = ref(false)
const loadingImages = ref(false)
const pageError = ref<string | null>(null)

// Export Container
const exportContainerId = ref('')
const exportOutputPath = ref('')
const exportingContainer = ref(false)

// Commit Container
const commitContainerId = ref('')
const commitRepository = ref('')
const commitTag = ref('latest')
const commitMessage = ref('')
const commitAuthor = ref('')
const committing = ref(false)
const commitResultId = ref<string | null>(null)

// Save Image
const saveImageName = ref('')
const saveOutputPath = ref('')
const savingImage = ref(false)

// Load Image
const loadInputPath = ref('')
const loadingImage = ref(false)
const loadResult = ref<string | null>(null)

// ─── Lifecycle ───────────────────────────────────────────────────────────
onMounted(async () => {
  loadingContainers.value = true
  loadingImages.value = true
  try {
    await fetchContainers(true)
  } catch (e: any) {
    pageError.value = e.toString()
  }
  loadingContainers.value = false

  try {
    await fetchImages()
  } catch (e: any) {
    // Non-critical
    console.warn('Failed to fetch images:', e.toString())
  }
  loadingImages.value = false
})

// ─── Computed ────────────────────────────────────────────────────────────
const runningContainers = computed(() =>
  containers.value.filter(c => c.state === 'running')
)

const allContainers = computed(() => containers.value)

const localImages = computed(() => images.value)

const isEmptyContainers = computed(() =>
  allContainers.value.length === 0 && !loadingContainers.value
)

const isEmptyImages = computed(() =>
  localImages.value.length === 0 && !loadingImages.value
)

// ─── Export Container ─────────────────────────────────────────────────────
async function chooseExportPath() {
  try {
    const selected = await save({
      defaultPath: `container-${exportContainerId.value || 'export'}.tar`,
      filters: [{ name: 'Tar Archive', extensions: ['tar'] }],
      title: 'Save Container Export As',
    })
    if (selected) {
      exportOutputPath.value = selected
    }
  } catch {
    // Dialog cancelled
  }
}

async function handleExportContainer() {
  if (!exportContainerId.value || !exportOutputPath.value) {
    notifyError('Missing fields', 'Please select a container and output path.')
    return
  }
  exportingContainer.value = true
  try {
    await exportContainer(exportContainerId.value, exportOutputPath.value)
    success('Container exported', `Exported to ${exportOutputPath.value}`)
    info('Path', exportOutputPath.value)
  } catch (e: any) {
    notifyError('Export failed', e.toString())
  }
  exportingContainer.value = false
}

// ─── Commit Container ─────────────────────────────────────────────────────
async function handleCommitContainer() {
  if (!commitContainerId.value || !commitRepository.value) {
    notifyError('Missing fields', 'Please select a container and enter a repository name.')
    return
  }
  committing.value = true
  commitResultId.value = null
  try {
    const imageId = await commitContainer(
      commitContainerId.value,
      commitRepository.value,
      commitTag.value,
      commitMessage.value || null,
      commitAuthor.value || null,
    )
    commitResultId.value = imageId
    success('Container committed', `New image: ${commitRepository.value}:${commitTag.value}`)
    // Refresh images list
    await fetchImages()
  } catch (e: any) {
    notifyError('Commit failed', e.toString())
  }
  committing.value = false
}

// ─── Save Image ───────────────────────────────────────────────────────────
async function chooseSavePath() {
  try {
    const selected = await save({
      defaultPath: `${(saveImageName.value || 'image').replace(/[/:]/g, '-')}.tar`,
      filters: [{ name: 'Tar Archive', extensions: ['tar'] }],
      title: 'Save Image As',
    })
    if (selected) {
      saveOutputPath.value = selected
    }
  } catch {
    // Dialog cancelled
  }
}

async function handleSaveImage() {
  if (!saveImageName.value || !saveOutputPath.value) {
    notifyError('Missing fields', 'Please select an image and output path.')
    return
  }
  savingImage.value = true
  try {
    await saveImage(saveImageName.value, saveOutputPath.value)
    success('Image saved', `Saved to ${saveOutputPath.value}`)
  } catch (e: any) {
    notifyError('Save failed', e.toString())
  }
  savingImage.value = false
}

// ─── Load Image ───────────────────────────────────────────────────────────
async function chooseLoadPath() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Docker Tar Archive', extensions: ['tar', 'gz'] }],
      title: 'Select Image Archive to Load',
    })
    if (selected && typeof selected === 'string') {
      loadInputPath.value = selected
    }
  } catch {
    // Dialog cancelled
  }
}

async function handleLoadImage() {
  if (!loadInputPath.value) {
    notifyError('Missing file', 'Please select a tar archive to load.')
    return
  }
  loadingImage.value = true
  loadResult.value = null
  try {
    const result = await loadImage(loadInputPath.value)
    loadResult.value = result
    success('Image loaded', result)
    // Refresh images list
    await fetchImages()
  } catch (e: any) {
    notifyError('Load failed', e.toString())
  }
  loadingImage.value = false
}

// ─── Drag & Drop ─────────────────────────────────────────────────────────
const dragOver = ref(false)
const dropError = ref<string | null>(null)

async function onDrop(event: DragEvent) {
  event.preventDefault()
  dragOver.value = false
  dropError.value = null

  const files = event.dataTransfer?.files
  if (!files || files.length === 0) return

  const file = files[0]
  const name = file.name.toLowerCase()
  if (!name.endsWith('.tar') && !name.endsWith('.tar.gz') && !name.endsWith('.tgz')) {
    dropError.value = 'Only .tar, .tar.gz, and .tgz files are supported.'
    return
  }

  loadInputPath.value = file.name
  // For Tauri, we need the full path from the file dialog instead
  // Use the drop event's path if available (webkitRelativePath)
  if (file instanceof File) {
    // On Tauri webview, the path property may be available
    const path = (file as any).path
    if (path) {
      loadInputPath.value = path
      return
    }
  }

  // Fallback: use a simulated path — inform user to use the file picker
  dropError.value = 'Drag & drop from the OS file manager requires the "Choose file" dialog. Please use the button below.'
}

function onDragOver(event: DragEvent) {
  event.preventDefault()
  dragOver.value = true
}

function onDragLeave() {
  dragOver.value = false
}
</script>

<template>
  <div class="breadcrumb">
    <i class="fa-solid fa-house"></i> <span>Home</span>
    <i class="fa-solid fa-chevron-right"></i>
    <i class="fa-solid fa-file-export" style="margin-right:4px"></i>
    <span class="current">Export / Import</span>
  </div>

  <div style="display:flex;align-items:center;justify-content:space-between;">
    <h1 class="text-h1">
      <i class="fa-solid fa-file-export" style="color:var(--accent-cyan);margin-right:10px"></i>
      Export / Import
    </h1>
    <div style="display:flex;gap:8px">
      <button class="btn btn-secondary" @click="fetchContainers(true); fetchImages()" :disabled="loadingContainers || loadingImages">
        <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loadingContainers || loadingImages }"></i> Refresh
      </button>
    </div>
  </div>

  <!-- ✅ Success Banner for Load Image -->
  <div v-if="loadResult" class="success-banner">
    <i class="fa-solid fa-circle-check"></i>
    <span>{{ loadResult }}</span>
    <button class="btn btn-ghost btn-sm" @click="loadResult = null">
      <i class="fa-solid fa-xmark"></i>
    </button>
  </div>

  <!-- Tabs -->
  <div class="tabs">
    <button
      :class="['tab', { active: activeTab === 'export' }]"
      @click="activeTab = 'export'"
    >
      <i class="fa-solid fa-file-export"></i> Export
    </button>
    <button
      :class="['tab', { active: activeTab === 'import' }]"
      @click="activeTab = 'import'"
    >
      <i class="fa-solid fa-file-import"></i> Import
    </button>
  </div>

  <!-- ─── TAB: Export ──────────────────────────────────────────────────── -->
  <div v-if="activeTab === 'export'" class="export-tab">
    <div class="card-grid">
      <!-- Export Container -->
      <div class="action-card">
        <div class="action-card-header">
          <i class="fa-solid fa-cubes" style="color:var(--accent-cyan)"></i>
          <h2 class="action-card-title">Export Container</h2>
        </div>
        <p class="action-card-desc">Export a container's filesystem as a tar archive.</p>

        <div class="form-group">
          <label class="form-label">Container</label>
          <select v-model="exportContainerId" class="form-input">
            <option value="" disabled>Select a container...</option>
            <option
              v-for="c in allContainers"
              :key="c.id"
              :value="c.id"
            >
              {{ c.name }} ({{ c.id.substring(0, 12) }}) — {{ c.state }}
            </option>
          </select>
          <p v-if="isEmptyContainers" class="form-hint">No containers found. Start a container first.</p>
        </div>

        <div class="form-group">
          <label class="form-label">Output path</label>
          <div class="input-with-browse">
            <input
              v-model="exportOutputPath"
              class="form-input mono"
              placeholder="/home/user/container-export.tar"
              readonly
            />
            <button class="btn btn-secondary btn-sm" @click="chooseExportPath">
              <i class="fa-solid fa-folder-open"></i> Choose file...
            </button>
          </div>
        </div>

        <button
          class="btn btn-primary"
          @click="handleExportContainer"
          :disabled="exportingContainer || !exportContainerId || !exportOutputPath"
        >
          <i class="fa-solid fa-file-arrow-down" :class="{ 'fa-spin': exportingContainer }"></i>
          {{ exportingContainer ? 'Exporting...' : 'Export' }}
        </button>
      </div>

      <!-- Commit Container -->
      <div class="action-card">
        <div class="action-card-header">
          <i class="fa-solid fa-camera" style="color:var(--accent-purple)"></i>
          <h2 class="action-card-title">Commit Container</h2>
        </div>
        <p class="action-card-desc">Create a new image from a container's changes.</p>

        <div class="form-group">
          <label class="form-label">Container</label>
          <select v-model="commitContainerId" class="form-input">
            <option value="" disabled>Select a container...</option>
            <option
              v-for="c in allContainers"
              :key="c.id"
              :value="c.id"
            >
              {{ c.name }} ({{ c.id.substring(0, 12) }}) — {{ c.state }}
            </option>
          </select>
        </div>

        <div class="form-group">
          <label class="form-label">Repository</label>
          <input
            v-model="commitRepository"
            class="form-input mono"
            placeholder="my-custom-image"
          />
        </div>

        <div style="display:flex;gap:8px">
          <div class="form-group" style="flex:1">
            <label class="form-label">Tag</label>
            <input
              v-model="commitTag"
              class="form-input mono"
              placeholder="latest"
            />
          </div>
        </div>

        <div class="form-group">
          <label class="form-label">Message (optional)</label>
          <input
            v-model="commitMessage"
            class="form-input"
            placeholder="Snapshot before update"
          />
        </div>

        <div class="form-group">
          <label class="form-label">Author (optional)</label>
          <input
            v-model="commitAuthor"
            class="form-input"
            placeholder="user@example.com"
          />
        </div>

        <button
          class="btn btn-primary"
          @click="handleCommitContainer"
          :disabled="committing || !commitContainerId || !commitRepository"
        >
          <i class="fa-solid fa-camera" :class="{ 'fa-spin': committing }"></i>
          {{ committing ? 'Committing...' : 'Commit' }}
        </button>

        <!-- Commit result -->
        <div v-if="commitResultId" class="result-banner">
          <i class="fa-solid fa-check-circle" style="color:var(--accent-green)"></i>
          <div>
            <strong>Image created:</strong>
            <code class="mono">{{ commitResultId }}</code>
          </div>
        </div>
      </div>

      <!-- Save Image -->
      <div class="action-card">
        <div class="action-card-header">
          <i class="fa-solid fa-layer-group" style="color:var(--accent-green)"></i>
          <h2 class="action-card-title">Save Image</h2>
        </div>
        <p class="action-card-desc">Save one or more images as a tar archive.</p>

        <div class="form-group">
          <label class="form-label">Image</label>
          <select v-model="saveImageName" class="form-input">
            <option value="" disabled>Select an image...</option>
            <option
              v-for="img in localImages"
              :key="img.id"
              :value="`${img.repository}:${img.tag}`"
            >
              {{ img.repository }}:{{ img.tag }}
            </option>
          </select>
          <p v-if="isEmptyImages" class="form-hint">No images found. Pull an image first.</p>
        </div>

        <div class="form-group">
          <label class="form-label">Output path</label>
          <div class="input-with-browse">
            <input
              v-model="saveOutputPath"
              class="form-input mono"
              placeholder="/home/user/image-save.tar"
              readonly
            />
            <button class="btn btn-secondary btn-sm" @click="chooseSavePath">
              <i class="fa-solid fa-folder-open"></i> Choose file...
            </button>
          </div>
        </div>

        <button
          class="btn btn-primary"
          @click="handleSaveImage"
          :disabled="savingImage || !saveImageName || !saveOutputPath"
        >
          <i class="fa-solid fa-floppy-disk" :class="{ 'fa-spin': savingImage }"></i>
          {{ savingImage ? 'Saving...' : 'Save' }}
        </button>
      </div>
    </div>
  </div>

  <!-- ─── TAB: Import ──────────────────────────────────────────────────── -->
  <div v-if="activeTab === 'import'" class="import-tab">
    <div class="card-grid">
      <!-- Load Image -->
      <div class="action-card">
        <div class="action-card-header">
          <i class="fa-solid fa-file-arrow-up" style="color:var(--accent-cyan)"></i>
          <h2 class="action-card-title">Load Image</h2>
        </div>
        <p class="action-card-desc">Load an image from a tar archive produced by <code>docker save</code>.</p>

        <div class="form-group">
          <label class="form-label">Archive file (.tar / .tar.gz)</label>
          <div class="input-with-browse">
            <input
              v-model="loadInputPath"
              class="form-input mono"
              placeholder="/home/user/image.tar"
              readonly
            />
            <button class="btn btn-secondary btn-sm" @click="chooseLoadPath">
              <i class="fa-solid fa-folder-open"></i> Choose file...
            </button>
          </div>
        </div>

        <button
          class="btn btn-primary"
          @click="handleLoadImage"
          :disabled="loadingImage || !loadInputPath"
        >
          <i class="fa-solid fa-cloud-arrow-up" :class="{ 'fa-spin': loadingImage }"></i>
          {{ loadingImage ? 'Loading...' : 'Load' }}
        </button>
      </div>

      <!-- Drag & Drop Area -->
      <div
        :class="['drop-zone', { 'drag-over': dragOver }]"
        @drop="onDrop"
        @dragover="onDragOver"
        @dragleave="onDragLeave"
      >
        <div class="drop-zone-icon">
          <i class="fa-solid fa-cloud-arrow-up"></i>
        </div>
        <h3 class="drop-zone-title">Drop tar archive here</h3>
        <p class="drop-zone-desc">Drag & drop a <code>.tar</code> or <code>.tar.gz</code> file to load it automatically.</p>

        <div v-if="dropError" class="drop-error">
          <i class="fa-solid fa-triangle-exclamation"></i> {{ dropError }}
        </div>
      </div>
    </div>
  </div>

  <!-- Loading states (A.6.1) -->
  <SkeletonLoader
    v-if="loadingContainers || loadingImages"
    variant="card"
    :count="3"
  />

  <!-- Error state (A.6.3) -->
  <ErrorState
    v-if="pageError"
    :message="pageError"
    suggestion="Make sure Docker is running and try again."
    @retry="onMounted"
  />
</template>

<style scoped>
/* ─── Breadcrumb ─── */
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

.breadcrumb .current {
  color: var(--text-main);
  font-weight: 500;
}

/* ─── Tabs ─── */
.tabs {
  display: flex;
  gap: 0;
  margin: 20px 0;
  border-bottom: 1px solid var(--border-color);
}

.tab {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 14px;
  font-weight: 500;
  font-family: var(--font-sans);
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all var(--transition-fast);
  margin-bottom: -1px;
}

.tab:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.tab.active {
  color: var(--accent-cyan);
  border-bottom-color: var(--accent-cyan);
}

/* ─── Card Grid ─── */
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(420px, 1fr));
  gap: 20px;
  margin-top: 8px;
}

/* ─── Action Card ─── */
.action-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 20px;
  transition: all var(--transition-fast);
}

.action-card:hover {
  border-color: rgba(0, 229, 255, 0.12);
  box-shadow: var(--shadow-md);
}

.action-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 4px;
}

.action-card-header i {
  font-size: 20px;
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  background: var(--bg-tertiary);
}

.action-card-title {
  font-size: 1.1rem;
  font-weight: 600;
  letter-spacing: -0.01em;
}

.action-card-desc {
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 16px;
  line-height: 1.5;
}

.action-card-desc code {
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--bg-tertiary);
  padding: 1px 4px;
  border-radius: 3px;
}

/* ─── Input with Browse Button ─── */
.input-with-browse {
  display: flex;
  gap: 6px;
  align-items: center;
}

.input-with-browse input {
  flex: 1;
}

/* ─── Form overrides ─── */
.form-hint {
  margin-top: 4px;
  font-size: 11px;
  color: var(--text-disabled);
}

/* ─── Result Banner ─── */
.result-banner {
  margin-top: 12px;
  padding: 10px 14px;
  background: rgba(16, 185, 129, 0.06);
  border: 1px solid rgba(16, 185, 129, 0.15);
  border-radius: var(--radius-md);
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--accent-green);
  word-break: break-all;
}

.result-banner code {
  font-family: var(--font-mono);
  font-size: 11px;
}

/* ─── Success Banner ─── */
.success-banner {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: rgba(16, 185, 129, 0.08);
  border: 1px solid rgba(16, 185, 129, 0.2);
  border-radius: var(--radius-md);
  margin-bottom: 16px;
  font-size: 13px;
  color: var(--accent-green);
  animation: fadeIn 0.3s ease-out;
  flex-wrap: wrap;
}

.success-banner i:first-child {
  font-size: 18px;
}

.success-banner span {
  flex: 1;
}

/* ─── Drop Zone ─── */
.drop-zone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 32px;
  border: 2px dashed var(--border-color);
  border-radius: var(--radius-lg);
  background: var(--bg-secondary);
  transition: all var(--transition-fast);
  cursor: pointer;
  text-align: center;
  min-height: 220px;
}

.drop-zone:hover,
.drop-zone.drag-over {
  border-color: var(--accent-cyan);
  background: rgba(0, 229, 255, 0.03);
  box-shadow: var(--shadow-glow);
}

.drop-zone.drag-over {
  transform: scale(1.02);
}

.drop-zone-icon {
  width: 64px;
  height: 64px;
  border-radius: 16px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: var(--accent-cyan);
  margin-bottom: 16px;
}

.drop-zone-title {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: 4px;
}

.drop-zone-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.5;
}

.drop-zone-desc code {
  font-family: var(--font-mono);
  font-size: 11px;
  background: var(--bg-tertiary);
  padding: 1px 4px;
  border-radius: 3px;
}

.drop-error {
  margin-top: 12px;
  padding: 8px 12px;
  background: rgba(239, 68, 68, 0.06);
  border: 1px solid rgba(239, 68, 68, 0.15);
  border-radius: var(--radius-sm);
  font-size: 12px;
  color: var(--accent-red);
  display: flex;
  align-items: center;
  gap: 6px;
}

.mono {
  font-family: var(--font-mono);
}

/* ─── Responsive ─── */
@media (max-width: 768px) {
  .card-grid {
    grid-template-columns: 1fr;
  }

  .input-with-browse {
    flex-direction: column;
  }
}
</style>
