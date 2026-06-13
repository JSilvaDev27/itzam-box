<!-- ItzamBox — Container File Explorer
     Copyright (C) 2026 SodigTech — GPL-3.0
     Allows navigating and managing files inside a running Docker container -->
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import {
  listContainerDir, downloadFile, uploadFile, readFilePreview,
  sanitizePath, formatFileSize, formatFileTime, getFileIconClass,
  type FileMetadata, type ContainerInfo,
} from '../composables/useDocker'
import { useContextMenu } from '../composables/useContextMenu'
import SkeletonLoader from '../components/shared/SkeletonLoader.vue'
import EmptyState from '../components/shared/EmptyState.vue'
import ErrorState from '../components/shared/ErrorState.vue'

const route = useRoute()
const router = useRouter()
const { show: showContextMenu } = useContextMenu()

// ─── State ──────────────────────────────────────────────────────────────

const props = defineProps<{ id: string }>()
const containerId = computed(() => props.id || (route.params.id as string))

const container = ref<ContainerInfo | null>(null)
const files = ref<FileMetadata[]>([])
const currentPath = ref('/')
const history = ref<string[]>(['/']) // navigation history for breadcrumb
const loading = ref(true)
const error = ref<string | null>(null)
const errorDetail = ref<string | null>(null)
const searchQuery = ref('')
const showHidden = ref(false)

// Preview state
const previewFile = ref<FileMetadata | null>(null)
const previewContent = ref<string>('')
const previewLoading = ref(false)
const previewError = ref<string | null>(null)
const showPreview = ref(false)

// Upload state
const uploadLoading = ref(false)
const uploadError = ref<string | null>(null)

// ─── Computed ───────────────────────────────────────────────────────────

const sortedFiles = computed(() => {
  let filtered = files.value.filter(f => {
    // Filter hidden files
    if (!showHidden.value && f.name.startsWith('.')) return false
    // Filter by search query
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      return f.name.toLowerCase().includes(q)
    }
    return true
  })

  // Sort: directories first, then alphabetical
  return filtered.sort((a, b) => {
    if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1
    return a.name.localeCompare(b.name)
  })
})

const fileCount = computed(() => files.value.filter(f => !f.is_dir).length)
const dirCount = computed(() => files.value.filter(f => f.is_dir).length)

const breadcrumbSegments = computed(() => {
  const parts = currentPath.value.split('/').filter(Boolean)
  const segments: { label: string; path: string }[] = []
  let accum = ''
  for (const part of parts) {
    accum += '/' + part
    segments.push({ label: part, path: accum })
  }
  return segments
})

// ─── Navigation ─────────────────────────────────────────────────────────

async function goToHistory(index: number) {
  history.value = history.value.slice(0, index + 1)
  const target = history.value[index]
  if (target !== currentPath.value) {
    currentPath.value = target
    await loadDirectory(target)
  }
}

function goHome() {
  history.value = ['/']
  currentPath.value = '/'
  loadDirectory('/')
}

async function enterDir(file: FileMetadata) {
  if (!file.is_dir) return
  history.value.push(file.full_path)
  currentPath.value = file.full_path
  await loadDirectory(file.full_path)
}

// ─── Data Loading ──────────────────────────────────────────────────────

async function loadDirectory(path: string) {
  loading.value = true
  error.value = null
  errorDetail.value = null
  try {
    const sanitized = sanitizePath(path)
    files.value = await listContainerDir(containerId.value, sanitized)
  } catch (e: any) {
    const msg = e.toString()
    error.value = msg.includes('permission denied')
      ? 'Permission denied'
      : msg.includes('No such container')
        ? 'Container not found'
        : msg.includes('is not running')
          ? 'Container is not running — file explorer requires a running container'
          : 'Failed to load directory'
    errorDetail.value = msg
    files.value = []
  }
  loading.value = false
}

