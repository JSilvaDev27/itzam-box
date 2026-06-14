// ItzamBox — Docker Event Stream Commands
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// Spawns `docker events --format '{{json .}}'` as a managed child process and
// forwards every parsed event to the frontend via Tauri's event system.

use std::sync::Mutex;

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::io::AsyncBufReadExt;
use tokio::process::Command as TokioCommand;
use tokio::io::BufReader as TokioBufReader;

use crate::engine::types::DockerEvent;

// ─── Shared State ───────────────────────────────────────────────────────────

/// Holds the optional handle to the running `docker events` child process so
/// we can kill it when the user stops the stream or the view unmounts.
#[derive(Default)]
pub struct EventStreamState {
    pub child: Mutex<Option<tokio::process::Child>>,
}

impl EventStreamState {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
        }
    }
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Parse a single raw JSON `docker events` line into our domain type.
fn parse_docker_event(raw: &Value) -> DockerEvent {
    let event_type = raw
        .get("Type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let action = raw
        .get("Action")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let actor_id = raw
        .get("Actor")
        .and_then(|a| a.get("ID"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let actor_name = raw
        .get("Actor")
        .and_then(|a| a.get("Attributes"))
        .and_then(|attrs| attrs.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let timestamp = raw
        .get("time")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let attributes = raw
        .get("Actor")
        .and_then(|a| a.get("Attributes"))
        .map(|attrs| {
            let mut map = std::collections::HashMap::new();
            if let Some(obj) = attrs.as_object() {
                for (k, v) in obj {
                    map.insert(k.clone(), v.as_str().unwrap_or("").to_string());
                }
            }
            map
        })
        .unwrap_or_default();

    DockerEvent {
        event_type,
        action,
        actor_id,
        actor_name,
        timestamp,
        attributes,
    }
}

// ─── Tauri Commands ─────────────────────────────────────────────────────────

/// Start streaming Docker events to the frontend.
///
/// Spawns `docker events --format '{{json .}}'` and reads its stdout line by
/// line inside a Tokio task. Every parsed event is emitted as a `"docker-event"`
/// Tauri event. The child handle is stored in [`EventStreamState`] so that
/// [`stop_event_stream`] can cleanly terminate it.
#[tauri::command]
pub async fn start_event_stream(
    app: AppHandle,
    state: State<'_, EventStreamState>,
) -> Result<(), String> {
    // If a stream is already running, silently succeed (idempotent start).
    {
        let guard = state.child.lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Ok(());
        }
    }

    let mut child = TokioCommand::new("docker")
        .args(["events", "--format", "{{json .}}"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("Failed to spawn docker events: {}", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "No stdout on docker events child".to_string())?;

    // Store handle so we can kill the process later.
    {
        let mut guard = state.child.lock().map_err(|e| e.to_string())?;
        *guard = Some(child);
    }

    let app_clone = app.clone();

    tokio::spawn(async move {
        let mut reader = TokioBufReader::new(stdout).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            // Trim whitespace — docker may emit blank lines on reconnect.
            let trimmed = line.trim().to_owned();
            if trimmed.is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(&trimmed) {
                Ok(raw) => {
                    let event = parse_docker_event(&raw);
                    let _ = app_clone.emit("docker-event", event);
                }
                Err(_) => {
                    // Docker may emit non-JSON diagnostics on stderr, but
                    // they go through stderr; we still log a debug trace.
                    log::debug!("Non-JSON docker events line: {}", trimmed);
                }
            }
        }

        // The stream ended (process exited or was killed). Clean up the state.
        if let Ok(mut guard) = app_clone.state::<EventStreamState>().child.lock() {
            guard.take();
        }

        log::info!("Docker event stream ended.");
    });

    log::info!("Docker event stream started.");
    Ok(())
}

/// Stop a running Docker event stream.
///
/// Kills the child `docker events` process and clears the stored handle.
/// Calling this when no stream is active is a no-op.
#[tauri::command]
pub async fn stop_event_stream(
    state: State<'_, EventStreamState>,
) -> Result<(), String> {
    let mut guard = state.child.lock().map_err(|e| e.to_string())?;

    if let Some(mut child) = guard.take() {
        // Kill the process — `kill_on_drop(true)` would do this on drop too,
        // but we explicitly kill here so the frontend can observe the stop
        // immediately and we can log the termination.
        let _ = child.start_kill();
        log::info!("Docker event stream stopped.");
    }

    Ok(())
}
