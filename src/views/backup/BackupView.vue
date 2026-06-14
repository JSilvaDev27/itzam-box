<!-- ItzamBox — Backup & Restore View
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useBackup } from '../../composables/useBackup'
import BackupSummaryCard from '../../components/backup/BackupSummaryCard.vue'
import BackupHistoryTable from '../../components/backup/BackupHistoryTable.vue'
import BackupProgressBar from '../../components/backup/BackupProgressBar.vue'
import CreateSnapshotModal from '../../components/backup/CreateSnapshotModal.vue'
import type { VolumeOption } from '../../components/backup/CreateSnapshotModal.vue'
import RestoreModal from '../../components/backup/RestoreModal.vue'
import type { RestoreVolumeOption } from '../../components/backup/RestoreModal.vue'
import ScheduleJobModal from '../../components/backup/ScheduleJobModal.vue'
import type { ScheduleVolumeOption } from '../../components/backup/ScheduleJobModal.vue'
import ErrorState from '../../components/shared/ErrorState.vue'
import { useNotifications } from '../../composables/useNotifications'

const {
  snapshots,
  jobs,
  activeProgress,
  loading,
  error,
  refreshAll,
  listenBackupProgress,
  stopListeningProgress,
  createSnapshot,
  restoreSnapshot,
  deleteBackup,
  verifyChecksum,
  cancelBackup,
  createJob,
  toggleJob,
  deleteJob,
  formatBackupSize,
  relativeTime,
} = useBackup()

const { success, error: notifyError } = useNotifications()

// ─── Tabs ──────────────────────────────────────────────────────────────

const activeTab = ref<'history' | 'scheduled'>('history')

// ─── Modal visibility ──────────────────────────────────────────────────

const showCreateModal = ref(false)
const showRestoreModal = ref(false)
const showScheduleModal = ref(false)

// ─── Restore modal state ───────────────────────────────────────────────

const restoreSnapshotData = ref<{
  id: number
  name: string
  sizeBytes: number
  sourceVolume: string
} | null>(null)

// ─── Available volumes (mock or from Tauri) ────────────────────────────

const availableVolumes = ref<VolumeOption[]>([
  { name: 'postgres_data', mountpoint: '/var/lib/docker/volumes/postgres_data', size_bytes: 2_100_000_000, hasAttachedContainers: true, attachedContainers: ['postgres'] },
  { name: 'redis_cache', mountpoint: '/var/lib/docker/volumes/redis_cache', size_bytes: 256_000_000, hasAttachedContainers: false },
  { name: 'app_configs', mountpoint: '/var/lib/docker/volumes/app_configs', size_bytes: 45_000_000, hasAttachedContainers: false },
  { name: 'grafana_data', mountpoint: '/var/lib/docker/volumes/grafana_data', size_bytes: 1_200_000_000, hasAttachedContainers: true, attachedContainers: ['grafana'] },
])

const availableScheduleVolumes = computed<ScheduleVolumeOption[]>(() =>
  availableVolumes.value.map(v => ({ name: v.name })),
)

const restoreVolumeOptions = computed<RestoreVolumeOption[]>(() =>
  availableVolumes.value.map(v => ({
    name: v.name,
    isExisting: true,
    hasAttachedContainers: v.hasAttachedContainers,
    attachedContainers: v.attachedContainers,
  })),
)

// ─── Lifecycle ─────────────────────────────────────────────────────────

onMounted(async () => {
  await listenBackupProgress()
  await refreshAll()
})

onUnmounted(() => {
  stopListeningProgress()
})

// ─── Summary computed ──────────────────────────────────────────────────

const computedSummary = computed(() => {
  const completed = snapshots.value.filter(s => s.status === 'completed')
  const totalBytes = completed.reduce((acc, s) => acc + (s.size_bytes || 0), 0)
  const dates = completed.map(s => s.created_at).sort((a, b) => b - a)
  const activeJobs = jobs.value.filter(j => j.enabled)

  return {
    total_snapshots: snapshots.value.length,
    total_size_bytes: totalBytes,
    last_backup_at: dates[0] || null,
    last_backup_name: dates[0] ? completed.find(s => s.created_at === dates[0])?.name || null : null,
    oldest_backup_at: dates[dates.length - 1] || null,
    active_jobs_count: activeJobs.length,
  }
})

// ─── Actions ───────────────────────────────────────────────────────────