async function loadContainer() {
  try {
    const containers = await invoke<ContainerInfo[]>('list_containers', { showAll: true })
    const found = containers.find(c => c.id.startsWith(containerId.value) || c.name === containerId.value)
    if (!found) {
      error.value = 'Container not found'
      return
    }
    container.value = found
    if (found.state !== 'running') {
      error.value = 'Container is not running — file explorer requires a running container'
      errorDetail.value = `Container "${found.name}" is in state: ${found.state}`
      loading.value = false
      return
    }
    await loadDirectory('/')
  } catch (e: any) {
    error.value = 'Failed to load container'
    errorDetail.value = e.toString()
    loading.value = false
  }
}

onMounted(() => {
  loadContainer()
})

// ─── File Preview ──────────────────────────────────────────────────────

async function openPreview(file: FileMetadata) {
  if (file.is_dir) {
    await enterDir(file)
    return
  }
  previewFile.value = file
  showPreview.value = true
  previewLoading.value = true
  previewError.value = null
  previewContent.value = ''
  try {
    const content = await readFilePreview(containerId.value, file.full_path, 10000)
    previewContent.value = content
  } catch (e: any) {
    previewError.value = e.toString()
  }
  previewLoading.value = false
}

function closePreview() {
  showPreview.value = false
  previewFile.value = null
  previewContent.value = ''
  previewError.value = null
}

// ─── Download / Upload ─────────────────────────────────────────────────

async function downloadCurrentFile() {
  if (!previewFile.value) return
  try {
    const dest = await save({
      defaultPath: previewFile.value.name,
      filters: [{ name: 'All Files', extensions: ['*'] }],
    })
    if (!dest) return
    await downloadFile(containerId.value, previewFile.value.full_path, dest)
  } catch (e: any) {
    previewError.value = 'Download failed: ' + e.toString()
  }
}

async function handleUpload() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'All Files', extensions: ['*'] }],
    })
    if (!selected) return
    uploadLoading.value = true
    uploadError.value = null
    const fileName = selected.split('/').pop() || selected.split('\\').pop() || 'file'
    const remoteDest = (currentPath.value === '/' ? '/' : currentPath.value + '/') + fileName
    await uploadFile(containerId.value, selected, remoteDest)
    await loadDirectory(currentPath.value) // Refresh
  } catch (e: any) {
    uploadError.value = 'Upload failed: ' + e.toString()
  }
  uploadLoading.value = false
}

// ─── Context Menu ──────────────────────────────────────────────────────

function onContextMenu(e: MouseEvent, file: FileMetadata) {
  e.preventDefault()
  e.stopPropagation()
  const items = [
    {
      id: 'copy-path',
      label: 'Copy Path',
      icon: 'fa-copy',
      action: () => navigator.clipboard.writeText(file.full_path),
    },
    {
      id: 'copy-name',
      label: 'Copy Name',
      icon: 'fa-copy',
      action: () => navigator.clipboard.writeText(file.name),
    },
  ]
  if (!file.is_dir) {
    items.push(
      { id: 'div1', label: '', divider: true } as any,
      {
        id: 'view',
        label: 'View',
        icon: 'fa-eye',
        action: () => openPreview(file),
      },
      {
        id: 'download',
        label: 'Download',
        icon: 'fa-cloud-arrow-down',
        action: async () => {
          try {
            const dest = await save({
              defaultPath: file.name,
              filters: [{ name: 'All Files', extensions: ['*'] }],
            })
            if (dest) await downloadFile(containerId.value, file.full_path, dest)
          } catch (e: any) {
            error.value = 'Download failed: ' + e.toString()
          }
        },
      },
    )
  }
  showContextMenu(e, items)
}

// ─── Helpers for permissions display ──────────────────────────────────

function displayOwner(item: FileMetadata): string {
  return item.owner || '—'
}

function displayGroup(item: FileMetadata): string {
  return item.group || '—'
}

// ─── Clipboard Helpers (for template usage) ──────────────────────────

async function copyPath(path: string) {
  try {
    await navigator.clipboard.writeText(path)
  } catch {
    // fallback
    const ta = document.createElement('textarea')
    ta.value = path
    ta.style.position = 'fixed'
    ta.style.opacity = '0'
    document.body.appendChild(ta)
    ta.select()
    document.execCommand('copy')
    document.body.removeChild(ta)
  }
}
</script>

