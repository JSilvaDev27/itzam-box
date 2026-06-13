// ItzamBox — Cleanup / Disk Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use tauri::State;
use crate::AppState;
use crate::engine::types::DiskUsageSummary;

#[tauri::command]
pub async fn get_disk_usage(state: State<'_, AppState>) -> Result<DiskUsageSummary, String> {
    state.engine.get_disk_usage().await
}

#[tauri::command]
pub async fn get_engine_version(state: State<'_, AppState>) -> Result<String, String> {
    state.engine.get_engine_version().await
}

#[tauri::command]
pub async fn get_engine_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    state.engine.get_engine_info().await
}

#[tauri::command]
pub async fn check_engine_status(state: State<'_, AppState>) -> Result<bool, String> {
    state.engine.check_engine_status().await
}

#[tauri::command]
pub async fn prune_containers(state: State<'_, AppState>) -> Result<u64, String> {
    state.engine.prune_containers().await
}

#[tauri::command]
pub async fn prune_images(state: State<'_, AppState>, dangling_only: bool) -> Result<u64, String> {
    state.engine.prune_images(dangling_only).await
}

#[tauri::command]
pub async fn prune_volumes(state: State<'_, AppState>) -> Result<u64, String> {
    state.engine.prune_volumes().await
}

#[tauri::command]
pub async fn prune_networks(state: State<'_, AppState>) -> Result<u64, String> {
    state.engine.prune_networks().await
}

#[tauri::command]
pub async fn list_container_dir(state: State<'_, AppState>, container_id: String, path: String) -> Result<Vec<crate::engine::types::FileMetadata>, String> {
    state.engine.list_container_dir(&container_id, &path).await
}