async function handleCreateSnapshot(config: {
  volume: string
  destPath: string
  name: string
  stopContainer: boolean
}) {
  showCreateModal.value = false
  try {
    await createSnapshot(config.volume, config.destPath, config.name, config.stopContainer)
    success('Snapshot created', `Backup ${config.name} completed successfully.`)
  } catch (e: any) {
    notifyError('Snapshot failed', e?.toString?.() || 'Unknown error')
  }
}

function openRestoreModal(snap: { id: number; name: string; size_bytes: number; source_volume: string }) {
  restoreSnapshotData.value = {
    id: snap.id,
    name: snap.name,
    sizeBytes: snap.size_bytes,
    sourceVolume: snap.source_volume,
  }
  showRestoreModal.value = true
}

async function handleRestore(config: {
  targetVolume: string
  createNew: boolean
  stopContainer: boolean
}) {
  if (!restoreSnapshotData.value) return
  showRestoreModal.value = false
  try {
    await restoreSnapshot(restoreSnapshotData.value.id, config.targetVolume, config.stopContainer)
    success('Restore completed', `Snapshot restored to volume '${config.targetVolume}'.`)
  } catch (e: any) {
    notifyError('Restore failed', e?.toString?.() || 'Unknown error')
  } finally {
    restoreSnapshotData.value = null
  }
}

async function handleDeleteSnapshot(snapshot: any) {
  if (!confirm(`Delete backup "${snapshot.name}"? This action cannot be undone.`)) return
  try {
    await deleteBackup(snapshot.id)
    success('Backup deleted', `${snapshot.name} has been removed.`)
  } catch (e: any) {
    notifyError('Delete failed', e?.toString?.() || 'Unknown error')
  }
}

async function handleVerifyChecksum(snapshot: any) {
  try {
    const matches = await verifyChecksum(snapshot.id)
    if (matches) {
      success('Checksum verified', `SHA256 matches for ${snapshot.name}.`)
    } else {
      notifyError('Checksum mismatch', `SHA256 does not match for ${snapshot.name}.`)
    }
  } catch (e: any) {
    notifyError('Verification failed', e?.toString?.() || 'Unknown error')
  }
}

async function handleRetrySnapshot(_snapshot: any) {
  // Retry a failed snapshot by opening the create modal pre-filled
  // For now, just show a notification
  notifyError('Not implemented', 'Retry is not yet implemented in this version.')
}

async function handleCancelBackup() {
  if (!activeProgress.value) return
  try {
    await cancelBackup(activeProgress.value.job_id)
    success('Backup cancelled', 'The running backup has been stopped and partial data discarded.')
  } catch (e: any) {
    notifyError('Cancel failed', e?.toString?.() || 'Unknown error')
  }
}

async function handleCreateJob(config: {
  name: string
  frequency: string
  cronExpression: string
  sourceVolumes: string[]
  destinationPath: string
  retentionCount: number
}) {
  showScheduleModal.value = false
  try {
    await createJob(config)
    success('Schedule saved', `Backup job "${config.name}" created successfully.`)
  } catch (e: any) {
    notifyError('Schedule failed', e?.toString?.() || 'Unknown error')
  }
}

async function handleToggleJob(job: any) {
  try {
    await toggleJob(job.id, !job.enabled)
    success(job.enabled ? 'Job paused' : 'Job resumed', `"${job.name}" is now ${job.enabled ? 'paused' : 'active'}.`)
  } catch (e: any) {
    notifyError('Toggle failed', e?.toString?.() || 'Unknown error')
  }
}

async function handleDeleteJob(job: any) {
  if (!confirm(`Delete schedule "${job.name}"? This action cannot be undone.`)) return
  try {
    await deleteJob(job.id)
    success('Schedule deleted', `"${job.name}" has been removed.`)
  } catch (e: any) {
    notifyError('Delete failed', e?.toString?.() || 'Unknown error')
  }
}

function getNextRun(job: any): string {
  if (!job.enabled) return 'Paused — enable to resume'
  if (job.next_run_at) return relativeTime(job.next_run_at)
  return 'Not scheduled'
}

const frequencyLabel: Record<string, string> = {
  hourly: 'Every hour',
  daily: 'Daily at 02:00',
  weekly: 'Weekly on Sunday',
  custom: 'Custom',
}
</script>

