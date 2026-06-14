// ItzamBox — Metrics History Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::{ContainerMetricsPoint, MetricsDataPoint};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_host_metrics_range(
    state: State<'_, AppState>,
    from: i64,
    to: i64,
) -> Result<Vec<MetricsDataPoint>, String> {
    let points = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::engine::metrics_history::query_range(&db, from, to)?
    };
    Ok(points)
}

#[tauri::command]
pub async fn get_container_metrics_range(
    state: State<'_, AppState>,
    container_id: String,
    from: i64,
    to: i64,
) -> Result<Vec<ContainerMetricsPoint>, String> {
    let points = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::engine::metrics_history::query_container_range(&db, &container_id, from, to)?
    };
    Ok(points)
}

#[tauri::command]
pub async fn export_metrics_csv(
    state: State<'_, AppState>,
    from: i64,
    to: i64,
    dest: String,
) -> Result<(), String> {
    let result = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::engine::metrics_history::export_csv(&db, from, to, &dest)
    };
    result
}

#[tauri::command]
pub async fn export_metrics_json(
    state: State<'_, AppState>,
    from: i64,
    to: i64,
    dest: String,
) -> Result<(), String> {
    let result = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        crate::engine::metrics_history::export_json(&db, from, to, &dest)
    };
    result
}

#[tauri::command]
pub async fn get_metrics_db_size(state: State<'_, AppState>) -> Result<u64, String> {
    crate::engine::metrics_history::get_db_size(&state.db_path)
}