<template>
  <div class="file-explorer-view">
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <i class="fa-solid fa-house"></i>
      <span style="cursor:pointer" @click="router.push('/')">Home</span>
      <i class="fa-solid fa-chevron-right"></i>
      <span style="cursor:pointer" @click="router.push('/containers')">Containers</span>
      <i class="fa-solid fa-chevron-right"></i>
      <span v-if="container" style="cursor:pointer;color:var(--text-muted)" @click="router.push('/containers')">
        {{ container.name }}
      </span>
      <i class="fa-solid fa-chevron-right"></i>
      <span class="current">Files</span>
    </div>

    <!-- Explorer Header -->
    <div class="explorer-header">
      <div class="explorer-info">
        <i class="fa-solid fa-folder-tree" style="color:var(--accent-cyan);font-size:20px"></i>
        <div>
          <div class="explorer-container-name">
            File Explorer — {{ container?.name || containerId }}
          </div>
          <div class="explorer-container-id" v-if="container">
            ID: {{ container.id?.slice(0, 12) }} · {{ container.image }} · Running
          </div>
        </div>
      </div>
      <div class="explorer-actions">
        <button class="btn btn-secondary" @click="handleUpload" :disabled="uploadLoading">
          <i class="fa-solid fa-cloud-arrow-up"></i> Upload File
        </button>
        <button class="btn btn-ghost" @click="loadDirectory(currentPath)">
          <i class="fa-solid fa-rotate" :class="{ 'fa-spin': loading }"></i>
        </button>
      </div>
    </div>

    <!-- Upload Error -->
    <div v-if="uploadError" class="error-inline">
      <i class="fa-solid fa-circle-exclamation"></i> {{ uploadError }}
      <button class="error-dismiss" @click="uploadError = null">&times;</button>
    </div>

    <!-- Path Bar / Breadcrumb -->
    <div class="path-bar">
      <span class="path-segment" @click="goHome" title="Home">
        <i class="fa-solid fa-house"></i>
      </span>
      <template v-for="(seg, i) in breadcrumbSegments" :key="i">
        <span class="path-sep">/</span>
        <span
          :class="['path-segment', { current: i === breadcrumbSegments.length - 1 }]"
          @click="goToHistory(i)"
        >
          {{ seg.label }}
        </span>
      </template>
    </div>

    <!-- Action Bar -->
    <div class="action-bar">
      <div class="action-bar-left">
        <div class="table-filter">
          <i class="fa-solid fa-magnifying-glass"></i>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Filter files..."
          />
        </div>
        <label class="toggle-label">
          <input type="checkbox" v-model="showHidden" />
          <span class="toggle-slider"></span>
          <span class="toggle-text">Show hidden</span>
        </label>
      </div>
      <div class="action-bar-hint" v-if="!loading">
        <i class="fa-solid fa-circle-info"></i>
        <span class="text-mono">{{ currentPath }}</span>
        <span> · {{ dirCount }} {{ dirCount === 1 ? 'directory' : 'directories' }} · {{ fileCount }} {{ fileCount === 1 ? 'file' : 'files' }}</span>
      </div>
    </div>

    <!-- Loading state (A.6.1) -->
    <SkeletonLoader v-if="loading && files.length === 0 && !error" variant="table-row" :rows="7" />

    <!-- Error state (A.6.3) -->
    <ErrorState
      v-if="error && !loading"
      :message="error"
      :suggestion="error?.includes('running') ? 'Start the container first to browse its files.' : 'Check the container ID and ensure Docker is running.'"
      :detail="errorDetail || undefined"
      icon="fa-solid fa-folder-open"
      @retry="loadContainer"
    />

    <!-- Empty state (A.6.2) -->
    <EmptyState
      v-if="!loading && !error && sortedFiles.length === 0 && !searchQuery"
      icon="fa-solid fa-folder-open"
      title="This directory is empty"
      :description="`No files or directories found at ${currentPath}`"
    />

    <EmptyState
      v-if="!loading && !error && sortedFiles.length === 0 && searchQuery"
      icon="fa-solid fa-magnifying-glass"
      title="No matches"
      :description="`No files match &quot;${searchQuery}&quot;`"
    />

    <!-- Explorer Layout (main view) -->
    <div class="explorer-layout" v-if="!loading && !error && sortedFiles.length > 0">
      <!-- File Tree (left panel) -->
      <div class="file-tree-section">
        <div class="file-tree-header">Files</div>
        <div class="file-tree">
          <div
            v-for="file in sortedFiles"
            :key="file.full_path"
            :class="[
              'file-tree-item',
              { selected: previewFile?.full_path === file.full_path },
            ]"
            @click="file.is_dir ? enterDir(file) : openPreview(file)"
            @dblclick="file.is_dir && enterDir(file)"
            @contextmenu="onContextMenu($event, file)"
          >
            <i
              :class="getFileIconClass(file.name, file.is_dir, file.is_symlink)"
              :style="{
                color: file.is_dir ? 'var(--accent-yellow)' : file.is_symlink ? 'var(--accent-purple)' : 'var(--text-muted)',
              }"
            ></i>
            <span class="file-name-text" :title="file.full_path">{{ file.name }}</span>
            <span v-if="file.is_symlink" class="symlink-badge">@</span>
          </div>
        </div>
      </div>

      <!-- File Content (right panel) -->
      <div class="file-content-section">
        <!-- No preview selected -->
        <div v-if="!showPreview" class="file-content-empty">
          <i class="fa-solid fa-file-circle-plus" style="font-size:48px;opacity:0.2;margin-bottom:12px"></i>
          <p style="color:var(--text-muted);font-size:13px">Select a file to preview</p>
        </div>

        <!-- Preview panel -->
        <template v-if="showPreview && previewFile">
          <div class="file-toolbar">
            <div class="file-toolbar-left">
              <i
                :class="getFileIconClass(previewFile.name, false, false)"
                style="color:var(--accent-cyan);font-size:15px"
              ></i>
              <span class="file-name">{{ previewFile.name }}</span>
              <span class="file-size">{{ formatFileSize(previewFile.size_bytes) }}</span>
              <span class="file-size">· {{ previewFile.permissions }} {{ displayOwner(previewFile) }}:{{ displayGroup(previewFile) }}</span>
            </div>
            <div class="file-toolbar-actions">
              <button class="icon-btn download" title="Download" @click="downloadCurrentFile">
                <i class="fa-solid fa-cloud-arrow-down"></i>
              </button>
              <button class="icon-btn" title="Copy path" @click="copyPath(previewFile.full_path)">
                <i class="fa-solid fa-copy"></i>
              </button>
              <button class="icon-btn" title="Close" @click="closePreview">
                <i class="fa-solid fa-xmark"></i>
              </button>
            </div>
          </div>

          <!-- Loading preview -->
          <div v-if="previewLoading" class="code-preview" style="display:flex;align-items:center;justify-content:center">
            <div style="text-align:center;color:var(--text-muted)">
              <i class="fa-solid fa-spinner fa-spin" style="font-size:24px;margin-bottom:8px"></i>
              <p style="font-size:12px">Loading preview...</p>
            </div>
          </div>

          <!-- Preview error -->
          <ErrorState
            v-else-if="previewError"
            :message="'Could not read file'"
            :detail="previewError"
            :suggestion="'The file may be binary or too large.'"
            @retry="openPreview(previewFile!)"
          />

          <!-- Code preview -->
          <div v-else class="code-preview">
            <div v-for="(line, i) in previewContent.split('\n')" :key="i" class="code-line" v-if="previewContent">
              <span class="code-num">{{ i + 1 }}</span>
              <span class="code-text">{{ line }}</span>
            </div>
            <div v-if="!previewContent" style="padding:20px;text-align:center;color:var(--text-disabled);font-size:12px">
              <i class="fa-solid fa-file-empty" style="font-size:32px;margin-bottom:8px;opacity:0.3"></i>
              <p>File is empty</p>
            </div>
          </div>
        </template>
      </div>
    </div>

    <!-- Explorer Layout (table view fallback when no preview panel on narrow layout) -->
    <div class="table-section" v-if="!loading && !error && sortedFiles.length > 0">
      <div class="col-headers">
        <div class="col-header" style="flex:0 0 28px"></div>
        <div class="col-header" style="flex:1">Name</div>
        <div class="col-header" style="flex:0 0 80px;text-align:right">Size</div>
        <div class="col-header" style="flex:0 0 110px">Permissions</div>
        <div class="col-header" style="flex:0 0 130px">Owner:Group</div>
        <div class="col-header" style="flex:0 0 150px">Modified</div>
      </div>
      <div
        v-for="file in sortedFiles"
        :key="file.full_path"
        :class="['file-row', { selected: previewFile?.full_path === file.full_path }]"
        @click="file.is_dir ? enterDir(file) : openPreview(file)"
        @dblclick="file.is_dir && enterDir(file)"
        @contextmenu="onContextMenu($event, file)"
      >
        <div style="flex:0 0 28px;text-align:center">
          <i
            :class="getFileIconClass(file.name, file.is_dir, file.is_symlink)"
            :style="{
              color: file.is_dir ? 'var(--accent-yellow)' : file.is_symlink ? 'var(--accent-purple)' : 'var(--text-muted)',
              fontSize: '14px',
            }"
          ></i>
        </div>
        <div class="file-name-cell">
          <span class="file-link">{{ file.name }}</span>
          <span v-if="file.is_symlink" class="symlink-badge">@</span>
        </div>
        <div class="file-size-cell">{{ file.is_dir ? '—' : formatFileSize(file.size_bytes) }}</div>
        <div class="file-perms-cell">
          <code class="perm-text">{{ file.permissions }}</code>
        </div>
        <div class="file-owner-cell">
          <span class="owner-text">{{ displayOwner(file) }}:{{ displayGroup(file) }}</span>
        </div>
        <div class="file-time-cell">{{ formatFileTime(file.updated_at) }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ─── File Explorer View ─── */
.file-explorer-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
  height: 100%;
}