<template>
  <div>
    <!-- Breadcrumb -->
    <div class="breadcrumb">
      <i class="fa-solid fa-house"></i> <span>Home</span>
      <i class="fa-solid fa-chevron-right"></i>
      <span>Operations</span>
      <i class="fa-solid fa-chevron-right"></i>
      <span class="current">Backups</span>
    </div>

    <!-- Page Header -->
    <div class="page-header">
      <h1 class="text-h1">
        <i class="fa-solid fa-box-archive" style="color:var(--accent-cyan);margin-right:8px;font-size:1.5rem"></i>
        Backup & Restore
      </h1>
      <div class="page-actions">
        <button class="btn btn-primary" @click="showCreateModal = true">
          <i class="fa-solid fa-camera"></i> Snapshot Now
        </button>
        <button class="btn btn-secondary" @click="showScheduleModal = true">
          <i class="fa-solid fa-clock"></i> Schedule Backup
        </button>
      </div>
    </div>

    <!-- Error state -->
    <ErrorState
      v-if="error && !loading"
      message="Failed to load backup data"
      :suggestion="'Check that the Docker engine is running and try again.'"
      :detail="error"
      icon="fa-solid fa-circle-exclamation"
      @retry="refreshAll"
    />

    <!-- Summary Cards -->
    <BackupSummaryCard :summary="computedSummary" :loading="loading && snapshots.length === 0" />

    <!-- Active Progress -->
    <BackupProgressBar :progress="activeProgress" @cancel="handleCancelBackup" />

    <!-- Tabs -->
    <div class="tabs" role="tablist">
      <button
        class="tabs__item"
        :class="{ active: activeTab === 'history' }"
        role="tab"
        :aria-selected="activeTab === 'history'"
        @click="activeTab = 'history'"
      >
        <i class="fa-solid fa-clock-rotate-left" style="margin-right:4px;font-size:11px"></i>
        History
      </button>
      <button
        class="tabs__item"
        :class="{ active: activeTab === 'scheduled' }"
        role="tab"
        :aria-selected="activeTab === 'scheduled'"
        @click="activeTab = 'scheduled'"
      >
        <i class="fa-solid fa-calendar" style="margin-right:4px;font-size:11px"></i>
        Scheduled
      </button>
    </div>

    <!-- History Tab -->
    <div v-show="activeTab === 'history'" role="tabpanel">
      <BackupHistoryTable
        :snapshots="snapshots"
        :loading="loading && snapshots.length === 0"
        @restore="openRestoreModal"
        @verify="handleVerifyChecksum"
        @delete="handleDeleteSnapshot"
        @retry="handleRetrySnapshot"
      />
    </div>

    <!-- Scheduled Tab -->
    <div v-show="activeTab === 'scheduled'" role="tabpanel">
      <div class="section">
        <div class="section-header">
          <span class="section-title">Scheduled Backup Jobs</span>
          <div class="table-toolbar">
            <button class="btn btn-primary btn-sm" @click="showScheduleModal = true">
              <i class="fa-solid fa-plus"></i> New Schedule
            </button>
          </div>
        </div>

        <!-- Loading skeleton -->
        <template v-if="loading && jobs.length === 0">
          <div v-for="i in 3" :key="'job-skel-' + i" class="data-row skeleton-row">
            <div class="skeleton skeleton-dot"></div>
            <div class="row-info">
              <div class="skeleton skeleton-text-md" style="width:50%"></div>
              <div class="skeleton skeleton-text-xs" style="width:30%;margin-top:4px"></div>
            </div>
            <div class="skeleton skeleton-tag"></div>
            <div class="skeleton skeleton-text-xs" style="width:80px"></div>
          </div>
        </template>

        <!-- Empty -->
        <div v-else-if="jobs.length === 0" class="empty-state" style="padding:40px 20px">
          <div class="empty-state-icon" style="width:56px;height:56px;font-size:24px">
            <i class="fa-solid fa-calendar"></i>
          </div>
          <h3 class="empty-state-title">No scheduled jobs</h3>
          <p class="empty-state-desc">
            Create a scheduled backup job to automate your backup routine.
          </p>
          <button class="btn btn-primary" @click="showScheduleModal = true">
            <i class="fa-solid fa-plus"></i> New Schedule
          </button>
        </div>

        <!-- Job Rows -->
        <div v-for="job in jobs" :key="job.id" class="data-row">
          <div class="row-info">
            <div class="row-name" :style="{ color: job.enabled ? 'var(--text-main)' : 'var(--text-muted)' }">
              {{ job.name }}
            </div>
            <div class="row-meta" style="display:flex;align-items:center;gap:8px;margin-top:4px">
              <span class="schedule-cron">{{ job.cron_expression }}</span>
              <span class="schedule-human">{{ frequencyLabel[job.frequency] || job.frequency }}</span>
            </div>
          </div>
          <div class="row-info" style="flex:0.5">
            <div style="font-size:11px;color:var(--text-muted)">
              Sources: {{ job.source_volumes.join(', ') }}
            </div>
            <div style="font-size:11px;color:var(--text-disabled)">
              Retention: Last {{ job.retention_count }}
            </div>
          </div>
          <div style="text-align:right;min-width:120px">
            <div :style="{ fontSize: '12px', color: job.enabled ? 'var(--accent-green)' : 'var(--text-disabled)' }">
              {{ getNextRun(job) }}
            </div>
          </div>
          <div class="row-actions" style="opacity:1;display:flex;align-items:center;gap:4px">
            <button
              class="action-btn"
              :title="job.enabled ? 'Pause' : 'Resume'"
              :aria-label="job.enabled ? 'Pause job' : 'Resume job'"
              :style="{ color: job.enabled ? 'var(--accent-green)' : 'var(--text-muted)' }"
              @click="handleToggleJob(job)"
            >
              <i :class="job.enabled ? 'fa-solid fa-pause' : 'fa-solid fa-play'"></i>
            </button>
            <button
              class="action-btn"
              title="Delete"
              aria-label="Delete job"
              style="color:var(--accent-red)"
              @click="handleDeleteJob(job)"
            >
              <i class="fa-solid fa-trash-can"></i>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- ─── Modals ─── -->

    <CreateSnapshotModal
      :visible="showCreateModal"
      :volumes="availableVolumes"
      :progress-active="activeProgress !== null"
      @close="showCreateModal = false"
      @create="handleCreateSnapshot"
    />

    <RestoreModal
      :visible="showRestoreModal"
      :snapshot-name="restoreSnapshotData?.name ?? ''"
      :snapshot-size="restoreSnapshotData ? formatBackupSize(restoreSnapshotData.sizeBytes) : ''"
      :source-volume="restoreSnapshotData?.sourceVolume ?? ''"
      :volumes="restoreVolumeOptions"
      :progress-active="activeProgress !== null"
      @close="showRestoreModal = false; restoreSnapshotData = null"
      @restore="handleRestore"
    />

    <ScheduleJobModal
      :visible="showScheduleModal"
      :volumes="availableScheduleVolumes"
      :saving="loading"
      @close="showScheduleModal = false"
      @save="handleCreateJob"
    />
  </div>
