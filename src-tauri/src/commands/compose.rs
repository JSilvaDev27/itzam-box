// ItzamBox — Docker Compose Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

// ─── Data Types ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComposeProject {
    pub name: String,
    pub path: String,
    pub file: String,
    pub services: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComposeServiceStatus {
    pub name: String,
    pub id: Option<String>,
    pub state: String,
    pub status: String,
    pub ports: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComposeFileInfo {
    pub services: Vec<ComposeServiceDefinition>,
    pub volumes: Vec<String>,
    pub networks: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComposeServiceDefinition {
    pub name: String,
    pub image: Option<String>,
    pub build: Option<String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub environment: Vec<String>,
    pub depends_on: Vec<String>,
}

// ─── Compose Binary Detection ─────────────────────────────────────────────

fn compose_is_plugin() -> bool {
    static IS_PLUGIN: OnceLock<bool> = OnceLock::new();
    *IS_PLUGIN.get_or_init(|| {
        Command::new("docker")
            .args(["compose", "version"])
            .output()
            .is_ok_and(|o| o.status.success())
    })
}

fn compose_is_standalone() -> bool {
    static IS_STANDALONE: OnceLock<bool> = OnceLock::new();
    *IS_STANDALONE.get_or_init(|| {
        Command::new("docker-compose")
            .arg("version")
            .output()
            .is_ok_and(|o| o.status.success())
    })
}

fn ensure_compose() -> Result<(), String> {
    if compose_is_plugin() || compose_is_standalone() {
        Ok(())
    } else {
        Err("Docker Compose not found. Install docker-compose or Docker Compose plugin.".into())
    }
}

/// Build a `std::process::Command` configured for the detected compose binary
/// with the given sub-args and compose file path.
fn build_compose_cmd(args: &[&str], compose_file: &str) -> Command {
    if compose_is_plugin() {
        let mut cmd = Command::new("docker");
        cmd.arg("compose");
        cmd.arg("-f");
        cmd.arg(compose_file);
        cmd.args(args);
        cmd
    } else {
        let mut cmd = Command::new("docker-compose");
        cmd.arg("-f");
        cmd.arg(compose_file);
        cmd.args(args);
        cmd
    }
}

/// Run a docker compose command synchronously and return stdout as String.
fn run_compose(args: &[&str], compose_file: &str) -> Result<String, String> {
    ensure_compose()?;
    let mut cmd = build_compose_cmd(args, compose_file);
    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Spawn a docker compose command in the background (fire-and-forget).
fn run_compose_spawn(args: &[&str], compose_file: &str) -> Result<(), String> {
    ensure_compose()?;
    let mut cmd = build_compose_cmd(args, compose_file);
    cmd.spawn().map_err(|e| format!("Failed to spawn: {}", e))?;
    Ok(())
}

// ─── Simple YAML Parser ────────────────────────────────────────────────────
// We parse docker-compose.yml / compose.yaml naively by scanning lines.
// This avoids adding the `serde_yaml` dependency.

fn parse_compose_yaml(content: &str) -> ComposeFileInfo {
    let mut services: Vec<ComposeServiceDefinition> = Vec::new();
    let mut volumes: Vec<String> = Vec::new();
    let mut networks: Vec<String> = Vec::new();

    #[derive(PartialEq)]
    enum Section {
        None,
        Services,
        Volumes,
        Networks,
    }

    let mut current_section = Section::None;
    let mut current_service_index: Option<usize> = None;
    let mut in_service_property = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        // Detect top-level sections ending with ':'
        if trimmed.ends_with(':') && !trimmed.starts_with('-') {
            let key = trimmed.trim_end_matches(':');
            match key {
                "services" => {
                    current_section = Section::Services;
                    continue;
                }
                "volumes" => {
                    current_section = Section::Volumes;
                    continue;
                }
                "networks" => {
                    current_section = Section::Networks;
                    continue;
                }
                "version" | "name" | "configs" | "secrets" => {
                    current_section = Section::None;
                    continue;
                }
                _ => {}
            }

            // If we're not in a recognized section, check if this key starts at indent 0 (new top-level)
            if indent == 0 && current_section != Section::None {
                // Leaving current section
                current_section = Section::None;
                continue;
            }
        }

        match current_section {
            Section::Services => {
                // Detect top-level service names (indent 2, key:)
                if indent == 2 && trimmed.ends_with(':') && !trimmed.starts_with('-') {
                    let svc_name = trimmed.trim_end_matches(':').to_string();
                    let def = ComposeServiceDefinition {
                        name: svc_name,
                        image: None,
                        build: None,
                        ports: Vec::new(),
                        volumes: Vec::new(),
                        environment: Vec::new(),
                        depends_on: Vec::new(),
                    };
                    services.push(def);
                    current_service_index = Some(services.len() - 1);
                    in_service_property.clear();
                    continue;
                }

                // Service properties (indent 4+)
                if indent >= 4 && current_service_index.is_some() {
                    if let Some(idx) = current_service_index {
                        // Check for a list item (starts with -)
                        if trimmed.starts_with('-') {
                            let item = trimmed.trim_start_matches("- ").trim();
                            if !item.is_empty() && !in_service_property.is_empty() {
                                match in_service_property.as_str() {
                                    "ports" => services[idx].ports.push(item.to_string()),
                                    "volumes" => {
                                        // Short syntax: host:container or just name
                                        let src = item.split(':').next().unwrap_or(item);
                                        services[idx].volumes.push(src.to_string());
                                    }
                                    "environment" => {
                                        services[idx].environment.push(item.to_string());
                                    }
                                    "depends_on" => {
                                        services[idx].depends_on.push(item.to_string());
                                    }
                                    _ => {}
                                }
                            }
                            continue;
                        }

                        // Key: value property at indent 4
                        if let Some((key, value)) = trimmed.split_once(':') {
                            let prop_key = key.trim();
                            let prop_val = value.trim();

                            // List property start (value is empty, items follow)
                            if prop_val.is_empty() {
                                in_service_property = prop_key.to_string();
                                continue;
                            }

                            match prop_key {
                                "image" => services[idx].image = Some(prop_val.to_string()),
                                "build" => services[idx].build = Some(prop_val.to_string()),
                                "container_name" | "restart" | "command" | "entrypoint" => {}
                                _ => {}
                            }
                        }
                    }
                    continue;
                }

                // If indent drops to 0 or 1, we left the services section
                if indent <= 1 {
                    current_section = Section::None;
                }
            }

            Section::Volumes => {
                // Volume names at indent 2, key:
                if indent == 2 && trimmed.ends_with(':') && !trimmed.starts_with('-') {
                    volumes.push(trimmed.trim_end_matches(':').to_string());
                }
                if indent <= 1 {
                    current_section = Section::None;
                }
            }

            Section::Networks => {
                // Network names at indent 2, key:
                if indent == 2 && trimmed.ends_with(':') && !trimmed.starts_with('-') {
                    networks.push(trimmed.trim_end_matches(':').to_string());
                }
                if indent <= 1 {
                    current_section = Section::None;
                }
            }

            Section::None => {}
        }
    }

    ComposeFileInfo {
        services,
        volumes,
        networks,
    }
}

// ─── Parse compose ps output ───────────────────────────────────────────────

fn parse_ps_json(json_str: &str) -> Vec<ComposeServiceStatus> {
    let mut statuses: Vec<ComposeServiceStatus> = Vec::new();

    // Try to parse as JSON array
    if let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
        for v in values {
            let name = v
                .get("Name")
                .or_else(|| v.get("name"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let id = v
                .get("ID")
                .or_else(|| v.get("id").or_else(|| v.get("Container")))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty() && s != "<none>");

            let state = v
                .get("State")
                .or_else(|| v.get("state"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let status = v
                .get("Status")
                .or_else(|| v.get("status"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let ports = v
                .get("Ports")
                .or_else(|| v.get("ports"))
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            statuses.push(ComposeServiceStatus {
                name,
                id,
                state,
                status,
                ports,
            });
        }
    }

    statuses
}

fn parse_ps_text(output: &str) -> Vec<ComposeServiceStatus> {
    let mut statuses: Vec<ComposeServiceStatus> = Vec::new();

    // Try to find the table separator line to determine column positions
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() < 2 {
        return statuses;
    }

    // Skip header line (index 0), process data lines
    for line in lines.iter().skip(1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Typical output columns: NAME, IMAGE, COMMAND, SERVICE, CREATED, STATUS, PORTS
        // We split by 2+ spaces to handle the variable-width columns
        let columns: Vec<&str> = line.split([' ', '\t']).filter(|s| !s.is_empty()).collect();

        if columns.len() < 4 {
            continue;
        }

        let name = columns[0].to_string();

        // Status and state are typically the last meaningful columns
        // Try to find running/exited/paused in the text
        let full_line = line.to_lowercase();
        let state = if full_line.contains("running") || full_line.contains("up ") {
            "running"
        } else if full_line.contains("exited") || full_line.contains("exit") {
            "exited"
        } else if full_line.contains("paused") {
            "paused"
        } else {
            "unknown"
        }
        .to_string();

        // Reconstruct status from the last columns
        let status = columns
            .iter()
            .skip(1)
            .cloned()
            .collect::<Vec<&str>>()
            .join(" ");

        statuses.push(ComposeServiceStatus {
            name,
            id: None,
            state,
            status,
            ports: String::new(),
        });
    }

    statuses
}

// ─── Directory Scanning for Compose Projects ──────────────────────────────

const COMPOSE_FILES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yaml",
    "compose.yml",
];

fn scan_for_compose(dir: &Path, max_depth: usize) -> Vec<(PathBuf, String, Vec<String>)> {
    let mut results: Vec<(PathBuf, String, Vec<String>)> = Vec::new();

    if !dir.is_dir() {
        return results;
    }

    // Check each compose file name in the current directory
    for &cf in COMPOSE_FILES {
        let candidate = dir.join(cf);
        if candidate.is_file() {
            let content = std::fs::read_to_string(&candidate).unwrap_or_default();
            let info = parse_compose_yaml(&content);
            let service_names: Vec<String> = info.services.into_iter().map(|s| s.name).collect();
            results.push((dir.to_path_buf(), cf.to_string(), service_names));
            return results; // Only one compose file per directory
        }
    }

    // Recurse into subdirectories if depth allows
    if max_depth > 0 {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut sub_entries: Vec<_> = entries.flatten().collect();
            sub_entries.sort_by_key(|e| e.file_name());

            for entry in sub_entries {
                let path = entry.path();
                if path.is_dir() {
                    let fname = entry.file_name();
                    let name = fname.to_string_lossy();
                    // Skip hidden & common dirs
                    if name.starts_with('.')
                        || name == "node_modules"
                        || name == "target"
                        || name == ".git"
                    {
                        continue;
                    }
                    let sub_results = scan_for_compose(&path, max_depth - 1);
                    results.extend(sub_results);
                }
            }
        }
    }

    results
}

// Resolve the compose file path within a project directory.
fn resolve_compose_file(project_path: &str) -> Result<String, String> {
    let path = Path::new(project_path);
    if !path.is_dir() {
        return Err(format!("Directory not found: {}", project_path));
    }
    for &cf in COMPOSE_FILES {
        let candidate = path.join(cf);
        if candidate.is_file() {
            return Ok(candidate.to_string_lossy().to_string());
        }
    }
    Err(format!(
        "No docker-compose.yml or compose.yaml found in {}",
        project_path
    ))
}

// ─── Tauri Commands ────────────────────────────────────────────────────────

/// Detect Docker Compose projects by searching for compose files
/// in the given directory (defaults to $HOME) up to 2 levels deep.
#[tauri::command]
pub async fn detect_compose_projects(dir: Option<String>) -> Result<Vec<ComposeProject>, String> {
    let base = dir.map(PathBuf::from).unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_string());
        PathBuf::from(home)
    });

    let found = scan_for_compose(&base, 2);

    let projects: Vec<ComposeProject> = found
        .into_iter()
        .map(|(path, file, services)| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "project".to_string());
            ComposeProject {
                name,
                path: path.to_string_lossy().to_string(),
                file,
                services,
            }
        })
        .collect();

    Ok(projects)
}

