<!-- ItzamBox — Schedule Backup Job Modal
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed, watch } from 'vue'

export interface ScheduleVolumeOption {
  name: string
}

const props = defineProps<{
  visible: boolean
  volumes: ScheduleVolumeOption[]
  saving?: boolean
}>()

const emit = defineEmits<{
  close: []
  save: [config: {
    name: string
    frequency: string
    cronExpression: string
    sourceVolumes: string[]
    destinationPath: string
    retentionCount: number
  }]
}>()

const jobName = ref('')
const frequency = ref('daily')
const customCron = ref('0 2 * * *')
const sourceVolumes = ref<string[]>([])
const destinationPath = ref('')
const retentionCount = ref(7)

const frequencyPresets: Record<string, { cron: string; label: string }> = {
  hourly: { cron: '0 * * * *', label: 'Every hour' },
  daily: { cron: '0 2 * * *', label: 'Daily at 02:00' },
  weekly: { cron: '0 3 * * 0', label: 'Weekly on Sunday at 03:00' },
  custom: { cron: '0 2 * * *', label: 'Custom expression' },
}

const effectiveCron = computed(() => {
  if (frequency.value === 'custom') return customCron.value
  return frequencyPresets[frequency.value]?.cron || '0 2 * * *'
})

const cronHumanReadable = computed(() => {
  if (frequency.value === 'custom') {
    return describeCron(customCron.value)
  }
  return frequencyPresets[frequency.value]?.label || ''
})

const isCustom = computed(() => frequency.value === 'custom')

const isAggressiveCron = computed(() => {
  const cron = effectiveCron.value
  const parts = cron.trim().split(/\s+/)
  if (parts.length < 5) return false
  // Check if it runs every minute (* in minute field)
  if (parts[0] === '*') return true
  // Check if it runs every X minutes (*/X where X < 5)
  if (parts[0].startsWith('*/')) {
    const interval = parseInt(parts[0].slice(2))
    if (!isNaN(interval) && interval < 5) return true
  }
  return false
})

const isValid = computed(() => {
  if (!jobName.value.trim()) return false
  if (!destinationPath.value.trim()) return false
  if (sourceVolumes.value.length === 0) return false
  if (frequency.value === 'custom') {
    if (!customCron.value.trim()) return false
    if (!isValidCron(customCron.value)) return false
  }
  return true
})

function isValidCron(cron: string): boolean {
  const parts = cron.trim().split(/\s+/)
  if (parts.length !== 5) return false
  for (const part of parts) {
    if (!/^(\*|\d+(-\d+)?)(\/\d+)?(,\d+(-\d+)?)*$/.test(part) && part !== '*' && part !== '?') {
      return false
    }
  }
  return true
}

function describeCron(cron: string): string {
  try {
    const parts = cron.trim().split(/\s+/)
    if (parts.length !== 5) return 'Invalid cron expression'

    const [minute, hour, dayOfMonth, month, dayOfWeek] = parts

    if (minute === '0' && hour === '2' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
      return 'Daily at 02:00'
    }
    if (minute === '0' && hour === '3' && dayOfMonth === '*' && month === '*' && dayOfWeek === '0') {
      return 'Weekly on Sunday at 03:00'
    }
    if (minute === '0' && hour === '*' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
      return 'Every hour'
    }
    if (minute === '*' && hour === '*' && dayOfMonth === '*' && month === '*' && dayOfWeek === '*') {
      return 'Every minute (aggressive!)'
    }
    return `Cron: ${cron}`
  } catch {
    return 'Invalid cron expression'
  }
}

function resetForm() {
  jobName.value = ''
  frequency.value = 'daily'
  customCron.value = '0 2 * * *'
  sourceVolumes.value = []
  destinationPath.value = ''
  retentionCount.value = 7
}

watch(() => props.visible, (v) => {
  if (v) resetForm()
})

function handleSave() {
  if (!isValid.value) return
  emit('save', {
    name: jobName.value.trim(),
    frequency: frequency.value,
    cronExpression: effectiveCron.value,
    sourceVolumes: sourceVolumes.value,
    destinationPath: destinationPath.value.trim(),
    retentionCount: retentionCount.value,
  })
}