/* ─── Explorer Header ─── */
.explorer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 16px 20px;
}

.explorer-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.explorer-container-name {
  font-size: 1.1rem;
  font-weight: 700;
  letter-spacing: -0.01em;
}

.explorer-container-id {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.explorer-actions {
  display: flex;
  gap: 6px;
}

/* ─── Error Inline ─── */
.error-inline {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  border-radius: var(--radius-md);
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.15);
  color: var(--accent-red);
  font-size: 12px;
}

.error-dismiss {
  margin-left: auto;
  background: none;
  border: none;
  color: var(--accent-red);
  cursor: pointer;
  font-size: 16px;
  padding: 0 4px;
  line-height: 1;
}

/* ─── Path Bar ─── */
.path-bar {
  display: flex;
  align-items: center;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 0 4px;
  overflow-x: auto;
  flex-shrink: 0;
}

.path-segment {
  padding: 6px 8px;
  font-size: 12px;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 4px;
  font-family: var(--font-mono);
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 4px;
  transition: all var(--transition-fast);
}

.path-segment:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

.path-segment.current {
  color: var(--accent-cyan);
  font-weight: 500;
}

.path-sep {
  color: var(--text-disabled);
  font-size: 12px;
  margin: 0 1px;
  flex-shrink: 0;
}

