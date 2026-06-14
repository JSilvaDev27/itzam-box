<!-- ItzamBox — Create Snapshot Modal
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useBackup } from '../../composables/useBackup'

export interface VolumeOption {
  name: string
  mountpoint: string
  size_bytes?: number
  hasAttachedContainers?: boolean
  attachedContainers?: string[]
}

const props = defineProps<{
  visible: boolean
  volumes: VolumeOption[]
  progressActive: boolean
}>()

const emit = defineEmits<{
  close: []
  create: [config: {
    volume: string
    destPath: string
    name: string
    stopContainer: boolean
  }]
}>()

const { formatBackupSize } = useBackup()

const selectedVolume = ref('')
const snapshotName = ref('')
const destPath = ref('')
const actionMode = ref<'stop' | 'anyway'>('anyway')

const showWarning = computed(() => {
  if (!selectedVolume.value) return false
  const vol = props.volumes.find(v => v.name === selectedVolume.value)
  return vol?.hasAttachedContainers === true
})

const attachedContainerNames = computed(() => {
  if (!selectedVolume.value) return ''
  const vol = props.volumes.find(v => v.name === selectedVolume.value)
  return vol?.attachedContainers?.join(', ') || ''
})

function resetForm() {
  selectedVolume.value = ''
  snapshotName.value = ''
  destPath.value = ''
  actionMode.value = 'anyway'
}

function generateSnapshotName(vol: string): string {
  const now = new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  const ts = `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}-${pad(now.getMinutes())}-${pad(now.getSeconds())}`
  return `${vol}_${ts}.tar.gz`
}

watch(selectedVolume, (vol) => {
  if (vol) {
    snapshotName.value = generateSnapshotName(vol)
  } else {
    snapshotName.value = ''
  }
})

watch(() => props.visible, (v) => {
  if (v) resetForm()
})

const isValid = computed(() => {
  return selectedVolume.value && snapshotName.value && destPath.value
})

function handleCreate() {
  if (!isValid.value) return
  emit('create', {
    volume: selectedVolume.value,
    destPath: destPath.value,
    name: snapshotName.value,
    stopContainer: actionMode.value === 'stop',
  })
}

function handleBrowse() {
  // In a full implementation, this would use Tauri's dialog API
  // For now, user types the path manually
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-backdrop" @click.self="emit('close')">
      <div class="modal-content" role="dialog" aria-modal="true" aria-labelledby="create-snapshot-title">
        <div class="modal-header">
          <h2 id="create-snapshot-title" class="modal-title">
            <i class="fa-solid fa-camera" style="color:var(--accent-cyan);margin-right:8px"></i>
            Create Snapshot
          </h2>
          <button class="drawer__close" @click="emit('close')" aria-label="Close">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>

        <div class="modal-body">
          <div class="form-group">
            <label class="form-label" for="backupVolume">Source Volume *</label>
            <select
              id="backupVolume"
              v-model="selectedVolume"
              class="form-input"
              aria-required="true"
            >
              <option value="">Select a volume...</option>
              <option
                v-for="vol in volumes"
                :key="vol.name"
                :value="vol.name"
              >
                {{ vol.name }} ({{ vol.size_bytes ? formatBackupSize(vol.size_bytes) : '?' }})
              </option>
            </select>
          </div>

          <div class="form-group">
            <label class="form-label" for="snapshotName">Snapshot Name</label>
            <input
              id="snapshotName"
              v-model="snapshotName"
              class="form-input mono"
              type="text"
            />
            <span class="form-hint">Auto-generated from volume name and timestamp</span>
          </div>

          <div class="form-group">
            <label class="form-label" for="backupDest">Destination Folder *</label>
            <div style="display:flex;gap:8px">
              <input
                id="backupDest"
                v-model="destPath"
                class="form-input"
                type="text"
                placeholder="/home/user/backups/"
                style="flex:1"
              />
              <button class="btn btn-secondary btn-sm" @click="handleBrowse" aria-label="Browse destination">
                <i class="fa-solid fa-folder-open"></i> Browse
              </button>
            </div>
          </div>

          <!-- Warning for attached containers -->
          <div v-if="showWarning" class="warning-box">
            <i class="fa-solid fa-triangle-exclamation warning-box__icon"></i>
            <div>
              <strong>Volume '{{ selectedVolume }}' is attached to running container(s): {{ attachedContainerNames }}.</strong><br />
              <span style="font-size:12px;opacity:0.8">
                Snapshot may be inconsistent if data is being written. Stop the container for a consistent backup.
              </span>
            </div>
          </div>

          <!-- Action mode selection -->
          <div v-if="showWarning" class="form-group" style="margin-top:12px">
            <label class="form-label">Backup Mode</label>
            <div style="display:flex;gap:12px">
              <label class="radio-option" :class="{ active: actionMode === 'stop' }">
                <input type="radio" v-model="actionMode" value="stop" />
                <i class="fa-solid fa-stop"></i>
                <div>
                  <strong>Stop & Snapshot</strong>
                  <span style="font-size:11px;color:var(--text-muted);display:block">Stop container, create backup, restart</span>
                </div>
              </label>
              <label class="radio-option" :class="{ active: actionMode === 'anyway' }">
                <input type="radio" v-model="actionMode" value="anyway" />
                <i class="fa-solid fa-forward"></i>
                <div>
                  <strong>Snapshot Anyway</strong>
                  <span style="font-size:11px;color:var(--text-muted);display:block">Create backup without stopping container</span>
                </div>
              </label>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-ghost" @click="emit('close')">Cancel</button>
          <button
            class="btn btn-primary"
            :disabled="!isValid || progressActive"
            @click="handleCreate"
          >
            <i class="fa-solid fa-camera"></i>
            {{ progressActive ? 'Creating...' : actionMode === 'stop' ? 'Stop & Snapshot' : 'Create Snapshot' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.form-hint {
  display: block;
  font-size: 11px;
  color: var(--text-disabled);
  margin-top: 4px;
}

.warning-box {
  display: flex;
  gap: 10px;
  padding: 12px;
  background: rgba(245, 158, 11, 0.06);
  border: 1px solid rgba(245, 158, 11, 0.15);
  border-radius: var(--radius-md);
  margin-top: 12px;
  font-size: 13px;
}

.warning-box__icon {
  color: var(--accent-yellow);
  font-size: 16px;
  margin-top: 1px;
}

.radio-option {
  flex: 1;
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  background: var(--bg-tertiary);
}

.radio-option:hover {
  border-color: var(--accent-cyan);
}

.radio-option.active {
  border-color: var(--accent-cyan);
  background: rgba(0, 229, 255, 0.04);
}

.radio-option input[type="radio"] {
  display: none;
}

.radio-option i {
  font-size: 18px;
  color: var(--accent-cyan);
  margin-top: 2px;
}
</style>