function toggleVolume(volName: string) {
  const idx = sourceVolumes.value.indexOf(volName)
  if (idx === -1) {
    sourceVolumes.value.push(volName)
  } else {
    sourceVolumes.value.splice(idx, 1)
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-backdrop" @click.self="emit('close')">
      <div class="modal-content" role="dialog" aria-modal="true" aria-labelledby="schedule-job-title">
        <div class="modal-header">
          <h2 id="schedule-job-title" class="modal-title">
            <i class="fa-solid fa-calendar-plus" style="color:var(--accent-cyan);margin-right:8px"></i>
            Schedule Backup Job
          </h2>
          <button class="drawer__close" @click="emit('close')" aria-label="Close">
            <i class="fa-solid fa-xmark"></i>
          </button>
        </div>

        <div class="modal-body">
          <div class="form-group">
            <label class="form-label" for="jobName">Job Name *</label>
            <input
              id="jobName"
              v-model="jobName"
              class="form-input"
              type="text"
              placeholder="Nightly Postgres Backup"
              aria-required="true"
            />
          </div>

          <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px">
            <div class="form-group">
              <label class="form-label" for="jobFreq">Frequency</label>
              <select id="jobFreq" v-model="frequency" class="form-input">
                <option value="hourly">Hourly</option>
                <option value="daily">Daily at 02:00</option>
                <option value="weekly">Weekly (Sunday)</option>
                <option value="custom">Custom (cron)</option>
              </select>
            </div>
            <div class="form-group">
              <label class="form-label" for="jobRetention">Retention</label>
              <select id="jobRetention" v-model.number="retentionCount" class="form-input">
                <option :value="3">Keep last 3</option>
                <option :value="7">Keep last 7</option>
                <option :value="14">Keep last 14</option>
                <option :value="30">Keep last 30</option>
              </select>
            </div>
          </div>

          <!-- Custom cron expression -->
          <div class="form-group">
            <label class="form-label" for="jobCron">Cron Expression</label>
            <div style="display:flex;align-items:center;gap:12px">
              <input
                id="jobCron"
                v-model="customCron"
                class="form-input"
                type="text"
                :style="{ fontFamily: 'var(--font-mono)', width: '160px' }"
                :disabled="!isCustom"
              />
              <span style="font-size:12px;color:var(--text-muted)">
                → {{ cronHumanReadable }}
              </span>
            </div>
            <span v-if="isAggressiveCron" class="form-warning">
              <i class="fa-solid fa-triangle-exclamation"></i>
              This cron expression runs very frequently and may impact performance.
            </span>
          </div>

          <!-- Source volumes -->
          <div class="form-group">
            <label class="form-label">Source Volume(s) *</label>
            <div class="volume-grid">
              <label
                v-for="vol in volumes"
                :key="vol.name"
                class="volume-chip"
                :class="{ selected: sourceVolumes.includes(vol.name) }"
              >
                <input
                  type="checkbox"
                  :checked="sourceVolumes.includes(vol.name)"
                  @change="toggleVolume(vol.name)"
                  style="display:none"
                />
                <i
                  :class="sourceVolumes.includes(vol.name) ? 'fa-solid fa-square-check' : 'fa-solid fa-square'"
                  :style="{ color: sourceVolumes.includes(vol.name) ? 'var(--accent-cyan)' : 'var(--text-disabled)' }"
                ></i>
                <span>{{ vol.name }}</span>
              </label>
              <div v-if="volumes.length === 0" style="font-size:12px;color:var(--text-muted);padding:8px 0">
                No volumes available. Create a volume first.
              </div>
            </div>
          </div>

          <!-- Destination -->
          <div class="form-group">
            <label class="form-label" for="jobDest">Destination *</label>
            <div style="display:flex;gap:8px">
              <input
                id="jobDest"
                v-model="destinationPath"
                class="form-input"
                type="text"
                placeholder="/home/user/backups/"
                style="flex:1"
              />
              <button class="btn btn-secondary btn-sm" aria-label="Browse destination">
                <i class="fa-solid fa-folder-open"></i>
              </button>
            </div>
          </div>
        </div>

        <div class="modal-footer">
          <button class="btn btn-ghost" @click="emit('close')">Cancel</button>
          <button
            class="btn btn-primary"
            :disabled="!isValid || saving"
            @click="handleSave"
          >
            <i class="fa-solid fa-check"></i>
            {{ saving ? 'Saving...' : 'Save Schedule' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.form-warning {
  display: block;
  font-size: 11px;
  color: var(--accent-yellow);
  margin-top: 4px;
}

.form-warning i {
  margin-right: 4px;
}

.volume-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 4px;
}

.volume-chip {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 12px;
  transition: all var(--transition-fast);
}

.volume-chip:hover {
  border-color: var(--accent-cyan);
}

.volume-chip.selected {
  border-color: var(--accent-cyan);
  background: rgba(0, 229, 255, 0.04);
}
</style>