/* ─── Action Bar ─── */
.action-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.action-bar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.action-bar-hint {
  font-size: 11px;
  color: var(--text-disabled);
  display: flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

/* ─── Toggle (Show hidden) ─── */
.toggle-label {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 11px;
  color: var(--text-muted);
  user-select: none;
}

.toggle-label input {
  display: none;
}

.toggle-slider {
  width: 32px;
  height: 18px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  position: relative;
  transition: all var(--transition-fast);
}

.toggle-slider::after {
  content: '';
  position: absolute;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--text-disabled);
  top: 2px;
  left: 2px;
  transition: all var(--transition-fast);
}

.toggle-label input:checked + .toggle-slider {
  background: var(--accent-cyan);
  border-color: var(--accent-cyan);
}

.toggle-label input:checked + .toggle-slider::after {
  background: #0a0c10;
  left: 16px;
}

.toggle-text {
  font-size: 11px;
}

/* ─── Explorer Layout (split) ─── */
.explorer-layout {
  display: flex;
  gap: 0;
  min-height: 350px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

/* ─── File Tree (left) ─── */
.file-tree-section {
  width: 280px;
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}

.file-tree-header {
  padding: 8px 12px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg-tertiary);
}

.file-tree {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.file-tree-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: var(--radius-sm);
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
  color: var(--text-muted);
}