/// Parse a docker-compose.yml / compose.yaml file and return its structure.
#[tauri::command]
pub async fn parse_compose_file(project_path: String) -> Result<ComposeFileInfo, String> {
    let path = Path::new(&project_path);
    if !path.is_dir() {
        return Err(format!("Directory not found: {}", project_path));
    }

    let compose_path = resolve_compose_file(&project_path)?;
    let content = std::fs::read_to_string(&compose_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(parse_compose_yaml(&content))
}

/// Start compose services (`docker compose up`).
/// If `detached` is true, runs in background (no blocking).
#[tauri::command]
pub async fn compose_up(
    project_path: String,
    detached: bool,
    services: Option<Vec<String>>,
) -> Result<(), String> {
    let compose_file = resolve_compose_file(&project_path)?;

    let mut args = vec!["up"];
    if detached {
        args.push("-d");
    }
    if let Some(svcs) = &services {
        for s in svcs {
            args.push(s.as_str());
        }
    }

    // Always spawn to avoid blocking the Tauri command
    run_compose_spawn(&args, &compose_file)
}

/// Stop and remove compose resources (`docker compose down`).
#[tauri::command]
pub async fn compose_down(
    project_path: String,
    remove_volumes: bool,
    remove_images: bool,
) -> Result<(), String> {
    let compose_file = resolve_compose_file(&project_path)?;

    let mut args = vec!["down"];
    if remove_volumes {
        args.push("-v");
    }
    if remove_images {
        args.push("--rmi");
        args.push("all");
    }

    run_compose(&args, &compose_file)?;
    Ok(())
}

/// Restart compose services (`docker compose restart`).
#[tauri::command]
pub async fn compose_restart(
    project_path: String,
    services: Option<Vec<String>>,
) -> Result<(), String> {
    let compose_file = resolve_compose_file(&project_path)?;

    let mut args = vec!["restart"];
    if let Some(svcs) = &services {
        for s in svcs {
            args.push(s.as_str());
        }
    }

    run_compose(&args, &compose_file)?;
    Ok(())
}

/// Fetch logs from compose services (`docker compose logs`).
#[tauri::command]
pub async fn compose_logs(
    project_path: String,
    tail: usize,
    services: Option<Vec<String>>,
) -> Result<String, String> {
    let compose_file = resolve_compose_file(&project_path)?;

    let tail_str = tail.to_string();
    let mut args = vec!["logs", "--no-color", "--tail", &tail_str];
    if let Some(svcs) = &services {
        for s in svcs {
            args.push(s.as_str());
        }
    }

    run_compose(&args, &compose_file)
}

/// List services in a compose project with their status (`docker compose ps`).
#[tauri::command]
pub async fn compose_ps(project_path: String) -> Result<Vec<ComposeServiceStatus>, String> {
    let compose_file = resolve_compose_file(&project_path)?;

    // Try JSON format first (modern Docker Compose)
    let json_args = vec!["ps", "--format", "json"];
    if let Ok(out) = run_compose(&json_args, &compose_file) {
        let parsed = parse_ps_json(&out);
        if !parsed.is_empty() {
            return Ok(parsed);
        }
    }

    // Fallback: parse text output
    let text_args = vec!["ps"];
    let out = run_compose(&text_args, &compose_file)?;
    Ok(parse_ps_text(&out))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub error: Option<String>,
}

/// Read compose file content directly
#[tauri::command]
pub async fn read_compose_file(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    if p.is_dir() {
        let file_path = resolve_compose_file(&path)?;
        std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read compose file: {}", e))
    } else {
        std::fs::read_to_string(p).map_err(|e| format!("Failed to read compose file: {}", e))
    }
}

/// Write compose file content directly
#[tauri::command]
pub async fn write_compose_file(path: String, content: String) -> Result<(), String> {
    let p = Path::new(&path);
    let target = if p.is_dir() {
        PathBuf::from(resolve_compose_file(&path)?)
    } else {
        p.to_path_buf()
    };

    std::fs::write(&target, content).map_err(|e| format!("Failed to write compose file: {}", e))
}

/// Validate compose file syntax using `docker compose config`
#[tauri::command]
pub async fn validate_compose_file(path: String) -> Result<ValidationResult, String> {
    ensure_compose()?;
    let p = Path::new(&path);
    let compose_file = if p.is_dir() {
        resolve_compose_file(&path)?
    } else {
        path.clone()
    };

    let mut cmd = build_compose_cmd(&["config", "--quiet"], &compose_file);
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                Ok(ValidationResult {
                    valid: true,
                    error: None,
                })
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok(ValidationResult {
                    valid: false,
                    error: Some(stderr),
                })
            }
        }
        Err(e) => Err(format!("Failed to run validation command: {}", e)),
    }
}

/// Format compose file content using prettier
#[tauri::command]
pub async fn format_compose_file(path: String) -> Result<String, String> {
    let p = Path::new(&path);
    let compose_file = if p.is_dir() {
        resolve_compose_file(&path)?
    } else {
        path.clone()
    };

    // Use npx prettier to format
    let output = Command::new("npx")
        .args(["prettier", "--parser", "yaml", &compose_file])
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                let formatted = String::from_utf8_lossy(&out.stdout).to_string();
                Ok(formatted)
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                Err(format!("Prettier formatting failed: {}", stderr))
            }
        }
        Err(_) => {
            // Fallback: if npx/prettier is not found, just return original content
            std::fs::read_to_string(&compose_file)
                .map_err(|e| format!("Failed to read fallback: {}", e))
        }
    }
}
