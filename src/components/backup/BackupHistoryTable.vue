<!-- ItzamBox — Backup History Table
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, computed } from 'vue'
import type { BackupSnapshot } from '../../composables/useBackup'
import { useBackup } from '../../composables/useBackup'

const props = defineProps<{
  snapshots: readonly BackupSnapshot[]
  loading?: boolean
}>()

const emit = defineEmits<{
  restore: [snapshot: BackupSnapshot]
  verify: [snapshot: BackupSnapshot]
  delete: [snapshot: BackupSnapshot]
  retry: [snapshot: BackupSnapshot]
}>()

const { formatBackupSize, relativeTime, formatDuration } = useBackup()

const filterText = ref('')
const filterStatus = ref('all')

const filteredSnapshots = computed(() => {
  let result = props.snapshots
  if (filterText.value) {
    const q = filterText.value.toLowerCase()
    result = result.filter(
      s => s.name.toLowerCase().includes(q) || s.source_volume.toLowerCase().includes(q),
    )
  }
  if (filterStatus.value !== 'all') {
    result = result.filter(s => s.status === filterStatus.value)
  }
  return result
})

const statusBadge = (status: string) => {
  if (status === 'completed') return 'badge badge--success'
  if (status === 'failed') return 'badge badge--failed'
  if (status === 'in_progress') return 'badge badge--progress'
  return 'badge'
}

const statusLabel = (status: string) => {
  if (status === 'completed') return 'Completed'
  if (status === 'failed') return 'Failed'
  if (status === 'in_progress') return 'In Progress'
  return status
}
</script>

<template>
  <div class="table-container">
    <div class="table-toolbar">
      <span class="table-toolbar__title">Backup History</span>
      <div class="table-toolbar__actions">
        <div class="table-filter">
          <i class="fa-solid fa-magnifying-glass"></i>
          <input
            v-model="filterText"
            type="text"
            placeholder="Filter by volume or name..."
            aria-label="Filter backups"
          />
        </div>
        <select
          v-model="filterStatus"
          class="form-group__select"
          style="padding:4px 28px 4px 8px;font-size:11px;min-width:auto"
          aria-label="Filter by status"
        >
          <option value="all">All Status</option>
          <option value="completed">Completed</option>
          <option value="failed">Failed</option>
          <option value="in_progress">In Progress</option>
        </select>
      </div>
    </div>

    <!-- Loading skeleton -->
    <template v-if="loading">
      <div v-for="i in 4" :key="'skel-' + i" class="data-row skeleton-row">
        <div class="skeleton skeleton-text-sm" style="width:40px;min-width:40px"></div>
        <div class="row-info">
          <div class="skeleton skeleton-text-md" style="width:60%"></div>
          <div class="skeleton skeleton-text-xs" style="width:40%;margin-top:4px"></div>
        </div>
        <div class="skeleton skeleton-tag"></div>
        <div class="skeleton skeleton-text-xs" style="width:50px"></div>
        <div class="skeleton skeleton-text-xs" style="width:60px"></div>
      </div>
    </template>

    <!-- Empty state -->
    <div v-else-if="filteredSnapshots.length === 0" class="empty-state" style="padding:40px 20px">
      <div class="empty-state-icon" style="width:56px;height:56px;font-size:24px">
        <i class="fa-solid fa-box-archive"></i>
      </div>
      <h3 class="empty-state-title">No backups found</h3>
      <p class="empty-state-desc">
        {{ filterText || filterStatus !== 'all' ? 'Try adjusting your filter criteria.' : 'Create your first snapshot using the "Snapshot Now" button above.' }}
      </p>
    </div>

    <!-- Table -->
    <table v-else class="table" aria-label="Backup History">
      <thead>
        <tr>
          <th>Snapshot Name</th>
          <th>Source Volume</th>
          <th style="text-align:right">Size</th>
          <th>Status</th>
          <th>Duration</th>
          <th>Created</th>
          <th style="width:120px">Actions</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="snap in filteredSnapshots" :key="snap.id">
          <td>
            <div class="table__name">
              <i
                :class="snap.status === 'failed' ? 'fa-solid fa-file-zipper' : 'fa-solid fa-file-zipper'"
                :style="{ color: snap.status === 'failed' ? 'var(--accent-red)' : 'var(--accent-cyan)', marginRight: '6px', fontSize: '11px' }"
              ></i>
              {{ snap.name }}
            </div>
          </td>
          <td><span class="text-mono">{{ snap.source_volume }}</span></td>
          <td style="text-align:right;font-family:var(--font-mono);font-size:12px">
            <span v-if="snap.status === 'completed'">{{ formatBackupSize(snap.size_bytes) }}</span>
            <span v-else style="color:var(--text-disabled)">—</span>
          </td>
          <td>
            <span :class="statusBadge(snap.status)">{{ statusLabel(snap.status) }}</span>
          </td>
          <td style="font-size:12px;color:var(--text-muted);font-family:var(--font-mono)">
            {{ formatDuration(snap.duration_seconds) }}
          </td>
          <td style="font-size:12px;color:var(--text-muted)">
            {{ relativeTime(snap.created_at) }}
          </td>
          <td>
            <div class="table__actions" style="opacity:1">
              <!-- Completed: restore + verify + delete -->
              <template v-if="snap.status === 'completed'">
                <button
                  class="btn btn-ghost btn-sm btn-icon"
                  title="Restore"
                  aria-label="Restore"
                  @click="emit('restore', snap)"
                >
                  <i class="fa-solid fa-rotate-left"></i>
                </button>
                <button
                  class="btn btn-ghost btn-sm btn-icon"
                  title="Verify checksum"
                  aria-label="Verify checksum"
                  @click="emit('verify', snap)"
                >
                  <i class="fa-solid fa-shield-check"></i>
                </button>
                <button
                  class="btn btn-ghost btn-sm btn-icon"
                  title="Delete"
                  aria-label="Delete"
                  style="color:var(--accent-red)"
                  @click="emit('delete', snap)"
                >
                  <i class="fa-solid fa-trash-can"></i>
                </button>
              </template>
              <!-- Failed: retry + delete -->
              <template v-else-if="snap.status === 'failed'">
                <button
                  class="btn btn-ghost btn-sm btn-icon"
                  title="Retry"
                  aria-label="Retry"
                  @click="emit('retry', snap)"
                >
                  <i class="fa-solid fa-arrows-rotate"></i>
                </button>
                <button
                  class="btn btn-ghost btn-sm btn-icon"
                  title="Delete"
                  aria-label="Delete"
                  style="color:var(--accent-red)"
                  @click="emit('delete', snap)"
                >
                  <i class="fa-solid fa-trash-can"></i>
                </button>
              </template>
              <!-- In progress: no actions -->
              <span v-else style="font-size:11px;color:var(--text-muted)">Running...</span>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.table-container {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
}

