// ItzamBox — Backup & Restore Tauri Commands Wrapper (Singleton)
// Copyright (C) 2026 SodigTech — GPL-3.0

import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ref, readonly } from 'vue'

// ─── Domain Types (mirrors engine/types.rs) ───────────────────────────────

export interface BackupSnapshot {
  id: number
  job_id: number | null
  name: string
  source_volume: string
  destination_path: string
  size_bytes: number
  sha256: string | null
  status: string // "completed" | "failed" | "in_progress"
  failure_reason: string | null
  duration_seconds: number | null
  created_at: number // unix epoch seconds
}

export interface BackupJob {
  id: number
  name: string
  frequency: string // "hourly" | "daily" | "weekly" | "custom"
  cron_expression: string
  source_volumes: string[]
  destination_path: string
  retention_count: number
  enabled: boolean
  created_at: number
  updated_at: number
  last_run_at: number | null
  next_run_at: number | null
}

export interface BackupProgress {
  job_id: string
  snapshot_name: string
  bytes_processed: number
  bytes_total: number
  elapsed_seconds: number
  percent: number
  status: 'in_progress' | 'completed' | 'failed'
  message: string
}

export interface BackupSummary {
  total_snapshots: number
  total_size_bytes: number
  last_backup_at: number | null
  last_backup_name: string | null
  oldest_backup_at: number | null
  active_jobs_count: number
}

// ─── Module-level Singleton State ─────────────────────────────────────────

const snapshots = ref<BackupSnapshot[]>([])
const jobs = ref<BackupJob[]>([])
const activeProgress = ref<BackupProgress | null>(null)
const summary = ref<BackupSummary | null>(null)
const loading = ref(false)
const error = ref<string | null>(null)

let unlistenProgress: (() => void) | null = null

// ─── Composable ──────────────────────────────────────────────────────────

