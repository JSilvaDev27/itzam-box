// ItzamBox — Settings Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use tauri::State;
use crate::AppState;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>, key: String) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT value FROM system_config WHERE key = ?1",
        [&key],
        |row| row.get(0),
    ).map_err(|e| format!("Config key '{}' not found: {}", key, e))
}

#[tauri::command]
pub fn set_config(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO system_config (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
