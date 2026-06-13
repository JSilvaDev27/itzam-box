// ItzamBox — Host Metrics Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::HostMetrics;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_host_metrics(state: State<'_, AppState>) -> Result<HostMetrics, String> {
    state.engine.get_host_metrics().await
}