</template>

<style scoped>
.breadcrumb {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

.breadcrumb i {
  font-size: 10px;
}

.breadcrumb .current {
  color: var(--text-main);
  font-weight: 500;
}

.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}

.page-actions {
  display: flex;
  gap: 8px;
}

/* Tabs */
.tabs {
  display: flex;
  gap: 0;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  padding: 4px;
  align-items: center;
  margin-bottom: 16px;
}

.tabs__item {
  padding: 8px 20px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-family: var(--font-sans);
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.tabs__item:hover {
  color: var(--text-main);
  background: var(--bg-hover);
}

.tabs__item.active {
  background: var(--bg-secondary);
  color: var(--accent-cyan);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

/* Scheduled row styles */
.schedule-cron {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--accent-cyan);
  background: rgba(0, 229, 255, 0.08);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
  display: inline-block;
}

.schedule-human {
  font-size: 11px;
  color: var(--text-muted);
}

.data-row {
  display: flex;
  align-items: center;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-light);
  gap: 12px;
  transition: background var(--transition-fast);
}

.data-row:hover {
  background: var(--bg-hover);
}

.row-info {
  flex: 1;
  min-width: 0;
}

.row-name {
  font-size: 14px;
  font-weight: 600;
}

.row-meta {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  display: flex;
  align-items: center;
  gap: 8px;
}

.row-actions {
  display: flex;
  gap: 2px;
}

.action-btn {
  width: 28px;
  height: 28px;
  border-radius: 6px;
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

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-main);
}

.skeleton-row {
  pointer-events: none;
}
</style>
