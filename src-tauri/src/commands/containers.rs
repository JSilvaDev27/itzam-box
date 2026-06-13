// ItzamBox — Container Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::ContainerInfo;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_containers(
    state: State<'_, AppState>,
    show_all: bool,
) -> Result<Vec<ContainerInfo>, String> {
    state.engine.list_containers(show_all).await
}

#[tauri::command]
pub async fn inspect_container(
    state: State<'_, AppState>,
    id: String,
) -> Result<serde_json::Value, String> {
    state.engine.inspect_container(&id).await
}

#[tauri::command]
pub async fn start_container(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.engine.start_container(&id).await
}

#[tauri::command]
pub async fn stop_container(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.engine.stop_container(&id).await
}

#[tauri::command]
pub async fn restart_container(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.engine.restart_container(&id).await
}

#[tauri::command]
pub async fn pause_container(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.engine.pause_container(&id).await
}

#[tauri::command]
pub async fn unpause_container(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.engine.unpause_container(&id).await
}

#[tauri::command]
pub async fn kill_container(
    state: State<'_, AppState>,
    id: String,
    signal: String,
) -> Result<(), String> {
    state.engine.kill_container(&id, &signal).await
}

#[tauri::command]
pub async fn get_container_stats(
    state: State<'_, AppState>,
    id: String,
) -> Result<crate::engine::types::ContainerStats, String> {
    state.engine.get_container_stats(&id).await
}

#[tauri::command]
pub async fn get_container_logs(
    _state: State<'_, AppState>,
    id: String,
    tail: usize,
    timestamps: bool,
) -> Result<String, String> {
    use std::process::Command;
    let tail_str = tail.to_string();
    let mut args = vec!["logs", "--tail", &tail_str];
    if timestamps {
        args.push("--timestamps");
    }
    args.push(&id);
    let output = Command::new("docker")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn rename_container(
    state: State<'_, AppState>,
    id: String,
    new_name: String,
) -> Result<(), String> {
    state.engine.rename_container(&id, &new_name).await
}

#[tauri::command]
pub async fn remove_container(
    state: State<'_, AppState>,
    id: String,
    force: bool,
    remove_volumes: bool,
) -> Result<(), String> {
    state
        .engine
        .remove_container(&id, force, remove_volumes)
        .await
}
