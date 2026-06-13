// ItzamBox — Host Metrics Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use tauri::State;
use crate::AppState;
use crate::engine::types::HostMetrics;

#[tauri::command]
pub async fn get_host_metrics(state: State<'_, AppState>) -> Result<HostMetrics, String> {
    state.engine.get_host_metrics().await
}
