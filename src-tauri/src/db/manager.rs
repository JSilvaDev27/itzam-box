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

    // Run migrations
    conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
        .map_err(|e| format!("Migration 001 failed: {}", e))?;
    conn.execute_batch(include_str!("../../migrations/002_templates.sql"))
        .map_err(|e| format!("Migration 002 failed: {}", e))?;
    conn.execute_batch(include_str!("../../migrations/003_notifications.sql"))
        .map_err(|e| format!("Migration 003 failed: {}", e))?;

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
        )
        .map_err(|e| format!("Failed to insert default config '{}': {}", key, e))?;
    }

    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_setup_database_creates_file() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("test.db");

        let conn = setup_database(db_path.clone()).expect("setup_database failed");
        // Verify the file exists
        assert!(db_path.exists());
        drop(conn);
    }

    #[test]
    fn test_setup_database_wal_mode() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("test_wal.db");

        let conn = setup_database(db_path).expect("setup_database failed");
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .expect("failed to query journal_mode");
        // WAL mode may be returned as "wal"
        assert!(journal_mode.to_lowercase() == "wal");
    }

    #[test]
    fn test_default_configs_inserted() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("test_defaults.db");

        let conn = setup_database(db_path).expect("setup_database failed");

        let mut stmt = conn
            .prepare("SELECT key, value FROM system_config ORDER BY key")
            .expect("failed to prepare query");

        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query failed")
            .filter_map(|r| r.ok())
            .collect();

        assert!(!rows.is_empty(), "Expected default configs to be inserted");
        // Verify known defaults exist
        let has_theme = rows.iter().any(|(k, v)| k == "theme" && v == "dark");
        let has_lang = rows.iter().any(|(k, v)| k == "lang" && v == "es");
        assert!(has_theme, "Default 'theme' config missing");
        assert!(has_lang, "Default 'lang' config missing");
    }

    #[test]
    fn test_idempotent_setup() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("test_idem.db");

        // First setup
        let _conn1 = setup_database(db_path.clone()).expect("first setup failed");
        // Second setup should succeed (INSERT OR IGNORE)
        let conn2 = setup_database(db_path.clone()).expect("second setup failed");

        let count: i64 = conn2
            .query_row("SELECT COUNT(*) FROM system_config", [], |row| row.get(0))
            .expect("count failed");

        // Should have the same 6 default rows, not duplicated
        assert_eq!(count, 6);
    }

    #[test]
    fn test_setup_migration_tables_exist() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("test_tables.db");

        let conn = setup_database(db_path).expect("setup_database failed");

        // Verify key tables from migration exist
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .expect("failed to query tables");

        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .expect("query failed")
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"system_config".to_string()));
        assert!(tables.contains(&"pinned_containers".to_string()));
        assert!(tables.contains(&"host_metrics_history".to_string()));
        assert!(tables.contains(&"container_templates".to_string()));
    }

    #[test]
    fn test_setup_in_nonexistent_directory() {
        let dir = tempdir().expect("failed to create temp dir");
        let db_path = dir.path().join("deeply/nested/path/test.db");

        let conn = setup_database(db_path.clone()).expect("setup_database in nested dir failed");
        assert!(db_path.exists());
        drop(conn);
    }
}
