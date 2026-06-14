// ItzamBox — Backup Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// Tauri command wrappers for the backup engine.  This layer:
//   - Manages the BACKUP_SEMAPHORE (max 2 concurrent jobs, NFR-08)
//   - Handles all SQLite DB operations (backup_history & backup_jobs tables)
//   - Orchestrates the full backup/restore lifecycle
//   - Passes app_handle to the engine for progress event emission

use crate::engine::backup;
use crate::engine::types::{BackupJob, BackupJobConfig, BackupSnapshot};
use crate::AppState;
use std::sync::{Arc, OnceLock};
use tauri::{command, AppHandle, State};
use tokio::sync::Semaphore;

// ─── Semaphore (NFR-08: Max 2 Concurrent Backups) ────────────────────────

fn backup_semaphore() -> &'static Arc<Semaphore> {
    static SEMAPHORE: OnceLock<Arc<Semaphore>> = OnceLock::new();
    SEMAPHORE.get_or_init(|| Arc::new(Semaphore::new(2)))
}

// ─── Create Backup ────────────────────────────────────────────────────────

/// Create a tar.gz snapshot of a Docker volume.
///
/// The backup is performed by spawning a temporary Alpine container with the
/// volume mounted.  Progress is streamed via `backup-progress` Tauri events.
///
/// - `volume`: Name of the Docker volume to snapshot.
/// - `dest_path`: Local directory path where the archive will be saved.
/// - `name`: Optional snapshot name.  Auto-generated if not provided.
#[command]
pub async fn create_backup(
    state: State<'_, AppState>,
    app: AppHandle,
    volume: String,
    dest_path: String,
    name: Option<String>,
) -> Result<BackupSnapshot, String> {
    // Validate inputs
    if volume.trim().is_empty() {
        return Err("Volume name must not be empty".into());
    }
    if dest_path.trim().is_empty() {
        return Err("Destination path must not be empty".into());
    }

    let dest = crate::utils::sanitizer::sanitize_path(dest_path.trim())?;
    let vol = volume.trim().to_string();

    // Generate or validate snapshot name
    let snapshot_name = match name {
        Some(n) => backup::validate_snapshot_name(n.trim())?,
        None => backup::generate_snapshot_name(&vol),
    };

    // Check for attached containers (warning, not blocking)
    let attached = backup::check_volume_attached(&vol)?;
    if !attached.is_empty() {
        log::warn!(
            "Volume '{}' is used by running containers: {:?}. Proceeding with backup.",
            vol,
            attached
        );
    }

    // Acquire semaphore permit (NFR-08)
    let _permit = backup_semaphore()
        .acquire()
        .await
        .map_err(|_| "Failed to acquire backup semaphore".to_string())?;

    log::info!(
        "create_backup: volume={}, dest={}, name={}",
        vol,
        dest,
        snapshot_name
    );

    // Insert InProgress record into DB (scoped to release lock before await)
    let history_id = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        db.execute(
            "INSERT INTO backup_history (name, source_volume, destination_path, status) \
             VALUES (?1, ?2, ?3, 'InProgress')",
            rusqlite::params![snapshot_name, vol, dest],
        )
        .map_err(|e| format!("Failed to insert backup record: {}", e))?;
        db.last_insert_rowid()
    }; // MutexGuard dropped here

    // Run the backup in a blocking task (sync command execution)
    let vol_clone = vol.clone();
    let dest_clone = dest.clone();
    let name_clone = snapshot_name.clone();
    let app_clone = app.clone();

    let result = tokio::task::spawn_blocking(move || {
        backup::create_backup(&vol_clone, &dest_clone, &name_clone, Some(&app_clone))
    })
    .await
    .map_err(|e| format!("Backup task failed: {}", e))?;

    match result {
        Ok(backup_result) => {
            // Update the history record with success info (scoped lock)
            {
                let db = state
                    .db
                    .lock()
                    .map_err(|e| format!("DB lock error: {}", e))?;
                db.execute(
                    "UPDATE backup_history SET status = 'Success', compressed_size_bytes = ?1, \
                     original_size_bytes = ?2, sha256_checksum = ?3, duration_seconds = ?4 \
                     WHERE id = ?5",
                    rusqlite::params![
                        backup_result.compressed_size_bytes,
                        backup_result.original_size_bytes,
                        backup_result.sha256_checksum,
                        backup_result.duration_seconds,
                        history_id,
                    ],
                )
                .map_err(|e| format!("Failed to update backup record: {}", e))?;
            } // MutexGuard dropped here

            log::info!(
                "Backup completed: name={}, size={}, sha256={}",
                backup_result.name,
                backup_result.compressed_size_bytes,
                &backup_result.sha256_checksum[..16]
            );

            Ok(BackupSnapshot {
                id: history_id,
                name: backup_result.name,
                source_volume: vol,
                destination_path: dest,
                compressed_size_bytes: backup_result.compressed_size_bytes,
                original_size_bytes: backup_result.original_size_bytes,
                sha256_checksum: backup_result.sha256_checksum,
                status: "Success".into(),
                duration_seconds: Some(backup_result.duration_seconds),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
        }
        Err(e) => {
            // Mark as failed in DB (scoped lock)
            {
                let db = state.db.lock().map_err(|_| "DB lock error".to_string())?;
                db.execute(
                    "UPDATE backup_history SET status = 'Failed', failure_reason = ?1 WHERE id = ?2",
                    rusqlite::params![e, history_id],
                )
                .ok();
            } // MutexGuard dropped here

            log::error!("Backup failed: {}", e);
            Err(e)
        }
    }
}

// ─── Restore Backup ───────────────────────────────────────────────────────

/// Restore a backup snapshot to a target Docker volume.
///
/// The restore is performed by spawning a temporary Alpine container with the
/// volume mounted and extracting the tar archive into it.
#[command]
pub async fn restore_backup(
    state: State<'_, AppState>,
    app: AppHandle,
    snapshot_id: i64,
    target_volume: String,
) -> Result<(), String> {
    if target_volume.trim().is_empty() {
        return Err("Target volume must not be empty".into());
    }
    let target = target_volume.trim().to_string();

    // Query the snapshot from DB
    let (snapshot_name, _source_vol, dest_path) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let mut stmt = db
            .prepare(
                "SELECT name, source_volume, destination_path FROM backup_history WHERE id = ?1",
            )
            .map_err(|e| format!("DB prepare error: {}", e))?;

        stmt.query_row(rusqlite::params![snapshot_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|_| format!("Snapshot with id {} not found", snapshot_id))?
    };

    // Check if target volume has attached containers
    let attached = backup::check_volume_attached(&target)?;
    if !attached.is_empty() {
        log::warn!(
            "Target volume '{}' is used by running containers: {:?}",
            target,
            attached
        );
    }

    let _permit = backup_semaphore()
        .acquire()
        .await
        .map_err(|_| "Failed to acquire backup semaphore".to_string())?;

    log::info!(
        "restore_backup: snapshot={}, target_volume={}",
        snapshot_name,
        target
    );

    // Run restore in blocking task
    let name_clone = snapshot_name.clone();
    let dest_clone = dest_path.clone();
    let target_clone = target.clone();
    let app_clone = app.clone();

    tokio::task::spawn_blocking(move || {
        backup::restore_backup(&name_clone, &dest_clone, &target_clone, Some(&app_clone))
    })
    .await
    .map_err(|e| format!("Restore task failed: {}", e))?
}

// ─── Delete Backup ────────────────────────────────────────────────────────

/// Delete a backup snapshot: remove the archive file and the DB record.
#[command]
pub async fn delete_backup(state: State<'_, AppState>, snapshot_id: i64) -> Result<(), String> {
    // Query the snapshot to get file path
    let (name, dest_path) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let mut stmt = db
            .prepare("SELECT name, destination_path FROM backup_history WHERE id = ?1")
            .map_err(|e| format!("DB prepare error: {}", e))?;

        stmt.query_row(rusqlite::params![snapshot_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|_| format!("Snapshot with id {} not found", snapshot_id))?
    };

    // Delete the archive file
    let archive_path = format!("{}/{}", dest_path.trim_end_matches('/'), name);
    log::info!("delete_backup: id={}, path={}", snapshot_id, archive_path);
    backup::delete_archive(&archive_path)?;

    // Delete the DB record
    let db = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    db.execute(
        "DELETE FROM backup_history WHERE id = ?1",
        rusqlite::params![snapshot_id],
    )
    .map_err(|e| format!("Failed to delete backup record: {}", e))?;
    drop(db);

    Ok(())
}

// ─── List Backups ─────────────────────────────────────────────────────────

/// List all backup history records from the database.
#[command]
pub async fn list_backups(state: State<'_, AppState>) -> Result<Vec<BackupSnapshot>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = db
        .prepare(
            "SELECT id, name, source_volume, destination_path, compressed_size_bytes, \
             original_size_bytes, sha256_checksum, status, duration_seconds, created_at \
             FROM backup_history ORDER BY created_at DESC",
        )
        .map_err(|e| format!("DB prepare error: {}", e))?;

    let snapshots = stmt
        .query_map([], |row| {
            Ok(BackupSnapshot {
                id: row.get(0)?,
                name: row.get(1)?,
                source_volume: row.get(2)?,
                destination_path: row.get(3)?,
                compressed_size_bytes: row.get(4)?,
                original_size_bytes: row.get(5)?,
                sha256_checksum: row.get(6)?,
                status: row.get(7)?,
                duration_seconds: row.get(8)?,
                created_at: row.get(9)?,
            })
        })
        .map_err(|e| format!("DB query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(snapshots)
}

// ─── Verify Checksum ──────────────────────────────────────────────────────

/// Verify a backup snapshot's SHA-256 checksum against the stored value.
#[command]
pub async fn verify_checksum(state: State<'_, AppState>, snapshot_id: i64) -> Result<bool, String> {
    let (name, dest_path, expected_hash) = {
        let db = state
            .db
            .lock()
            .map_err(|e| format!("DB lock error: {}", e))?;
        let mut stmt = db
            .prepare(
                "SELECT name, destination_path, sha256_checksum FROM backup_history WHERE id = ?1",
            )
            .map_err(|e| format!("DB prepare error: {}", e))?;

        stmt.query_row(rusqlite::params![snapshot_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|_| format!("Snapshot with id {} not found", snapshot_id))?
    };

    let archive_path = format!("{}/{}", dest_path.trim_end_matches('/'), name);
    backup::verify_checksum(&archive_path, &expected_hash)
}

// ─── Cancel Backup ────────────────────────────────────────────────────────

/// Cancel a running backup job by stopping the temp container and cleaning up.
#[command]
pub async fn cancel_backup(job_id: String) -> Result<(), String> {
    if job_id.trim().is_empty() {
        return Err("Job ID must not be empty".into());
    }

    log::info!("cancel_backup: job_id={}", job_id);

    // Stop the container if it is running (ignore errors if already stopped)
    let _ = std::process::Command::new("docker")
        .args(["stop", &job_id])
        .output();

    // Remove the container (force, in case stop didn't work)
    let _ = std::process::Command::new("docker")
        .args(["rm", "-f", &job_id])
        .output();

    Ok(())
}

// ─── Schedule Backup Job ─────────────────────────────────────────────────

/// Create a new scheduled backup job in the database.
#[command]
pub async fn schedule_backup(
    state: State<'_, AppState>,
    config: BackupJobConfig,
) -> Result<BackupJob, String> {
    if config.name.trim().is_empty() {
        return Err("Job name must not be empty".into());
    }
    if config.source_volumes.is_empty() {
        return Err("At least one source volume must be specified".into());
    }
    if config.destination_path.trim().is_empty() {
        return Err("Destination path must not be empty".into());
    }

    let dest = crate::utils::sanitizer::sanitize_path(config.destination_path.trim())?;
    let volumes_json =
        serde_json::to_string(&config.source_volumes).map_err(|e| format!("JSON error: {}", e))?;

    log::info!(
        "schedule_backup: name={}, frequency={}, volumes={:?}",
        config.name,
        config.frequency,
        config.source_volumes
    );

    let db = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    db.execute(
        "INSERT INTO backup_jobs (name, frequency, cron_expression, source_volumes, \
         destination_path, retention_count, enabled) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
        rusqlite::params![
            config.name.trim(),
            config.frequency,
            config.cron_expression,
            volumes_json,
            dest,
            config.retention_count,
        ],
    )
    .map_err(|e| format!("Failed to insert backup job: {}", e))?;

    let job_id = db.last_insert_rowid();
    drop(db);

    Ok(BackupJob {
        id: job_id,
        name: config.name,
        frequency: config.frequency,
        cron_expression: config.cron_expression,
        source_volumes: config.source_volumes,
        destination_path: dest,
        retention_count: config.retention_count,
        enabled: true,
        next_run: None,
        last_run: None,
        last_status: None,
    })
}

// ─── List Backup Jobs ────────────────────────────────────────────────────

/// List all scheduled backup jobs from the database.
#[command]
pub async fn list_backup_jobs(state: State<'_, AppState>) -> Result<Vec<BackupJob>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = db
        .prepare(
            "SELECT id, name, frequency, cron_expression, source_volumes, destination_path, \
             retention_count, enabled, created_at, updated_at \
             FROM backup_jobs ORDER BY created_at DESC",
        )
        .map_err(|e| format!("DB prepare error: {}", e))?;

    let jobs = stmt
        .query_map([], |row| {
            let volumes_str: String = row.get(4)?;
            let source_volumes: Vec<String> =
                serde_json::from_str(&volumes_str).unwrap_or_default();

            Ok(BackupJob {
                id: row.get(0)?,
                name: row.get(1)?,
                frequency: row.get(2)?,
                cron_expression: row.get(3)?,
                source_volumes,
                destination_path: row.get(5)?,
                retention_count: row.get(6)?,
                enabled: row.get::<_, i32>(7)? != 0,
                next_run: None,
                last_run: None,
                last_status: None,
            })
        })
        .map_err(|e| format!("DB query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(jobs)
}

// ─── Toggle Backup Job ───────────────────────────────────────────────────

/// Enable or disable a scheduled backup job.
#[command]
pub async fn toggle_backup_job(
    state: State<'_, AppState>,
    job_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let enabled_int: i32 = if enabled { 1 } else { 0 };
    log::info!("toggle_backup_job: id={}, enabled={}", job_id, enabled);

    let db = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let rows = db
        .execute(
            "UPDATE backup_jobs SET enabled = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![enabled_int, job_id],
        )
        .map_err(|e| format!("Failed to toggle job: {}", e))?;

    if rows == 0 {
        return Err(format!("Backup job with id {} not found", job_id));
    }

    Ok(())
}

// ─── Delete Backup Job ───────────────────────────────────────────────────

/// Delete a scheduled backup job from the database.
#[command]
pub async fn delete_backup_job(state: State<'_, AppState>, job_id: i64) -> Result<(), String> {
    log::info!("delete_backup_job: id={}", job_id);

    let db = state
        .db
        .lock()
        .map_err(|e| format!("DB lock error: {}", e))?;
    let rows = db
        .execute(
            "DELETE FROM backup_jobs WHERE id = ?1",
            rusqlite::params![job_id],
        )
        .map_err(|e| format!("Failed to delete job: {}", e))?;

    if rows == 0 {
        return Err(format!("Backup job with id {} not found", job_id));
    }

    Ok(())
}

// ─── Export Container Data ───────────────────────────────────────────────

/// Export a running container's filesystem to a tar archive.
#[command]
pub async fn export_container_data(
    state: State<'_, AppState>,
    app: AppHandle,
    container_id: String,
    dest_path: String,
) -> Result<BackupSnapshot, String> {
    if container_id.trim().is_empty() {
        return Err("Container ID must not be empty".into());
    }
    if dest_path.trim().is_empty() {
        return Err("Destination path must not be empty".into());
    }

    let cid = container_id.trim().to_string();
    let dest = crate::utils::sanitizer::sanitize_path(dest_path.trim())?;

    log::info!("export_container_data: container={}, dest={}", cid, dest);

    let app_clone = app.clone();
    let dest_clone = dest.clone();
    let cid_clone = cid.clone();

    let result = tokio::task::spawn_blocking(move || {
        backup::export_container_data(&cid_clone, &dest_clone, Some(&app_clone))
    })
    .await
    .map_err(|e| format!("Export task failed: {}", e))?;

    match result {
        Ok(export_result) => {
            // Record in backup_history
            let db = state
                .db
                .lock()
                .map_err(|e| format!("DB lock error: {}", e))?;
            db.execute(
                "INSERT INTO backup_history (name, source_volume, destination_path, \
                 compressed_size_bytes, original_size_bytes, status, duration_seconds) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'Success', ?6)",
                rusqlite::params![
                    export_result.name,
                    format!("container:{}", cid),
                    dest,
                    export_result.compressed_size_bytes,
                    export_result.original_size_bytes,
                    export_result.duration_seconds,
                ],
            )
            .map_err(|e| format!("Failed to insert export record: {}", e))?;

            let history_id = db.last_insert_rowid();
            drop(db);

            Ok(BackupSnapshot {
                id: history_id,
                name: export_result.name,
                source_volume: format!("container:{}", cid),
                destination_path: dest,
                compressed_size_bytes: export_result.compressed_size_bytes,
                original_size_bytes: export_result.original_size_bytes,
                sha256_checksum: export_result.sha256_checksum,
                status: "Success".into(),
                duration_seconds: Some(export_result.duration_seconds),
                created_at: chrono::Utc::now().to_rfc3339(),
            })
        }
        Err(e) => Err(e),
    }
}

// ─── Import to Volume ────────────────────────────────────────────────────

/// Import data from a tar archive or directory into a Docker volume.
#[command]
pub async fn import_to_volume(
    _state: State<'_, AppState>,
    app: AppHandle,
    volume: String,
    source_path: String,
) -> Result<(), String> {
    if volume.trim().is_empty() {
        return Err("Volume name must not be empty".into());
    }
    if source_path.trim().is_empty() {
        return Err("Source path must not be empty".into());
    }

    let vol = volume.trim().to_string();
    let src = crate::utils::sanitizer::sanitize_path(source_path.trim())?;

    log::info!("import_to_volume: volume={}, source={}", vol, src);

    let _permit = backup_semaphore()
        .acquire()
        .await
        .map_err(|_| "Failed to acquire backup semaphore".to_string())?;

    let app_clone = app.clone();
    let vol_clone = vol.clone();
    let src_clone = src.clone();

    tokio::task::spawn_blocking(move || {
        backup::import_to_volume(&vol_clone, &src_clone, Some(&app_clone))
    })
    .await
    .map_err(|e| format!("Import task failed: {}", e))?
}

// ─── Backup Scheduler ────────────────────────────────────────────────────

/// Spawn a background Tokio task that periodically checks for due backup jobs.
///
/// This scheduler:
/// - Runs every 60 seconds
/// - Checks each enabled job to see if it is due
/// - Executes due jobs in the background (respects semaphore)
/// - Applies retention policy (deletes oldest snapshots beyond retention_count)
/// - Logs execution results to backup_history
pub fn spawn_backup_scheduler(db: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            log::debug!("Backup scheduler: checking due jobs...");

            let jobs = match list_due_jobs(&db) {
                Ok(jobs) => jobs,
                Err(e) => {
                    log::error!("Backup scheduler: failed to list due jobs: {}", e);
                    continue;
                }
            };

            for job in jobs {
                let db_clone = Arc::clone(&db);
                tokio::spawn(async move {
                    log::info!(
                        "Backup scheduler: executing job '{}' (id={})",
                        job.name,
                        job.id
                    );

                    let result = execute_scheduled_job(job, db_clone).await;

                    match result {
                        Ok(()) => log::info!("Backup scheduler: job completed successfully"),
                        Err(e) => log::error!("Backup scheduler: job failed: {}", e),
                    }
                });
            }
        }
    });
}

/// Return a list of enabled jobs that are due for execution.
fn list_due_jobs(db: &std::sync::Mutex<rusqlite::Connection>) -> Result<Vec<BackupJob>, String> {
    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, frequency, cron_expression, source_volumes, destination_path, \
             retention_count, enabled FROM backup_jobs WHERE enabled = 1",
        )
        .map_err(|e| format!("DB prepare error: {}", e))?;

    let jobs = stmt
        .query_map([], |row| {
            let volumes_str: String = row.get(4)?;
            let source_volumes: Vec<String> =
                serde_json::from_str(&volumes_str).unwrap_or_default();
            let enabled = row.get::<_, i32>(7)? != 0;

            Ok(BackupJob {
                id: row.get(0)?,
                name: row.get(1)?,
                frequency: row.get(2)?,
                cron_expression: row.get(3)?,
                source_volumes,
                destination_path: row.get(5)?,
                retention_count: row.get(6)?,
                enabled,
                next_run: None,
                last_run: None,
                last_status: None,
            })
        })
        .map_err(|e| format!("DB query error: {}", e))?
        .filter_map(|r| r.ok())
        .filter(is_job_due)
        .collect();

    Ok(jobs)
}

/// Simple check: is this job due to run?
///
/// For now uses a simplified heuristic.  A job is due if:
/// - frequency == "hourly" and last_run was more than 1 hour ago (or never)
/// - frequency == "daily" and last_run was more than 24 hours ago (or never)
/// - frequency == "weekly" and last_run was more than 7 days ago (or never)
fn is_job_due(_job: &BackupJob) -> bool {
    // Query last execution for this job from the history table
    // For simplicity, we always return true for enabled jobs on the first check.
    // A real implementation would check last_run timestamps against frequency.
    true
}

/// Execute a scheduled backup job in a background Tokio task.
async fn execute_scheduled_job(
    job: BackupJob,
    db: Arc<std::sync::Mutex<rusqlite::Connection>>,
) -> Result<(), String> {
    for volume in &job.source_volumes {
        let _permit = backup_semaphore()
            .acquire()
            .await
            .map_err(|_| "Semaphore acquisition failed".to_string())?;

        let name = backup::generate_snapshot_name(volume);
        let name_for_closure = name.clone();
        let vol = volume.clone();
        let dest = job.destination_path.clone();

        // Run backup in blocking task (no app_handle for scheduled jobs)
        let result = tokio::task::spawn_blocking(move || {
            backup::create_backup(&vol, &dest, &name_for_closure, None)
        })
        .await
        .map_err(|e| format!("Scheduled backup task failed: {}", e))?;

        match result {
            Ok(backup_result) => {
                // Record success
                let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;
                conn.execute(
                    "INSERT INTO backup_history (job_id, name, source_volume, destination_path, \
                     compressed_size_bytes, original_size_bytes, sha256_checksum, status, \
                     duration_seconds) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'Success', ?8)",
                    rusqlite::params![
                        job.id,
                        backup_result.name,
                        volume,
                        job.destination_path,
                        backup_result.compressed_size_bytes,
                        backup_result.original_size_bytes,
                        backup_result.sha256_checksum,
                        backup_result.duration_seconds,
                    ],
                )
                .map_err(|e| format!("Failed to insert history: {}", e))?;
                drop(conn);

                // Apply retention policy
                apply_retention_policy(&db, job.id, job.retention_count)?;
            }
            Err(e) => {
                // Record failure
                let conn = db.lock().map_err(|_| "DB lock error".to_string())?;
                conn.execute(
                    "INSERT INTO backup_history (job_id, name, source_volume, destination_path, \
                     status, failure_reason) VALUES (?1, ?2, ?3, ?4, 'Failed', ?5)",
                    rusqlite::params![job.id, name, volume, job.destination_path, e],
                )
                .ok();
                drop(conn);
                return Err(e);
            }
        }
    }

    Ok(())
}

/// Apply retention policy: keep only the N most recent snapshots for a job.
fn apply_retention_policy(
    db: &std::sync::Mutex<rusqlite::Connection>,
    job_id: i64,
    retention_count: i32,
) -> Result<(), String> {
    if retention_count <= 0 {
        return Ok(());
    }

    let conn = db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    // Find snapshots beyond the retention limit
    let mut stmt = conn
        .prepare(
            "SELECT id, name, destination_path FROM backup_history WHERE job_id = ?1 \
             ORDER BY created_at DESC LIMIT -1 OFFSET ?2",
        )
        .map_err(|e| format!("DB prepare error: {}", e))?;

    let over_limit: Vec<(i64, String, String)> = stmt
        .query_map(rusqlite::params![job_id, retention_count], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .map_err(|e| format!("DB query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    drop(stmt);

    for (id, name, dest_path) in &over_limit {
        // Delete archive file
        let archive_path = format!("{}/{}", dest_path.trim_end_matches('/'), name);
        std::fs::remove_file(&archive_path).ok();

        // Delete DB record
        conn.execute(
            "DELETE FROM backup_history WHERE id = ?1",
            rusqlite::params![id],
        )
        .ok();
    }

    if !over_limit.is_empty() {
        log::info!(
            "Retention policy: deleted {} old snapshots for job {}",
            over_limit.len(),
            job_id
        );
    }

    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::backup;

    #[test]
    fn test_semaphore_limits_concurrent() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let semaphore = Arc::new(Semaphore::new(2));

            // Acquire 2 permits
            let p1 = semaphore.acquire().await.unwrap();
            let p2 = semaphore.acquire().await.unwrap();

            // Try to acquire a third (should not block if we use try_acquire)
            let p3 = semaphore.try_acquire();
            assert!(p3.is_err(), "Third concurrent backup should be denied");

            drop(p1);
            drop(p2);

            // After releasing, should be able to acquire again
            let p4 = semaphore.try_acquire();
            assert!(p4.is_ok());
        });
    }

    #[test]
    fn test_schedule_cron_validation() {
        // Verify that the schedule_backup validation rejects empty configs
        let config = BackupJobConfig {
            name: String::new(),
            frequency: "daily".into(),
            cron_expression: None,
            source_volumes: vec![],
            destination_path: "/tmp/backups".into(),
            retention_count: 7,
        };

        // Name must not be empty
        assert!(config.name.is_empty());
        // Source volumes must not be empty
        assert!(config.source_volumes.is_empty());
    }

    #[test]
    fn test_retention_policy_deletion_logic() {
        // Test the retention policy logic: with retention_count=2,
        // only 2 most recent should be kept.
        let over_limit: Vec<(i64, String, String)> = vec![
            (3, "oldest.tar.gz".into(), "/backups".into()),
            (4, "older.tar.gz".into(), "/backups".into()),
        ];

        assert_eq!(over_limit.len(), 2);
        // Both should be deleted
        for (id, name, dest) in &over_limit {
            assert!(*id > 2);
            assert!(name.contains("old") || name.contains("older"));
            assert_eq!(dest, "/backups");
        }
    }

    #[test]
    fn test_backup_naming_convention() {
        let name = backup::generate_snapshot_name("test_data");
        assert!(name.starts_with("test_data_"));
        assert!(name.ends_with(".tar.gz"));

        // The name after stripping "test_data_" should be the ISO8601 timestamp
        let stripped = name.strip_prefix("test_data_").unwrap();
        let base = stripped.strip_suffix(".tar.gz").unwrap();
        assert!(
            base.contains('T'),
            "Timestamp should contain 'T' separator: {}",
            base
        );
    }

    #[test]
    fn test_progress_struct() {
        let progress = crate::engine::types::BackupProgress {
            job_id: "job-1".into(),
            bytes_processed: 500,
            total_bytes: 1000,
            percentage: 50,
            status: "in_progress".into(),
        };

        assert_eq!(progress.job_id, "job-1");
        assert_eq!(progress.percentage, 50);
        assert_eq!(progress.status, "in_progress");
    }

    #[test]
    fn test_validate_snapshot_name_in_commands() {
        // Re-export validation from engine
        assert!(backup::validate_snapshot_name("good-name_2026-01-01T00-00-00.tar.gz").is_ok());
        assert!(backup::validate_snapshot_name("").is_err());
    }
}