export function useBackup() {
  // ── Progress Listener ──────────────────────────────────────────────

  async function listenBackupProgress(): Promise<void> {
    if (unlistenProgress) return
    try {
      unlistenProgress = await listen<BackupProgress>('backup-progress', (event) => {
        const p = event.payload
        activeProgress.value = p
        if (p.status === 'completed' || p.status === 'failed') {
          setTimeout(() => {
            if (activeProgress.value?.job_id === p.job_id) {
              activeProgress.value = null
            }
          }, 3000)
        }
      })
    } catch (e: any) {
      console.warn('[Backup] listen failed, proceeding without progress listener:', e?.toString?.() ?? e)
    }
  }

  function stopListeningProgress(): void {
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
    }
  }

  // ── Summary ────────────────────────────────────────────────────────

  async function loadSummary(): Promise<void> {
    try {
      summary.value = await invoke<BackupSummary>('get_backup_summary')
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    }
  }

  // ── Snapshots ──────────────────────────────────────────────────────

  async function loadBackups(): Promise<void> {
    try {
      snapshots.value = await invoke<BackupSnapshot[]>('list_backups')
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    }
  }

  async function createSnapshot(
    volume: string,
    destPath: string,
    name: string,
    stopContainer: boolean,
  ): Promise<BackupSnapshot> {
    try {
      const result = await invoke<BackupSnapshot>('create_backup', {
        volume,
        destPath,
        name,
        stopContainer,
      })
      await loadBackups()
      await loadSummary()
      error.value = null
      return result
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function restoreSnapshot(
    snapshotId: number,
    targetVolume: string,
    stopContainer: boolean,
  ): Promise<void> {
    try {
      await invoke<void>('restore_backup', {
        snapshotId,
        targetVolume,
        stopContainer,
      })
      await loadBackups()
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function deleteBackup(snapshotId: number): Promise<void> {
    try {
      await invoke<void>('delete_backup', { snapshotId })
      snapshots.value = snapshots.value.filter(s => s.id !== snapshotId)
      await loadSummary()
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function verifyChecksum(snapshotId: number): Promise<boolean> {
    try {
      const result = await invoke<boolean>('verify_checksum', { snapshotId })
      error.value = null
      return result
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function cancelBackup(jobId: string): Promise<void> {
    try {
      await invoke<void>('cancel_backup', { jobId })
      activeProgress.value = null
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  // ── Scheduled Jobs ─────────────────────────────────────────────────

  async function loadJobs(): Promise<void> {
    try {
      jobs.value = await invoke<BackupJob[]>('list_backup_jobs')
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    }
  }

  async function createJob(config: {
    name: string
    frequency: string
    cronExpression: string
    sourceVolumes: string[]
    destinationPath: string
    retentionCount: number
  }): Promise<BackupJob> {
    try {
      const result = await invoke<BackupJob>('schedule_backup', {
        name: config.name,
        frequency: config.frequency,
        cronExpression: config.cronExpression,
        sourceVolumes: config.sourceVolumes,
        destinationPath: config.destinationPath,
        retentionCount: config.retentionCount,
      })
      await loadJobs()
      error.value = null
      return result
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function toggleJob(jobId: number, enabled: boolean): Promise<void> {
    try {
      await invoke<void>('toggle_backup_job', { jobId, enabled })
      const idx = jobs.value.findIndex(j => j.id === jobId)
      if (idx !== -1) {
        jobs.value[idx] = { ...jobs.value[idx], enabled }
      }
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  async function deleteJob(jobId: number): Promise<void> {
    try {
      await invoke<void>('delete_backup_job', { jobId })
      jobs.value = jobs.value.filter(j => j.id !== jobId)
      error.value = null
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
      throw e
    }
  }

  // ── Bulk Refresh ───────────────────────────────────────────────────

  async function refreshAll(): Promise<void> {
    loading.value = true
    error.value = null
    try {
      await Promise.allSettled([
        loadBackups(),
        loadJobs(),
        loadSummary(),
      ])
    } catch (e: any) {
      error.value = e?.toString?.() ?? String(e)
    } finally {
      loading.value = false
    }
  }

  // ── Utility ────────────────────────────────────────────────────────

  function formatBackupSize(bytes: number): string {
    if (bytes === 0) return '—'
    if (bytes > 1e9) return (bytes / 1e9).toFixed(1) + ' GB'
    if (bytes > 1e6) return Math.round(bytes / 1e6) + ' MB'
    if (bytes > 1e3) return Math.round(bytes / 1e3) + ' KB'
    return bytes + ' B'
  }

  function relativeTime(unixSeconds: number): string {
    const now = Math.floor(Date.now() / 1000)
    const diff = now - unixSeconds
    if (diff < 0) return 'just now'
    if (diff < 5) return 'just now'
    if (diff < 60) return `${diff}s ago`
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
    if (diff < 2592000) return `${Math.floor(diff / 86400)}d ago`
    return `${Math.floor(diff / 2592000)}mo ago`
  }

  function formatDuration(seconds: number | null): string {
    if (seconds === null || seconds === undefined) return '—'
    if (seconds < 60) return `${seconds}s`
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    return `${h}h ${m}m`
  }

  return {
    // State
    snapshots: readonly(snapshots),
    jobs: readonly(jobs),
    activeProgress: readonly(activeProgress),
    summary: readonly(summary),
    loading: readonly(loading),
    error: readonly(error),

    // Progress
    listenBackupProgress,
    stopListeningProgress,

    // Summary
    loadSummary,

    // Snapshots
    loadBackups,
    createSnapshot,
    restoreSnapshot,
    deleteBackup,
    verifyChecksum,
    cancelBackup,

    // Jobs
    loadJobs,
    createJob,
    toggleJob,
    deleteJob,

    // Bulk
    refreshAll,

    // Utilities
    formatBackupSize,
    relativeTime,
    formatDuration,
  }
}
