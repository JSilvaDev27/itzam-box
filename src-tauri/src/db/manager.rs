// ItzamBox — SQLite3 Database Manager
// Copyright (C) 2026 SodigTech — GPL-3.0

use rusqlite::Connection;
use std::path::PathBuf;

pub fn setup_database(db_path: PathBuf) -> Result<Connection, String> {
    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create data dir: {}", e))?;
    }

    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;

    // Enable WAL mode for concurrent reads
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("Failed to enable WAL: {}", e))?;

    // Run initial migration
    conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
        .map_err(|e| format!("Migration failed: {}", e))?;

    // Insert default configurations if not present
    let defaults = [
        ("theme", "dark"),
        ("lang", "es"),
        ("sidebar_collapsed", "false"),
        ("metrics_interval_ms", "1500"),
        ("log_tail_default", "500"),
        ("auto_refresh_containers", "true"),
    ];

    for (key, value) in &defaults {
        conn.execute(
            "INSERT OR IGNORE INTO system_config (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        ).map_err(|e| format!("Failed to insert default config '{}': {}", key, e))?;
    }

    Ok(conn)
}
