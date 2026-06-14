<!-- ItzamBox — Backup Summary Card
     Copyright (C) 2026 SodigTech — GPL-3.0 -->
<script setup lang="ts">
import type { BackupSummary } from '../../composables/useBackup'
import { useBackup } from '../../composables/useBackup'

const props = defineProps<{
  summary: BackupSummary | null
  loading?: boolean
}>()

const { formatBackupSize, relativeTime } = useBackup()

function safeCount(val: number | null | undefined, fallback = '--'): string {
  if (val === null || val === undefined) return fallback
  return val.toLocaleString()
}

function safeTime(unixSeconds: number | null | undefined): string {
  if (!unixSeconds) return 'Never'
  return relativeTime(unixSeconds)
}
</script>

<template>
  <div class="backup-summary-grid">
    <!-- Total Backups -->
    <div class="backup-summary-card" :class="{ 'skeleton-card': loading }">
      <div class="backup-summary-card__icon backup-summary-card__icon--cyan">
        <i class="fa-solid fa-archive"></i>
      </div>
      <template v-if="loading">
        <div class="skeleton skeleton-text-lg" style="width:60px"></div>
        <div class="skeleton skeleton-text-xs" style="width:80px;margin-top:6px"></div>
        <div class="skeleton skeleton-text-xs" style="width:100px"></div>
      </template>
      <template v-else>
        <div class="backup-summary-card__value backup-summary-card__value--cyan">
          {{ safeCount(summary?.total_snapshots) }}
        </div>
        <div class="backup-summary-card__label">Total Backups</div>
        <div class="backup-summary-card__sub">Across all volumes</div>
      </template>
    </div>

    <!-- Last Backup -->
    <div class="backup-summary-card" :class="{ 'skeleton-card': loading }">
      <div class="backup-summary-card__icon backup-summary-card__icon--green">
        <i class="fa-solid fa-clock"></i>
      </div>
      <template v-if="loading">
        <div class="skeleton skeleton-text-lg" style="width:80px"></div>
        <div class="skeleton skeleton-text-xs" style="width:90px;margin-top:6px"></div>
        <div class="skeleton skeleton-text-xs" style="width:120px"></div>
      </template>
      <template v-else>
        <div class="backup-summary-card__value backup-summary-card__value--main" style="font-size:1.25rem">
          {{ safeTime(summary?.last_backup_at) }}
        </div>
        <div class="backup-summary-card__label">Last Backup</div>
        <div v-if="summary?.last_backup_name" class="backup-summary-card__sub">
          {{ summary.last_backup_name }}
        </div>
        <div v-else class="backup-summary-card__sub">No backups yet</div>
      </template>
    </div>

    <!-- Total Size -->
    <div class="backup-summary-card" :class="{ 'skeleton-card': loading }">
      <div class="backup-summary-card__icon backup-summary-card__icon--purple">
        <i class="fa-solid fa-hard-drive"></i>
      </div>
      <template v-if="loading">
        <div class="skeleton skeleton-text-lg" style="width:70px"></div>
        <div class="skeleton skeleton-text-xs" style="width:80px;margin-top:6px"></div>
        <div class="skeleton skeleton-text-xs" style="width:100px"></div>
      </template>
      <template v-else>
        <div class="backup-summary-card__value backup-summary-card__value--purple">
          {{ summary?.total_size_bytes ? formatBackupSize(summary.total_size_bytes) : '0 B' }}
        </div>
        <div class="backup-summary-card__label">Total Size</div>
        <div class="backup-summary-card__sub">Compressed archives</div>
      </template>
    </div>

    <!-- Scheduled Jobs -->
    <div class="backup-summary-card" :class="{ 'skeleton-card': loading }">
      <div class="backup-summary-card__icon backup-summary-card__icon--yellow">
        <i class="fa-solid fa-calendar-check"></i>
      </div>
      <template v-if="loading">
        <div class="skeleton skeleton-text-lg" style="width:40px"></div>
        <div class="skeleton skeleton-text-xs" style="width:90px;margin-top:6px"></div>
        <div class="skeleton skeleton-text-xs" style="width:100px"></div>
      </template>
      <template v-else>
        <div class="backup-summary-card__value backup-summary-card__value--yellow">
          {{ safeCount(summary?.active_jobs_count) }}
        </div>
        <div class="backup-summary-card__label">Scheduled Jobs</div>
        <div class="backup-summary-card__sub">Active schedules</div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.backup-summary-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.backup-summary-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  transition: border-color var(--transition-fast), transform var(--transition-fast);
}

.backup-summary-card:hover {
  border-color: rgba(0, 229, 255, 0.12);
  transform: translateY(-1px);
}

.backup-summary-card.skeleton-card {
  pointer-events: none;
}

.backup-summary-card.skeleton-card:hover {
  transform: none;
  border-color: var(--border-color);
}

.backup-summary-card__icon {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  margin-bottom: 8px;
}

.backup-summary-card__icon--cyan {
  background: rgba(0, 229, 255, 0.1);
  color: var(--accent-cyan);
}

.backup-summary-card__icon--green {
  background: rgba(16, 185, 129, 0.1);
  color: var(--accent-green);
}

.backup-summary-card__icon--purple {
  background: rgba(168, 85, 247, 0.1);
  color: var(--accent-purple);
}

.backup-summary-card__icon--yellow {
  background: rgba(245, 158, 11, 0.1);
  color: var(--accent-yellow);
}

.backup-summary-card__value {
  font-size: 1.75rem;
  font-weight: 700;
  font-family: var(--font-mono);
  letter-spacing: -0.02em;
  line-height: 1;
}

.backup-summary-card__value--cyan { color: var(--accent-cyan); }
.backup-summary-card__value--purple { color: var(--accent-purple); }
.backup-summary-card__value--yellow { color: var(--accent-yellow); }
.backup-summary-card__value--main { color: var(--text-main); }

.backup-summary-card__label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-top: 4px;
}

.backup-summary-card__sub {
  font-size: 11px;
  color: var(--text-disabled);
}
</style>
