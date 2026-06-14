// ItzamBox — Image Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::{ImageInfo, ImageLayerInfo};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DockerHubImage {
    pub name: String,
    pub description: String,
    pub star_count: u32,
    pub pull_count: u64,
    pub is_official: bool,
    pub is_automated: bool,
}

/// Search Docker Hub for images matching the given query.
/// Calls the Docker Hub v2 REST API directly.
#[tauri::command]
pub async fn search_dockerhub(query: String, limit: Option<usize>) -> Result<Vec<DockerHubImage>, String> {
    let page_size = limit.unwrap_or(25).min(100);
    let url = format!(
        "https://hub.docker.com/v2/search/repositories/?query={}&page_size={}",
        urlencoding(&query),
        page_size
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Failed to reach Docker Hub: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Docker Hub returned HTTP {}",
            response.status()
        ));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Docker Hub response: {}", e))?;

    let results = body
        .get("results")
        .and_then(|r| r.as_array())
        .ok_or_else(|| "Unexpected Docker Hub response format".to_string())?;

    let images: Vec<DockerHubImage> = results
        .iter()
        .map(|r| DockerHubImage {
            name: r
                .get("repo_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: r
                .get("short_description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            star_count: r.get("star_count").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            pull_count: r.get("pull_count").and_then(|v| v.as_u64()).unwrap_or(0),
            is_official: r.get("is_official").and_then(|v| v.as_bool()).unwrap_or(false),
            is_automated: r.get("is_automated").and_then(|v| v.as_bool()).unwrap_or(false),
        })
        .collect();

    Ok(images)
}

/// Simple percent-encoding for the query string (covers most cases).
fn urlencoding(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

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

#[tauri::command]
pub async fn get_image_history(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<ImageLayerInfo>, String> {
    state.engine.get_image_history(&id).await
}

/// Save one or more images as a tar archive.
/// Executes: `docker save -o <output_path> <image_name>`
#[tauri::command]
pub async fn save_image(
    image_name: String,
    output_path: String,
) -> Result<(), String> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let output = Command::new("docker")
        .args(["save", "-o", &output_path, &image_name])
        .output()
        .map_err(|e| format!("Failed to spawn docker save: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker save failed: {}", stderr.trim()));
    }

    log::info!("Saved image {} to {}", image_name, output_path);
    Ok(())
}

/// Load an image from a tar archive.
/// Executes: `docker load -i <input_path>`
/// Returns the "Loaded image: <name>" string from the output.
#[tauri::command]
pub async fn load_image(input_path: String) -> Result<String, String> {
    if !Path::new(&input_path).exists() {
        return Err(format!("File not found: {}", input_path));
    }

    let output = Command::new("docker")
        .args(["load", "-i", &input_path])
        .output()
        .map_err(|e| format!("Failed to spawn docker load: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("docker load failed: {}", stderr.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Docker may write the result to stdout or stderr depending on version
    let result = if stdout.contains("Loaded image") {
        stdout.trim().to_string()
    } else if stderr.contains("Loaded image") {
        stderr.trim().to_string()
    } else {
        // Fallback: return the full output
        let full = format!("{}{}", stdout.trim(), stderr.trim());
        if full.is_empty() {
            "Image loaded successfully (no output)".to_string()
        } else {
            full
        }
    };

    log::info!("Loaded image from {}: {}", input_path, result);
    Ok(result)
}

// ─── Build Image ──────────────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct BuildLogLine {
    pub line: String,
    pub stream: String, // "stdout" or "stderr"
}

#[derive(Serialize, Clone, Debug)]
pub struct BuildComplete {
    pub success: bool,
    pub image_id: Option<String>,
    pub tags: Vec<String>,
    pub error: Option<String>,
}

/// Execute `docker build` with real-time log streaming via Tauri events.
///
/// Docker writes its build progress to **stderr** (not stdout), so we
/// capture both streams simultaneously on separate threads.
#[tauri::command]
pub async fn build_image(
    app: AppHandle,
    dockerfile_path: String,
    context_dir: String,
    tags: Vec<String>,
    build_args: Vec<String>,
    no_cache: bool,
    pull_base: bool,
) -> Result<String, String> {
    // ── Build the docker build command ──────────────────────────────────
    let mut cmd = Command::new("docker");
    cmd.arg("build");

    // Dockerfile path
    cmd.arg("-f");
    cmd.arg(&dockerfile_path);

    // Tags
    for tag in &tags {
        cmd.arg("-t");
        cmd.arg(tag);
    }

    // Build args
    for arg in &build_args {
        cmd.arg("--build-arg");
        cmd.arg(arg);
    }

    // No cache
    if no_cache {
        cmd.arg("--no-cache");
    }

    // Pull base image
    if pull_base {
        cmd.arg("--pull");
    }

    // Context directory
    cmd.arg(&context_dir);

    // Pipe stdout + stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // ── Spawn the process ───────────────────────────────────────────────
    let mut child = cmd.spawn().map_err(|e| {
        format!("Failed to spawn `docker build`: {}", e)
    })?;

    let stdout = child.stdout.take()
        .ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = child.stderr.take()
        .ok_or_else(|| "Failed to capture stderr".to_string())?;

    let app_stdout = app.clone();
    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for text in reader.lines().map_while(Result::ok) {
            let payload = BuildLogLine {
                line: text,
                stream: "stdout".to_string(),
            };
            app_stdout.emit("build-log", payload).ok();
        }
    });

    let app_stderr = app.clone();
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for text in reader.lines().map_while(Result::ok) {
            let payload = BuildLogLine {
                line: text,
                stream: "stderr".to_string(),
            };
            app_stderr.emit("build-log", payload).ok();
        }
    });

    // ── Wait for both reader threads ────────────────────────────────────
    stdout_handle.join().map_err(|_| "stdout reader panicked".to_string())?;
    stderr_handle.join().map_err(|_| "stderr reader panicked".to_string())?;

    // ── Wait for process to finish ──────────────────────────────────────
    let exit_status = child.wait().map_err(|e| {
        format!("Failed to wait on docker build: {}", e)
    })?;

    if !exit_status.success() {
        let exit_code = exit_status.code().unwrap_or(-1);
        let err_msg = format!("docker build failed with exit code {}", exit_code);

        let complete = BuildComplete {
            success: false,
            image_id: None,
            tags: tags.clone(),
            error: Some(err_msg.clone()),
        };
        app.emit("build-complete", complete).ok();

        return Err(err_msg);
    }

    // ── Extract image ID from build output ──────────────────────────────
    // Docker prints "Successfully built <sha>" on stdout at the end.
    // Also look for "writing image sha256:..." in stderr (BuildKit).
    let image_id = extract_image_id(&dockerfile_path, &context_dir, &tags).unwrap_or_default();

    let complete = BuildComplete {
        success: true,
        image_id: Some(image_id.clone()),
        tags: tags.clone(),
        error: None,
    };
    app.emit("build-complete", complete).ok();

    Ok(image_id)
}

/// Re-run `docker build` quietly and parse the image ID from its output.
/// We do a second pass because the streaming threads consumed the output.
fn extract_image_id(
    dockerfile_path: &str,
    context_dir: &str,
    tags: &[String],
) -> Option<String> {
    let mut cmd = Command::new("docker");
    cmd.arg("build");
    cmd.arg("-q"); // Quiet mode — only print image ID
    cmd.arg("-f");
    cmd.arg(dockerfile_path);
    for tag in tags {
        cmd.arg("-t");
        cmd.arg(tag);
    }
    cmd.arg(context_dir);

    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let id = stdout.trim();
            if !id.is_empty() {
                return Some(id.to_string());
            }
            // Also check stderr for BuildKit output
            let stderr = String::from_utf8_lossy(&output.stderr);
            for line in stderr.lines() {
                if let Some(pos) = line.find("writing image sha256:") {
                    let hash_line = &line[pos + 20..];
                    let hash = hash_line.split(':').next().unwrap_or(hash_line).trim();
                    if !hash.is_empty() {
                        return Some(format!("sha256:{}", hash));
                    }
                }
            }
        }
    }
    None
}
