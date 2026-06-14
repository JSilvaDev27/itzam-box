-- ItzamBox — Backup History & Job Scheduling Tables
-- Copyright (C) 2026 SodigTech — GPL-3.0
--
-- Migration 005: Adds tables for backup job definitions and execution history.
-- See AD-003: Temp Container + tar over Docker SDK.

-- Backup job definitions (scheduled recurring jobs)
CREATE TABLE IF NOT EXISTS backup_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    frequency TEXT NOT NULL,            -- 'hourly', 'daily', 'weekly', 'custom'
    cron_expression TEXT,               -- NULL unless frequency = 'custom'
    source_volumes TEXT NOT NULL,       -- JSON array: ["vol1", "vol2"]
    destination_path TEXT NOT NULL,
    retention_count INTEGER NOT NULL DEFAULT 7,
    enabled INTEGER NOT NULL DEFAULT 1, -- Boolean: 1=enabled, 0=paused
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Backup execution history (snapshots + job runs)
CREATE TABLE IF NOT EXISTS backup_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    job_id INTEGER,                     -- NULL for manual snapshots
    name TEXT NOT NULL,
    source_volume TEXT NOT NULL,
    destination_path TEXT NOT NULL,
    compressed_size_bytes INTEGER NOT NULL DEFAULT 0,
    original_size_bytes INTEGER NOT NULL DEFAULT 0,
    sha256_checksum TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'InProgress',  -- Success, Failed, InProgress
    failure_reason TEXT,
    duration_seconds INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (job_id) REFERENCES backup_jobs(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_backup_history_date
    ON backup_history(created_at);

CREATE INDEX IF NOT EXISTS idx_backup_history_volume
    ON backup_history(source_volume);

CREATE INDEX IF NOT EXISTS idx_backup_history_status
    ON backup_history(status);

CREATE INDEX IF NOT EXISTS idx_backup_jobs_enabled
    ON backup_jobs(enabled);
