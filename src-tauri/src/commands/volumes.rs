// ItzamBox — Volume Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::VolumeInfo;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_volumes(state: State<'_, AppState>) -> Result<Vec<VolumeInfo>, String> {
    state.engine.list_volumes().await
}

#[tauri::command]
pub async fn create_volume(
    state: State<'_, AppState>,
    name: String,
    driver: Option<String>,
) -> Result<(), String> {
    state.engine.create_volume(&name, driver.as_deref()).await
}

#[tauri::command]
pub async fn remove_volume(
    state: State<'_, AppState>,
    name: String,
    force: bool,
) -> Result<(), String> {
    state.engine.remove_volume(&name, force).await
}
