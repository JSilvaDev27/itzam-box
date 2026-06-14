// ItzamBox — Notifications Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotificationRecord {
    pub id: String,
    pub type_str: String,
    pub title: String,
    pub message: String,
    pub read: bool,
    pub created_at: i64,
}

#[tauri::command]
pub async fn save_notification(
    state: State<'_, AppState>,
    id: String,
    type_str: String,
    title: String,
    message: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Prune excess if count >= 100
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM notifications", [], |r| r.get(0))
        .unwrap_or(0);

    if count >= 100 {
        conn.execute(
            "DELETE FROM notifications WHERE id NOT IN (SELECT id FROM notifications ORDER BY created_at DESC LIMIT 99)",
            [],
        )
        .ok();
    }

    let created_at = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO notifications (id, type, title, message, read, created_at) VALUES (?1, ?2, ?3, ?4, 0, ?5)",
        rusqlite::params![id, type_str, title, message, created_at],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_notifications(
    state: State<'_, AppState>,
    limit: Option<usize>,
    unread_only: Option<bool>,
) -> Result<Vec<NotificationRecord>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let mut query = "SELECT id, type, title, message, read, created_at FROM notifications".to_string();
    if unread_only.unwrap_or(false) {
        query.push_str(" WHERE read = 0");
    }
    query.push_str(" ORDER BY created_at DESC");
    if let Some(lim) = limit {
        query.push_str(&format!(" LIMIT {}", lim));
    }

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            let read_int: i32 = row.get(4)?;
            Ok(NotificationRecord {
                id: row.get(0)?,
                type_str: row.get(1)?,
                title: row.get(2)?,
                message: row.get(3)?,
                read: read_int != 0,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for rec in rows.flatten() {
        list.push(rec);
    }
    Ok(list)
}

#[tauri::command]
pub async fn mark_notification_read(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE notifications SET read = 1 WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn mark_all_read(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("UPDATE notifications SET read = 1", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_notifications(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notifications", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