.table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-light);
}

.table-toolbar__title {
  font-size: 14px;
  font-weight: 600;
}

.table-toolbar__actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.table-filter {
  display: flex;
  align-items: center;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 6px 10px;
  font-size: 12px;
  color: var(--text-muted);
  gap: 6px;
}

.table-filter input {
  background: none;
  border: none;
  outline: none;
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
  width: 180px;
}

.table-filter input::placeholder {
  color: var(--text-disabled);
}

.form-group__select {
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-main);
  font-size: 12px;
  font-family: var(--font-sans);
  padding: 6px 28px 6px 10px;
  cursor: pointer;
  outline: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' fill='%239ca3af' viewBox='0 0 16 16'%3E%3Cpath d='M8 11L3 6h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 6px center;
}

.form-group__select:focus {
  border-color: var(--accent-cyan);
}

table {
  width: 100%;
  border-collapse: collapse;
}

thead th {
  padding: 10px 20px;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-disabled);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-light);
  text-align: left;
}

tbody tr {
  border-bottom: 1px solid var(--border-light);
  transition: background var(--transition-fast);
}

tbody tr:hover {
  background: var(--bg-hover);
}

tbody td {
  padding: 12px 20px;
  font-size: 13px;
}

.table__name {
  display: flex;
  align-items: center;
  font-weight: 500;
}

.table__mono {
  font-family: var(--font-mono);
  font-size: 12px;
}

.table__actions {
  display: flex;
  gap: 2px;
}

.skeleton-row {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  gap: 12px;
  border-bottom: 1px solid var(--border-light);
}

.badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 600;
}

.badge--success {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.badge--failed {
  background: rgba(239, 68, 68, 0.1);
  color: var(--accent-red);
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.badge--progress {
  background: rgba(0, 229, 255, 0.08);
  color: var(--accent-cyan);
  border: 1px solid rgba(0, 229, 255, 0.15);
  animation: pulse-progress 1.5s infinite;
}

@keyframes pulse-progress {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.6; }
}
</style>
