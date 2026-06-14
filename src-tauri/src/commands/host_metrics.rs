// ItzamBox — Host Metrics Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::HostMetrics;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_host_metrics(state: State<'_, AppState>) -> Result<HostMetrics, String> {
    let metrics = state.engine.get_host_metrics().await?;

    // Persist current snapshot to the raw host_metrics_history table.
    if let Ok(db) = state.db.lock() {
        if let Err(e) = crate::engine::metrics_history::insert_host_metrics(&db, &metrics) {
            log::warn!("Failed to persist host metrics: {}", e);
        }
    }

    Ok(metrics)
}
