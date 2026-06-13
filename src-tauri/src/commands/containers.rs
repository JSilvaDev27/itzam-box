// ItzamBox — Container Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use tauri::State;
use crate::AppState;
use crate::engine::types::ContainerInfo;

#[tauri::command]
pub async fn list_containers(state: State<'_, AppState>, show_all: bool) -> Result<Vec<ContainerInfo>, String> {
    state.engine.list_containers(show_all).await
}
