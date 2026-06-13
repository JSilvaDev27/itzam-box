// ItzamBox — Image Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use tauri::State;
use crate::AppState;
use crate::engine::types::ImageInfo;

#[tauri::command]
pub async fn list_images(state: State<'_, AppState>) -> Result<Vec<ImageInfo>, String> {
    state.engine.list_images().await
}
