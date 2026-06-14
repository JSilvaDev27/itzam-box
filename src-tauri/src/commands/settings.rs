// ItzamBox — Settings Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn get_config(state: State<'_, AppState>, key: String) -> Result<String, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.query_row(
        "SELECT value FROM system_config WHERE key = ?1",
        [&key],
        |row| row.get(0),
    )
    .map_err(|e| format!("Config key '{}' not found: {}", key, e))
}

#[tauri::command]
pub fn set_config(state: State<'_, AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO system_config (key, value) VALUES (?1, ?2)",
        rusqlite::params![key, value],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

/// Export all settings from system_config as a JSON object.
#[tauri::command]
pub fn export_settings(state: State<'_, AppState>) -> Result<Value, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM system_config ORDER BY key")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })
        .map_err(|e| e.to_string())?;

    let mut map = serde_json::Map::new();
    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        map.insert(key, Value::String(value));
    }
    Ok(Value::Object(map))
}

/// Import settings from a JSON object. Each key-value pair is
/// upserted into system_config. Invalid JSON returns an error.
#[tauri::command]
pub fn import_settings(state: State<'_, AppState>, json: String) -> Result<(), String> {
    let parsed: Value =
        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON payload: {}", e))?;

    let obj = parsed
        .as_object()
        .ok_or_else(|| "JSON payload must be a key-value object".to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    for (key, value) in obj {
        let str_val = match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        conn.execute(
            "INSERT OR REPLACE INTO system_config (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, str_val],
        )
        .map_err(|e| format!("Failed to set '{}': {}", key, e))?;
    }
    Ok(())
}
