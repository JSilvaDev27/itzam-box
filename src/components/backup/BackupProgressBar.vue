<!-- ItzamBox — Backup Progress Bar
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { computed } from 'vue'
import type { BackupProgress } from '../../composables/useBackup'

const props = defineProps<{
  progress: BackupProgress | null
}>()

const emit = defineEmits<{
  cancel: []
}>()

const bytesProcessedFormatted = computed(() => {
  if (!props.progress) return '0 B'
  return formatBytes(props.progress.bytes_processed)
})

const bytesTotalFormatted = computed(() => {
  if (!props.progress) return '0 B'
  return formatBytes(props.progress.bytes_total)
})

const percentDisplay = computed(() => {
  if (!props.progress) return 0
  return Math.min(Math.round(props.progress.percent), 100)
})

const elapsedFormatted = computed(() => {
  if (!props.progress) return '0s'
  const s = props.progress.elapsed_seconds
  if (s < 60) return `${s}s`
  if (s < 3600) return `${Math.floor(s / 60)}m ${s % 60}s`
  return `${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m`
})

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  if (bytes > 1e9) return (bytes / 1e9).toFixed(1) + ' GB'
  if (bytes > 1e6) return Math.round(bytes / 1e6) + ' MB'
  if (bytes > 1e3) return Math.round(bytes / 1e3) + ' KB'
  return bytes + ' B'
}

const statusIcon = computed(() => {
  if (!props.progress) return ''
  if (props.progress.status === 'completed') return 'fa-solid fa-circle-check'
  if (props.progress.status === 'failed') return 'fa-solid fa-circle-xmark'
  return 'fa-solid fa-spinner'
})

const statusColor = computed(() => {
  if (!props.progress) return 'var(--accent-cyan)'
  if (props.progress.status === 'completed') return 'var(--accent-green)'
  if (props.progress.status === 'failed') return 'var(--accent-red)'
  return 'var(--accent-cyan)'
})
</script>

<template>
  <div v-if="progress" class="backup-progress" :class="{ 'backup-progress--completed': progress.status === 'completed', 'backup-progress--failed': progress.status === 'failed' }">
    <div class="backup-progress__header">
      <div class="backup-progress__title">
        <i :class="statusIcon" :style="{ color: statusColor, animation: progress.status === 'in_progress' ? 'spin 0.8s linear infinite' : 'none' }"></i>
        <span>{{ progress.message || progress.snapshot_name }}</span>
      </div>
      <button
        v-if="progress.status === 'in_progress'"
        class="btn btn-danger btn-sm"
        @click="emit('cancel')"
      >
        <i class="fa-solid fa-stop"></i> Cancel
      </button>
    </div>

    <div class="progress-bar-track">
      <div
        class="progress-bar-fill"
        :style="{
          width: percentDisplay + '%',
          background: progress.status === 'completed'
            ? 'var(--accent-green)'
            : progress.status === 'failed'
            ? 'var(--accent-red)'
            : 'linear-gradient(90deg, var(--accent-cyan), #00b8d4)',
        }"
      ></div>
    </div>

    <div class="backup-progress__stats">
      <span>
        Progress: <span class="backup-progress__stat-value">{{ percentDisplay }}%</span>
      </span>
      <span>
        Size: <span class="backup-progress__stat-value">{{ bytesProcessedFormatted }} / {{ bytesTotalFormatted }}</span>
      </span>
      <span>
        Elapsed: <span class="backup-progress__stat-value">{{ elapsedFormatted }}</span>
      </span>
    </div>
  </div>
</template>

<style scoped>
.backup-progress {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 20px;
  margin-bottom: 16px;
  animation: fadeIn 0.3s ease-out;
}

.backup-progress--completed {
  border-color: rgba(16, 185, 129, 0.2);
}

.backup-progress--failed {
  border-color: rgba(239, 68, 68, 0.2);
}

.backup-progress__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.backup-progress__title {
  font-size: 14px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-bar-track {
  width: 100%;
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.3s ease-out;
  min-width: 2%;
}

.backup-progress__stats {
  display: flex;
  gap: 24px;
  margin-top: 12px;
  font-size: 12px;
  color: var(--text-muted);
}

.backup-progress__stat-value {
  font-family: var(--font-mono);
  font-weight: 600;
  color: var(--text-main);
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
