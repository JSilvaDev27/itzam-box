// ItzamBox — Image Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::ImageInfo;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_images(state: State<'_, AppState>) -> Result<Vec<ImageInfo>, String> {
    state.engine.list_images().await
}

#[tauri::command]
pub async fn pull_image(state: State<'_, AppState>, image_name: String) -> Result<(), String> {
    state.engine.pull_image(&image_name).await
}

#[tauri::command]
pub async fn remove_image(
    state: State<'_, AppState>,
    id: String,
    force: bool,
) -> Result<(), String> {
    state.engine.remove_image(&id, force).await
}

#[tauri::command]
pub async fn tag_image(
    state: State<'_, AppState>,
    id: String,
    repository: String,
    tag: String,
) -> Result<(), String> {
    state.engine.tag_image(&id, &repository, &tag).await
}

#[tauri::command]
pub async fn inspect_image(
    state: State<'_, AppState>,
    id: String,
) -> Result<serde_json::Value, String> {
    state.engine.inspect_image(&id).await
}