.file-tree-item:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

.file-tree-item.selected {
  background: var(--bg-tertiary);
  color: var(--accent-cyan);
}

.file-tree-item i {
  width: 16px;
  text-align: center;
  font-size: 13px;
  flex-shrink: 0;
}

.file-name-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ─── File Content (right) ─── */
.file-content-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.file-content-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

/* ─── File Toolbar ─── */
.file-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-light);
  font-size: 11px;
  color: var(--text-muted);
  flex-shrink: 0;
}

.file-toolbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.file-name {
  font-weight: 600;
  color: var(--text-main);
  font-family: var(--font-mono);
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-size {
  color: var(--text-disabled);
  white-space: nowrap;
}

.file-toolbar-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

/* ─── Icon Buttons ─── */
.icon-btn {
  width: 28px;
  height: 28px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  transition: all var(--transition-fast);
}

.icon-btn:hover {
  background: var(--bg-hover);
  color: var(--text-main);
}

.icon-btn.download:hover {
  color: var(--accent-green);
}

/* ─── Code Preview ─── */
.code-preview {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  font-family: var(--font-mono);
  font-size: 12px;
  line-height: 1.7;
  background: var(--bg-primary);
}

.code-line {
  display: flex;
  gap: 16px;
  padding: 0;
  min-height: 1.4em;
}

.code-num {
  width: 36px;
  text-align: right;
  color: var(--text-disabled);
  font-size: 11px;
  user-select: none;
  flex-shrink: 0;
}

.code-text {
  white-space: pre;
  color: var(--text-main);
  word-break: break-all;
}

/* ─── Table section (fallback) ─── */
.table-section {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.col-headers {
  display: flex;
  align-items: center;
  padding: 8px 20px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
  gap: 12px;
}

.col-header {
  cursor: default;
  display: flex;
  align-items: center;
  gap: 4px;
}

.file-row {
  display: flex;
  align-items: center;
  padding: 8px 20px;
  border-bottom: 1px solid var(--border-light);
  transition: background var(--transition-fast);
  gap: 12px;
  cursor: pointer;
}

.file-row:hover {
  background: var(--bg-hover);
}

.file-row.selected {
  background: var(--bg-tertiary);
}

.file-name-cell {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 4px;
}

.file-link {
  font-size: 13px;
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--text-main);
}

.file-row:hover .file-link {
  color: var(--accent-cyan);
}

.file-size-cell {
  flex: 0 0 80px;
  text-align: right;
  font-size: 12px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.file-perms-cell {
  flex: 0 0 110px;
  font-size: 11px;
}

.perm-text {
  font-family: var(--font-mono);
  color: var(--text-disabled);
  background: var(--bg-tertiary);
  padding: 1px 6px;
  border-radius: 3px;
  font-size: 11px;
}

.file-owner-cell {
  flex: 0 0 130px;
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.owner-text {
  color: var(--text-muted);
}

.file-time-cell {
  flex: 0 0 150px;
  font-size: 11px;
  color: var(--text-muted);
}

/* ─── Symlink Badge ─── */
.symlink-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: rgba(168, 85, 247, 0.15);
  color: var(--accent-purple);
  font-size: 9px;
  font-weight: 700;
  flex-shrink: 0;
}
</style>