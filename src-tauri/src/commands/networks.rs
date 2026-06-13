// ItzamBox — Network Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use tauri::State;
use crate::AppState;
use crate::engine::types::NetworkInfo;

#[tauri::command]
pub async fn list_networks(state: State<'_, AppState>) -> Result<Vec<NetworkInfo>, String> {
    state.engine.list_networks().await
}

#[tauri::command]
pub async fn create_network(state: State<'_, AppState>, name: String, driver: String, subnet: Option<String>, gateway: Option<String>) -> Result<(), String> {
    state.engine.create_network(&name, &driver, subnet.as_deref(), gateway.as_deref()).await
}

#[tauri::command]
pub async fn remove_network(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.engine.remove_network(&id).await
}
