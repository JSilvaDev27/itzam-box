// ItzamBox — Registry Management Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// CRUD for private container registries + docker login/logout/push via CLI.

use crate::AppState;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter, State};

// ─── Data Types ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Registry {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    pub is_default: bool,
}

/// Safe version of Registry returned to the frontend — never exposes auth_token.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistrySafe {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub username: Option<String>,
    pub is_default: bool,
}

// ─── CRUD ──────────────────────────────────────────────────────────────────

/// List all registries (without auth_token for security).
#[tauri::command]
pub async fn list_registries(state: State<'_, AppState>) -> Result<Vec<RegistrySafe>, String> {
    let db = state.db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    let mut stmt = db
        .prepare("SELECT id, name, url, username, is_default FROM registries ORDER BY name")
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(RegistrySafe {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                username: row.get(3)?,
                is_default: row.get::<_, i32>(4)? != 0,
            })
        })
        .map_err(|e| format!("Query failed: {}", e))?;

    let mut registries = Vec::new();
    for row in rows {
        registries.push(row.map_err(|e| format!("Row read error: {}", e))?);
    }
    Ok(registries)
}

/// Add a new registry. If `is_default` is set, unmark any existing default first.
#[tauri::command]
pub async fn add_registry(
    state: State<'_, AppState>,
    name: String,
    url: String,
    username: Option<String>,
    auth_token: Option<String>,
    is_default: bool,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    if is_default {
        db.execute("UPDATE registries SET is_default = 0", [])
            .map_err(|e| format!("Failed to reset defaults: {}", e))?;
    }

    db.execute(
        "INSERT INTO registries (name, url, username, auth_token, is_default) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![name, url, username, auth_token, is_default as i32],
    )
    .map_err(|e| format!("Failed to insert registry: {}", e))?;

    let id = db.last_insert_rowid();
    Ok(id)
}

/// Update an existing registry.
#[tauri::command]
pub async fn update_registry(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    url: String,
    username: Option<String>,
    auth_token: Option<String>,
    is_default: bool,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("DB lock error: {}", e))?;

    if is_default {
        db.execute("UPDATE registries SET is_default = 0", [])
            .map_err(|e| format!("Failed to reset defaults: {}", e))?;
    }

    db.execute(
        "UPDATE registries SET name = ?1, url = ?2, username = ?3, auth_token = ?4, is_default = ?5 WHERE id = ?6",
        rusqlite::params![name, url, username, auth_token, is_default as i32, id],
    )
    .map_err(|e| format!("Failed to update registry: {}", e))?;

    Ok(())
}

/// Remove a registry by id.
#[tauri::command]
pub async fn remove_registry(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    db.execute("DELETE FROM registries WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| format!("Failed to delete registry: {}", e))?;
    Ok(())
}

/// Set the given registry as the default (unmark others first).
#[tauri::command]
pub async fn set_default_registry(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("DB lock error: {}", e))?;
    db.execute("UPDATE registries SET is_default = 0", [])
        .map_err(|e| format!("Failed to reset defaults: {}", e))?;
    db.execute(
        "UPDATE registries SET is_default = 1 WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| format!("Failed to set default registry: {}", e))?;
    Ok(())
}

// ─── Docker Auth Commands ──────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct LoginOutput {
    pub line: String,
}

/// Execute `docker login` via the CLI using password-stdin.
/// Returns the raw CLI output (e.g. "Login Succeeded").
#[tauri::command]
pub async fn docker_login(
    url: String,
    username: String,
    password: String,
) -> Result<String, String> {
    let mut child = Command::new("docker")
        .args(["login", &url, "--username", &username, "--password-stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker login: {}", e))?;

    // Write password to stdin and close
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(password.as_bytes())
            .map_err(|e| format!("Failed to write password: {}", e))?;
        // Drop stdin to close it — docker will proceed
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to wait for docker login: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(if !stdout.is_empty() { stdout } else { stderr })
    } else {
        let msg = if !stderr.is_empty() { stderr } else { stdout };
        Err(format!("Docker login failed: {}", msg))
    }
}

/// Execute `docker logout` for the given registry URL.
#[tauri::command]
pub async fn docker_logout(url: String) -> Result<(), String> {
    let output = Command::new("docker")
        .args(["logout", &url])
        .output()
        .map_err(|e| format!("Failed to spawn docker logout: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("Docker logout failed: {}", stderr));
    }
    Ok(())
}

// ─── Push Image with Progress Events ───────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct PushLogLine {
    pub line: String,
    pub stream: String, // "stdout" or "stderr"
}

#[derive(Serialize, Clone, Debug)]
pub struct PushComplete {
    pub success: bool,
    pub image: String,
    pub error: Option<String>,
}

/// Push an image to a registry. Optionally tags it first.
/// Emits `push-log` and `push-complete` Tauri events similar to build_image.
#[tauri::command]
pub async fn push_image(
    app: AppHandle,
    image_name: String,
    registry_url: Option<String>,
) -> Result<(), String> {
    let push_target = if let Some(ref reg_url) = registry_url {
        // Determine registry host from URL (e.g. "https://ghcr.io" -> "ghcr.io")
        let registry_host = reg_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        let tagged = format!("{}/{}", registry_host, image_name);

        // Step 1: `docker tag`
        let tag_status = Command::new("docker")
            .args(["tag", &image_name, &tagged])
            .output()
            .map_err(|e| format!("Failed to run docker tag: {}", e))?;

        if !tag_status.status.success() {
            let err = String::from_utf8_lossy(&tag_status.stderr).trim().to_string();
            let err_msg = format!("docker tag failed: {}", err);
            let complete = PushComplete {
                success: false,
                image: image_name.clone(),
                error: Some(err_msg.clone()),
            };
            app.emit("push-complete", complete).ok();
            return Err(err_msg);
        }
        tagged
    } else {
        image_name.clone()
    };

    // Step 2: `docker push`
    let mut child = Command::new("docker")
        .args(["push", &push_target])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn docker push: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture stderr".to_string())?;

    let app_stdout = app.clone();
    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(text) = line {
                let payload = PushLogLine {
                    line: text,
                    stream: "stdout".to_string(),
                };
                app_stdout.emit("push-log", payload).ok();
            }
        }
    });

    let app_stderr = app.clone();
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(text) = line {
                let payload = PushLogLine {
                    line: text,
                    stream: "stderr".to_string(),
                };
                app_stderr.emit("push-log", payload).ok();
            }
        }
    });

    stdout_handle
        .join()
        .map_err(|_| "stdout reader panicked".to_string())?;
    stderr_handle
        .join()
        .map_err(|_| "stderr reader panicked".to_string())?;

    let exit_status = child
        .wait()
        .map_err(|e| format!("Failed to wait on docker push: {}", e))?;

    if !exit_status.success() {
        let code = exit_status.code().unwrap_or(-1);
        let err_msg = format!("docker push failed with exit code {}", code);
        let complete = PushComplete {
            success: false,
            image: image_name.clone(),
            error: Some(err_msg.clone()),
        };
        app.emit("push-complete", complete).ok();
        return Err(err_msg);
    }

    let complete = PushComplete {
        success: true,
        image: image_name.clone(),
        error: None,
    };
    app.emit("push-complete", complete).ok();

    // If we tagged, remove the extra tag after successful push (cleanup)
    if registry_url.is_some() {
        Command::new("docker")
            .args(["rmi", &push_target])
            .output()
            .ok();
    }

    Ok(())
}
