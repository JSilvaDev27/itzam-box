<!-- ItzamBox — Restore Snapshot Modal
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, watch } from 'vue'

export interface RestoreVolumeOption {
  name: string
  isExisting: boolean
  hasAttachedContainers?: boolean
  attachedContainers?: string[]
}

const props = defineProps<{
  visible: boolean
  snapshotName: string
  snapshotSize: string
  sourceVolume: string
  volumes: RestoreVolumeOption[]
  progressActive: boolean
}>()

const emit = defineEmits<{
  close: []
  restore: [config: {
    targetVolume: string
    createNew: boolean
    stopContainer: boolean
  }]
}>()

const targetVolume = ref('')
const newVolumeName = ref('')
const actionMode = ref<'stop' | 'cancel'>('stop')

const isCreateNew = computed(() => targetVolume.value === '__new__')

const showOverwriteWarning = computed(() => {
  if (isCreateNew.value) return false
  return true // existing volume always has overwrite warning
})

const showContainerWarning = computed(() => {
  if (isCreateNew.value || !targetVolume.value) return false
  const vol = props.volumes.find(v => v.name === targetVolume.value)
  return vol?.hasAttachedContainers === true
})

const attachedContainerNames = computed(() => {
  if (!targetVolume.value) return ''
  const vol = props.volumes.find(v => v.name === targetVolume.value)
  return vol?.attachedContainers?.join(', ') || ''
})

const isValid = computed(() => {
  if (isCreateNew.value) return newVolumeName.value.trim().length > 0
  return targetVolume.value.length > 0
})

function resetForm() {
  targetVolume.value = ''
  newVolumeName.value = ''
  actionMode.value = 'stop'
}

watch(() => props.visible, (v) => {
  if (v) resetForm()
})

function handleRestore() {
  if (!isValid.value) return
  emit('restore', {
    targetVolume: isCreateNew.value ? newVolumeName.value : targetVolume.value,
    createNew: isCreateNew.value,
    stopContainer: showContainerWarning.value && actionMode.value === 'stop',
  })
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-backdrop" @click.self="emit('close')">
      <div class="modal-content" role="dialog" aria-modal="true" aria-labelledby="restore-snapshot-title">
        <div class="modal-header">
          <h2 id="restore-snapshot-title" class="modal-title">
            <i class="fa-solid fa-rotate-left" style="color:var(--accent-green);margin-right:8px"></i>
            Restore Snapshot
          </h2>
          <button class="drawer__close" @click="emit('close')" aria-label="Close">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>

        <div class="modal-body">
          <div class="form-group">
            <label class="form-label">Snapshot</label>
            <input
              class="form-input"
              type="text"
              :value="snapshotName + ' (' + snapshotSize + ')'"
              readonly
            />
          </div>

          <div class="form-group">
            <label class="form-label">Source Volume</label>
            <input class="form-input" type="text" :value="sourceVolume" readonly />
          </div>

          <div class="form-group">
            <label class="form-label" for="restoreTarget">Target Volume *</label>
            <select
              id="restoreTarget"
              v-model="targetVolume"
              class="form-input"
              aria-required="true"
            >
              <option value="">Select target...</option>
              <option
                v-for="vol in volumes.filter(v => v.isExisting)"
                :key="vol.name"
                :value="vol.name"
              >
                {{ vol.name }} (existing)
              </option>
              <option value="__new__">+ Create new volume...</option>
            </select>
          </div>

          <!-- New volume name input -->
          <div v-if="isCreateNew" class="form-group">
            <label class="form-label" for="newVolumeName">New Volume Name *</label>
            <input
              id="newVolumeName"
              v-model="newVolumeName"
              class="form-input"
              type="text"
              placeholder="my-restored-volume"
            />
          </div>

          <!-- Overwrite warning (existing volume) -->
          <div v-if="showOverwriteWarning && targetVolume && !isCreateNew" class="warning-box">
            <i class="fa-solid fa-triangle-exclamation warning-box__icon"></i>
            <div>
              <strong>⚠️ Volume '{{ targetVolume }}' already exists.</strong><br />
              <span style="font-size:12px;opacity:0.8">
                Restoring will overwrite existing data. This action cannot be undone.
              </span>
            </div>
          </div>

          <!-- Container attached warning -->
          <div v-if="showContainerWarning" class="warning-box" style="margin-top:8px">
            <i class="fa-solid fa-triangle-exclamation warning-box__icon"></i>
            <div>
              <strong>Volume '{{ targetVolume }}' is mounted by container(s): {{ attachedContainerNames }} (running).</strong><br />
              <span style="font-size:12px;opacity:0.8">
                Restore requires the container to be stopped. Existing data will be overwritten.
              </span>
            </div>
          </div>

          <!-- Action buttons for container stop -->
          <div v-if="showContainerWarning" class="form-group" style="margin-top:12px">
            <label class="form-label">Action</label>
            <div style="display:flex;gap:12px">
              <label class="radio-option" :class="{ active: actionMode === 'stop' }">
                <input type="radio" v-model="actionMode" value="stop" />
                <i class="fa-solid fa-stop"></i>
                <div>
                  <strong>Stop & Restore</strong>
                  <span style="font-size:11px;color:var(--text-muted);display:block">Stop container, restore data, restart</span>
                </div>
              </label>
              <label class="radio-option" :class="{ active: actionMode === 'cancel' }">
                <input type="radio" v-model="actionMode" value="cancel" />
                <i class="fa-solid fa-xmark"></i>
                <div>
                  <strong>Cancel</strong>
                  <span style="font-size:11px;color:var(--text-muted);display:block">Do not proceed with restore</span>
                </div>
              </label>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-ghost" @click="emit('close')">Cancel</button>
          <button
            class="btn btn-danger"
            :disabled="!isValid || progressActive || (showContainerWarning && actionMode === 'cancel')"
            @click="handleRestore"
          >
            <i class="fa-solid fa-rotate-left"></i>
            {{ progressActive ? 'Restoring...' : showContainerWarning ? 'Stop & Restore' : 'Restore' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
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
