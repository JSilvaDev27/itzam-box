// ItzamBox — Container Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::ContainerInfo;
use crate::AppState;
use std::process::Command;
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
    let stats = state.engine.get_container_stats(&id).await?;

    // Persist snapshot to the raw container_metrics_history table.
    if let Ok(db) = state.db.lock() {
        if let Err(e) = crate::engine::metrics_history::insert_container_stats(&db, &stats) {
            log::warn!("Failed to persist container stats for {}: {}", id, e);
        }
    }

    Ok(stats)
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

// ─── Container Creation ─────────────────────────────────────────────────

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn create_and_run_container(
    state: State<'_, AppState>,
    image: String,
    name: Option<String>,
    ports: Vec<crate::engine::types::PortMapping>,
    volumes: Vec<String>,
    env_vars: Vec<String>,
    network: Option<String>,
    restart_policy: Option<String>,
    command: Option<Vec<String>>,
    detach: bool,
    cpu_limit: Option<f64>,
    memory_limit: Option<String>,
    privileged: bool,
) -> Result<String, String> {
    state
        .engine
        .create_and_run_container(
            &image,
            name.as_deref(),
            ports,
            volumes,
            env_vars,
            network.as_deref(),
            restart_policy.as_deref(),
            command,
            detach,
            cpu_limit,
            memory_limit,
            privileged,
        )
        .await
}

// ─── Export / Import Commands ───────────────────────────────────────────

/// Export a container's filesystem as a tar archive.
/// Executes: `docker export -o <output_path> <container_id>`
#[tauri::command]
pub async fn export_container(container_id: String, output_path: String) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["export", "-o", &output_path, &container_id])
        .output()
        .map_err(|e| format!("Failed to spawn docker export: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker export failed: {}", stderr.trim()));
    }

    log::info!("Exported container {} to {}", container_id, output_path);
    Ok(())
}

/// Commit a container to a new image.
/// Executes: `docker commit [OPTIONS] <container_id> <repository>:<tag>`
#[tauri::command]
pub async fn commit_container(
    container_id: String,
    repository: String,
    tag: String,
    message: Option<String>,
    author: Option<String>,
) -> Result<String, String> {
    let mut args = vec!["commit".to_string()];

    if let Some(ref msg) = message {
        args.push("-m".to_string());
        args.push(msg.clone());
    }
    if let Some(ref a) = author {
        args.push("-a".to_string());
        args.push(a.clone());
    }

    let image_tag = format!("{}:{}", repository, tag);
    args.push(container_id.clone());
    args.push(image_tag.clone());

    let output = Command::new("docker")
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to spawn docker commit: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker commit failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Docker commit returns the new image ID (e.g. "sha256:abc123...")
    let image_id = stdout.trim().to_string();

    log::info!(
        "Committed container {} as {} — ID: {}",
        container_id,
        image_tag,
        image_id
    );
    Ok(image_id)
}

// ─── File Explorer Commands ──────────────────────────────────────────────

#[tauri::command]
pub async fn download_file_from_container(
    state: State<'_, AppState>,
    container_id: String,
    remote_path: String,
    local_dest: String,
) -> Result<(), String> {
    state
        .engine
        .download_file_from_container(
            &container_id,
            &remote_path,
            std::path::PathBuf::from(local_dest),
        )
        .await
}

#[tauri::command]
pub async fn upload_file_to_container(
    state: State<'_, AppState>,
    container_id: String,
    local_src: String,
    remote_dest: String,
) -> Result<(), String> {
    state
        .engine
        .upload_file_to_container(
            &container_id,
            std::path::PathBuf::from(local_src),
            &remote_dest,
        )
        .await
}

#[tauri::command]
pub async fn read_file_preview(
    state: State<'_, AppState>,
    container_id: String,
    remote_path: String,
    max_bytes: usize,
) -> Result<String, String> {
    state
        .engine
        .read_file_preview(&container_id, &remote_path, max_bytes)
        .await
}
