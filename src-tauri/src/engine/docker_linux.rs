// ItzamBox — Docker Linux Engine (Hybrid REST API + CLI)
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// AD-001.4: REST API for reads (via Unix socket), CLI for writes.
// Falls back to CLI if the Unix socket is unavailable.

use crate::engine::traits::ContainerEngine;
use crate::engine::types::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

// ─── Docker Unix Socket Client (REST API) ────────────────────────────────

/// Low-level HTTP client that speaks to the Docker Engine API over a
/// Unix domain socket.  Used exclusively for read-only operations.
struct DockerUnixClient {
    socket_path: String,
}

impl DockerUnixClient {
    /// Try to create a client bound to `/var/run/docker.sock`.
    /// Returns `Err` when the socket file does not exist.
    fn new() -> Result<Self, String> {
        let path = "/var/run/docker.sock";
        if !std::path::Path::new(path).exists() {
            return Err("Docker socket not found at /var/run/docker.sock".into());
        }
        Ok(Self {
            socket_path: path.to_string(),
        })
    }

    /// Perform a raw HTTP GET against the Docker API.
    /// The response body is returned as a `String`.
    async fn get(&self, api_path: &str) -> Result<String, String> {
        let socket_path = self.socket_path.clone();
        let api_path = api_path.to_owned();

        let fut = async move {
            let mut stream = UnixStream::connect(&socket_path).await?;
            let request = format!(
                "GET {} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n",
                api_path
            );
            stream.write_all(request.as_bytes()).await?;
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await?;
            Ok::<_, std::io::Error>(buf)
        };

        let buf = timeout(Duration::from_secs(30), fut)
            .await
            .map_err(|_| "Docker API request timed out after 30s".to_string())?
            .map_err(|e| format!("Docker socket I/O error: {}", e))?;

        let response = String::from_utf8_lossy(&buf);

        // Split HTTP headers from body at the first blank line.
        let parts: Vec<&str> = response.splitn(2, "\r\n\r\n").collect();
        if parts.len() < 2 {
            return Err("Invalid HTTP response from Docker daemon".into());
        }

        // Parse status line: "HTTP/1.1 200 OK"
        let status_line = parts[0].lines().next().unwrap_or("");
        let status_code: u16 = status_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("500")
            .parse()
            .unwrap_or(500);

        if status_code >= 400 {
            let body = parts[1].trim();
            // 404 is common for missing resources — surface it cleanly.
            return Err(format!("Docker API error ({}): {}", status_code, body));
        }

        Ok(parts[1].to_string())
    }

    /// Perform a GET and deserialise the JSON body into a generic value.
    async fn get_json_value(&self, api_path: &str) -> Result<serde_json::Value, String> {
        let body = self.get(api_path).await?;
        if body.trim().is_empty() {
            return Ok(serde_json::Value::Null);
        }
        serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse Docker API response: {}", e))
    }
}

// ─── Engine ──────────────────────────────────────────────────────────────

pub struct DockerLinuxEngine {
    /// `Some` when the Unix socket is present and usable at startup.
    rest_client: Option<DockerUnixClient>,
}

impl DockerLinuxEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let rest_client = DockerUnixClient::new().ok();
        Self { rest_client }
    }

    /// Quick file-existence check for the Docker socket.
    fn is_available() -> bool {
        std::path::Path::new("/var/run/docker.sock").exists()
    }

    /// Run a CLI `docker` subcommand and return stdout.
    /// Used as a fallback for reads and as the primary path for writes.
    fn run_docker(args: &[&str]) -> Result<String, String> {
        let output = Command::new("docker")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to run docker: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Docker error: {}", stderr.trim()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Parse a single field out of a `docker inspect` JSON array.
    /// Retained for compatibility — used by legacy CLI fallback paths.
    #[allow(dead_code)]
    fn parse_docker_inspect(output: &str, field: &str) -> String {
        let vals: Result<Vec<serde_json::Value>, _> = serde_json::from_str(output);
        if let Ok(vals) = vals {
            if let Some(first) = vals.first() {
                return first
                    .get(field)
                    .map(|v| v.to_string().trim_matches('"').to_string())
                    .unwrap_or_default();
            }
        }
        String::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  ContainerEngine trait implementation
// ═══════════════════════════════════════════════════════════════════════════

#[async_trait]
impl ContainerEngine for DockerLinuxEngine {
    // ─── Diagnostics ─────────────────────────────────────────────────────

    async fn check_engine_status(&self) -> Result<bool, String> {
        if let Some(client) = &self.rest_client {
            match client.get("/_ping").await {
                Ok(body) => return Ok(body.trim() == "OK"),
                Err(_) => return Ok(false),
            }
        }
        Ok(Self::is_available())
    }

    async fn get_engine_version(&self) -> Result<String, String> {
        if let Some(client) = &self.rest_client {
            match client.get_json_value("/version").await {
                Ok(val) => {
                    if let Some(ver) = val.get("Version").and_then(|v| v.as_str()) {
                        return Ok(ver.to_string());
                    }
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }
        if !Self::is_available() {
            return Err("Docker daemon not available".into());
        }
        let output = Self::run_docker(&["version", "--format", "{{.Server.Version}}"])?;
        Ok(output.trim().to_string())
    }

    async fn get_engine_info(&self) -> Result<serde_json::Value, String> {
        if let Some(client) = &self.rest_client {
            return client.get_json_value("/info").await;
        }
        let output = Self::run_docker(&["info", "--format", "{{json .}}"])?;
        serde_json::from_str(output.trim()).map_err(|e| format!("Parse error: {}", e))
    }

    // ─── Containers ──────────────────────────────────────────────────────

    async fn list_containers(&self, show_all: bool) -> Result<Vec<ContainerInfo>, String> {
        if let Some(client) = &self.rest_client {
            let query = if show_all { "?all=true" } else { "?all=false" };
            match client
                .get_json_value(&format!("/containers/json{}", query))
                .await
            {
                Ok(val) => {
                    if let Some(arr) = val.as_array() {
                        let mut containers = Vec::with_capacity(arr.len());
                        for raw in arr {
                            let id = raw
                                .get("Id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let name = raw
                                .get("Names")
                                .and_then(|n| n.as_array())
                                .and_then(|names| names.first())
                                .and_then(|n| n.as_str())
                                .map(|s| s.trim_start_matches('/').to_string())
                                .unwrap_or_default();

                            let image = raw
                                .get("Image")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let state = raw
                                .get("State")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let status = raw
                                .get("Status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let created_at =
                                raw.get("Created").and_then(|v| v.as_i64()).unwrap_or(0);

                            // Ports
                            let ports = raw
                                .get("Ports")
                                .and_then(|p| p.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .map(|p| PortMapping {
                                            host_ip: p
                                                .get("IP")
                                                .and_then(|ip| ip.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                            host_port: p
                                                .get("PublicPort")
                                                .and_then(|pp| pp.as_u64())
                                                .unwrap_or(0)
                                                as u16,
                                            container_port: p
                                                .get("PrivatePort")
                                                .and_then(|pp| pp.as_u64())
                                                .unwrap_or(0)
                                                as u16,
                                            protocol: p
                                                .get("Type")
                                                .and_then(|t| t.as_str())
                                                .unwrap_or("tcp")
                                                .to_string(),
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            // Labels
                            let labels = raw
                                .get("Labels")
                                .and_then(|l| l.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .map(|(k, v)| {
                                            (k.clone(), v.as_str().unwrap_or("").to_string())
                                        })
                                        .collect::<std::collections::HashMap<String, String>>()
                                })
                                .unwrap_or_default();

                            let compose_project = labels.get("com.docker.compose.project").cloned();
                            let compose_service = labels.get("com.docker.compose.service").cloned();

                            containers.push(ContainerInfo {
                                id,
                                name,
                                image,
                                status,
                                state,
                                ports,
                                cpu_percentage: 0.0,
                                memory_usage_bytes: 0,
                                memory_limit_bytes: 0,
                                network_rx_bytes: 0,
                                network_tx_bytes: 0,
                                pid: 0,
                                restart_count: 0,
                                created_at,
                                started_at: None,
                                labels,
                                compose_project,
                                compose_service,
                            });
                        }
                        return Ok(containers);
                    }
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }

        // ── CLI fallback ──
        let mut args = vec!["ps", "--format", "{{json .}}", "--no-trunc"];
        if show_all {
            args.push("-a");
        }
        let output = Self::run_docker(&args)?;
        let mut containers = Vec::new();
        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line) {
                // Labels come as a flat string: "key1=val1,key2=val2,..."
                let labels: std::collections::HashMap<String, String> = raw
                    .get("Labels")
                    .and_then(|v| v.as_str())
                    .map(parse_flat_labels)
                    .unwrap_or_default();

                let compose_project =
                    labels.get("com.docker.compose.project").cloned();
                let compose_service =
                    labels.get("com.docker.compose.service").cloned();

                containers.push(ContainerInfo {
                    id: raw_get(&raw, "ID"),
                    name: raw_get(&raw, "Names"),
                    image: raw_get(&raw, "Image"),
                    status: raw_get(&raw, "Status"),
                    state: raw_get(&raw, "State"),
                    ports: vec![],
                    cpu_percentage: 0.0,
                    memory_usage_bytes: 0,
                    memory_limit_bytes: 0,
                    network_rx_bytes: 0,
                    network_tx_bytes: 0,
                    pid: raw_get(&raw, "PIDs").parse().unwrap_or(0),
                    restart_count: 0,
                    created_at: raw_get(&raw, "CreatedAt").parse().unwrap_or(0),
                    started_at: None,
                    labels,
                    compose_project,
                    compose_service,
                });
            }
        }
        Ok(containers)
    }

    async fn inspect_container(&self, id: &str) -> Result<serde_json::Value, String> {
        if let Some(client) = &self.rest_client {
            let result = client
                .get_json_value(&format!("/containers/{}/json", id))
                .await;
            match result {
                Ok(val) => return Ok(val),
                Err(_) => { /* fall through to CLI */ }
            }
        }
        let output = Self::run_docker(&["inspect", id])?;
        let vals: Vec<serde_json::Value> =
            serde_json::from_str(&output).map_err(|e| format!("Parse: {}", e))?;
        vals.into_iter().next().ok_or("Container not found".into())
    }

    // ─── Container Lifecycle (writes — CLI only) ─────────────────────────

    async fn start_container(&self, id: &str) -> Result<(), String> {
        Self::run_docker(&["start", id])?;
        Ok(())
    }

    async fn stop_container(&self, id: &str) -> Result<(), String> {
        Self::run_docker(&["stop", id])?;
        Ok(())
    }

    async fn restart_container(&self, id: &str) -> Result<(), String> {
        Self::run_docker(&["restart", id])?;
        Ok(())
    }

    async fn pause_container(&self, id: &str) -> Result<(), String> {
        Self::run_docker(&["pause", id])?;
        Ok(())
    }

    async fn unpause_container(&self, id: &str) -> Result<(), String> {
        Self::run_docker(&["unpause", id])?;
        Ok(())
    }

    async fn kill_container(&self, id: &str, signal: &str) -> Result<(), String> {
        if signal.is_empty() {
            Self::run_docker(&["kill", id])?;
        } else {
            Self::run_docker(&["kill", "-s", signal, id])?;
        }
        Ok(())
    }

    async fn rename_container(&self, id: &str, new_name: &str) -> Result<(), String> {
        Self::run_docker(&["rename", id, new_name])?;
        Ok(())
    }

    async fn remove_container(
        &self,
        id: &str,
        force: bool,
        remove_volumes: bool,
    ) -> Result<(), String> {
        let mut args = vec!["rm"];
        if force {
            args.push("-f");
        }
        if remove_volumes {
            args.push("-v");
        }
        args.push(id);
        Self::run_docker(&args)?;
        Ok(())
    }

    async fn get_container_stats(&self, id: &str) -> Result<ContainerStats, String> {
        if let Some(client) = &self.rest_client {
            match client
                .get_json_value(&format!("/containers/{}/stats?stream=false", id))
                .await
            {
                Ok(raw) => {
                    // ── CPU percentage ──
                    let cpu_percentage = {
                        let cpu_stats = raw.get("cpu_stats");
                        let precpu_stats = raw.get("precpu_stats");
                        if let (Some(cur), Some(pre)) = (cpu_stats, precpu_stats) {
                            let cur_usage = cur
                                .pointer("/cpu_usage/total_usage")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let pre_usage = pre
                                .pointer("/cpu_usage/total_usage")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let cur_system = cur
                                .get("system_cpu_usage")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let pre_system = pre
                                .get("system_cpu_usage")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0);
                            let online_cpus =
                                cur.get("online_cpus").and_then(|v| v.as_u64()).unwrap_or(1);

                            let cpu_delta = cur_usage.saturating_sub(pre_usage);
                            let system_delta = cur_system.saturating_sub(pre_system);

                            if system_delta > 0 && cpu_delta > 0 {
                                (cpu_delta as f64 / system_delta as f64)
                                    * online_cpus as f64
                                    * 100.0
                            } else {
                                0.0
                            }
                        } else {
                            0.0
                        }
                    };

                    // ── Memory ──
                    let memory_usage_bytes = raw
                        .pointer("/memory_stats/usage")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let memory_limit_bytes = raw
                        .pointer("/memory_stats/limit")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let memory_percentage = if memory_limit_bytes > 0 {
                        (memory_usage_bytes as f64 / memory_limit_bytes as f64) * 100.0
                    } else {
                        0.0
                    };

                    // ── Network ──
                    let net_rx: u64 = raw
                        .get("networks")
                        .and_then(|n| n.as_object())
                        .map(|obj| {
                            obj.values()
                                .filter_map(|iface| iface.get("rx_bytes").and_then(|v| v.as_u64()))
                                .sum()
                        })
                        .unwrap_or(0);
                    let net_tx: u64 = raw
                        .get("networks")
                        .and_then(|n| n.as_object())
                        .map(|obj| {
                            obj.values()
                                .filter_map(|iface| iface.get("tx_bytes").and_then(|v| v.as_u64()))
                                .sum()
                        })
                        .unwrap_or(0);

                    // ── Block I/O ──
                    let block_read: u64 = raw
                        .pointer("/blkio_stats/io_service_bytes_recursive")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter(|entry| {
                                    entry
                                        .get("op")
                                        .and_then(|o| o.as_str())
                                        .map(|o| o == "read")
                                        .unwrap_or(false)
                                })
                                .filter_map(|entry| entry.get("value").and_then(|v| v.as_u64()))
                                .sum()
                        })
                        .unwrap_or(0);
                    let block_write: u64 = raw
                        .pointer("/blkio_stats/io_service_bytes_recursive")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter(|entry| {
                                    entry
                                        .get("op")
                                        .and_then(|o| o.as_str())
                                        .map(|o| o == "write")
                                        .unwrap_or(false)
                                })
                                .filter_map(|entry| entry.get("value").and_then(|v| v.as_u64()))
                                .sum()
                        })
                        .unwrap_or(0);

                    // ── PIDs ──
                    let pids = raw
                        .pointer("/pids_stats/current")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32;

                    return Ok(ContainerStats {
                        container_id: id.to_string(),
                        cpu_percentage,
                        memory_usage_bytes,
                        memory_limit_bytes,
                        memory_percentage,
                        network_rx_bytes: net_rx,
                        network_tx_bytes: net_tx,
                        block_read_bytes: block_read,
                        block_write_bytes: block_write,
                        pids,
                        timestamp: chrono::Utc::now().timestamp(),
                    });
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }

        // ── CLI fallback ──
        let output = Self::run_docker(&["stats", "--no-stream", "--format", "{{json .}}", id])?;
        let raw: serde_json::Value =
            serde_json::from_str(output.trim()).map_err(|e| format!("Parse: {}", e))?;
        Ok(ContainerStats {
            container_id: raw_get(&raw, "ID"),
            cpu_percentage: raw_get(&raw, "CPUPerc")
                .trim_end_matches('%')
                .parse()
                .unwrap_or(0.0),
            memory_usage_bytes: parse_mem(&raw_get(&raw, "MemUsage")),
            memory_limit_bytes: 0,
            memory_percentage: raw_get(&raw, "MemPerc")
                .trim_end_matches('%')
                .parse()
                .unwrap_or(0.0),
            network_rx_bytes: parse_net(&raw_get(&raw, "NetIO"), true),
            network_tx_bytes: parse_net(&raw_get(&raw, "NetIO"), false),
            block_read_bytes: 0,
            block_write_bytes: 0,
            pids: raw_get(&raw, "PIDs").parse().unwrap_or(0),
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    async fn create_and_run_container(
        &self,
        image: &str,
        name: Option<&str>,
        ports: Vec<PortMapping>,
        volumes: Vec<String>,
        env_vars: Vec<String>,
        network: Option<&str>,
        restart_policy: Option<&str>,
        command: Option<Vec<String>>,
        detach: bool,
        cpu_limit: Option<f64>,
        memory_limit: Option<String>,
        privileged: bool,
    ) -> Result<String, String> {
        let mut args: Vec<String> = vec!["run".to_string()];
        if detach {
            args.push("-d".into());
        }
        if let Some(n) = name {
            args.push("--name".into());
            args.push(n.to_string());
        }
        if privileged {
            args.push("--privileged".into());
        }
        if let Some(cpu) = cpu_limit {
            args.push("--cpus".into());
            args.push(cpu.to_string());
        }
        if let Some(mem) = memory_limit {
            args.push("--memory".into());
            args.push(mem.to_string());
        }
        for p in &ports {
            args.push("-p".into());
            args.push(format!(
                "{}:{}:{}/{}",
                p.host_ip, p.host_port, p.container_port, p.protocol
            ));
        }
        for v in &volumes {
            args.push("-v".into());
            args.push(v.clone());
        }
        for e in &env_vars {
            args.push("-e".into());
            args.push(e.clone());
        }
        if let Some(net) = network {
            args.push("--network".into());
            args.push(net.to_string());
        }
        if let Some(rp) = restart_policy {
            args.push("--restart".into());
            args.push(rp.to_string());
        }
        args.push(image.to_string());
        if let Some(cmd) = &command {
            for c in cmd {
                args.push(c.clone());
            }
        }
        let refs: Vec<&str> = args.iter().map(|s| s.as_ref()).collect();
        Self::run_docker(&refs)
    }

    // ─── File Explorer ─────────────────────────────────────────────────────

    async fn list_container_dir(&self, cid: &str, path: &str) -> Result<Vec<FileMetadata>, String> {
        let output = Self::run_docker(&["exec", cid, "ls", "-la", "--time-style=+%s", path])?;
        let files: Vec<FileMetadata> = output
            .lines()
            .skip(1) // skip the "total N" line
            .filter_map(|line| parse_ls_line(line, path))
            .collect();
        Ok(files)
    }

    async fn download_file_from_container(
        &self,
        cid: &str,
        remote: &str,
        local: PathBuf,
    ) -> Result<(), String> {
        Self::run_docker(&[
            "cp",
            &format!("{}:{}", cid, remote),
            local.to_str().unwrap_or("/tmp/out"),
        ])?;
        Ok(())
    }

    async fn upload_file_to_container(
        &self,
        cid: &str,
        local: PathBuf,
        remote: &str,
    ) -> Result<(), String> {
        Self::run_docker(&[
            "cp",
            local.to_str().unwrap_or("/tmp/in"),
            &format!("{}:{}", cid, remote),
        ])?;
        Ok(())
    }

    async fn read_file_preview(
        &self,
        cid: &str,
        remote: &str,
        max: usize,
    ) -> Result<String, String> {
        let output = Self::run_docker(&["exec", cid, "head", "-c", &max.to_string(), remote])?;
        Ok(output)
    }

    // ─── Images ──────────────────────────────────────────────────────────

    async fn list_images(&self) -> Result<Vec<ImageInfo>, String> {
        if let Some(client) = &self.rest_client {
            match client.get_json_value("/images/json").await {
                Ok(val) => {
                    if let Some(arr) = val.as_array() {
                        let mut images = Vec::with_capacity(arr.len());
                        for raw in arr {
                            let id = raw
                                .get("Id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            // RepoTags is an array of "repo:tag" strings.
                            let (repository, tag) = raw
                                .get("RepoTags")
                                .and_then(|t| t.as_array())
                                .and_then(|tags| tags.first())
                                .and_then(|t| t.as_str())
                                .map(split_repo_tag)
                                .unwrap_or_default();

                            let size_bytes = raw.get("Size").and_then(|v| v.as_u64()).unwrap_or(0);

                            let created_at =
                                raw.get("Created").and_then(|v| v.as_i64()).unwrap_or(0);

                            images.push(ImageInfo {
                                id,
                                repository,
                                tag,
                                size_bytes,
                                created_at,
                                architecture: "amd64".into(),
                                os: "linux".into(),
                                layer_count: 0,
                            });
                        }
                        return Ok(images);
                    }
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }

        // ── CLI fallback ──
        let output = Self::run_docker(&["images", "--format", "{{json .}}"])?;
        let mut images = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                images.push(ImageInfo {
                    id: raw_get(&raw, "ID"),
                    repository: raw_get(&raw, "Repository"),
                    tag: raw_get(&raw, "Tag"),
                    size_bytes: parse_size(&raw_get(&raw, "Size")),
                    created_at: raw_get(&raw, "CreatedAt").parse().unwrap_or(0),
                    architecture: "amd64".into(),
                    os: "linux".into(),
                    layer_count: 0,
                });
            }
        }
        Ok(images)
    }

    async fn inspect_image(&self, id: &str) -> Result<serde_json::Value, String> {
        if let Some(client) = &self.rest_client {
            match client.get_json_value(&format!("/images/{}/json", id)).await {
                Ok(val) => return Ok(val),
                Err(_) => { /* fall through to CLI */ }
            }
        }
        let output = Self::run_docker(&["image", "inspect", id])?;
        let vals: Vec<serde_json::Value> =
            serde_json::from_str(&output).map_err(|e| format!("Parse: {}", e))?;
        vals.into_iter().next().ok_or("Image not found".into())
    }

    async fn get_image_history(&self, id: &str) -> Result<Vec<ImageLayerInfo>, String> {
        if let Some(client) = &self.rest_client {
            match client
                .get_json_value(&format!("/images/{}/history", id))
                .await
            {
                Ok(val) => {
                    if let Some(arr) = val.as_array() {
                        let mut layers = Vec::with_capacity(arr.len());
                        for raw in arr {
                            let digest = raw
                                .get("Id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let size_bytes = raw
                                .get("Size")
                                .and_then(|v| v.as_i64())
                                .map(|v| v.max(0) as u64)
                                .unwrap_or(0);

                            let command = raw
                                .get("CreatedBy")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let created_at =
                                raw.get("Created").and_then(|v| v.as_i64()).unwrap_or(0);

                            layers.push(ImageLayerInfo {
                                digest,
                                size_bytes,
                                command,
                                created_at,
                            });
                        }
                        return Ok(layers);
                    }
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }

        // ── CLI fallback ──
        let output = Self::run_docker(&["history", "--format", "{{json .}}", "--no-trunc", id])?;
        let mut layers = Vec::new();
        for line in output.lines() {
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                layers.push(ImageLayerInfo {
                    digest: raw_get(&raw, "ID"),
                    size_bytes: parse_size(&raw_get(&raw, "Size")),
                    command: raw_get(&raw, "CreatedBy"),
                    created_at: 0,
                });
            }
        }
        Ok(layers)
    }

    // ─── Image Lifecycle (writes — CLI only) ─────────────────────────────

    async fn pull_image(&self, image_name: &str) -> Result<(), String> {
        Self::run_docker(&["pull", image_name])?;
        Ok(())
    }

    async fn remove_image(&self, id: &str, force: bool) -> Result<(), String> {
        let mut args = vec!["rmi"];
        if force {
            args.push("-f");
        }
        args.push(id);
        Self::run_docker(&args)?;
        Ok(())
    }

    async fn tag_image(&self, id: &str, repo: &str, tag: &str) -> Result<(), String> {
        Self::run_docker(&["tag", id, &format!("{}:{}", repo, tag)])?;
        Ok(())
    }

    // ─── Volumes ─────────────────────────────────────────────────────────

    async fn list_volumes(&self) -> Result<Vec<VolumeInfo>, String> {
        if let Some(client) = &self.rest_client {
            match client.get_json_value("/volumes").await {
                Ok(val) => {
                    if let Some(arr) = val.get("Volumes").and_then(|v| v.as_array()) {
                        let mut volumes = Vec::with_capacity(arr.len());
                        for raw in arr {
                            let name = raw
                                .get("Name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let driver = raw
                                .get("Driver")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let mountpoint = raw
                                .get("Mountpoint")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let size_bytes = raw
                                .get("UsageData")
                                .and_then(|u| u.get("Size"))
                                .and_then(|v| v.as_u64());

                            let labels = raw
                                .get("Labels")
                                .and_then(|l| l.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .map(|(k, v)| {
                                            (k.clone(), v.as_str().unwrap_or("").to_string())
                                        })
                                        .collect::<std::collections::HashMap<String, String>>()
                                })
                                .unwrap_or_default();

                            volumes.push(VolumeInfo {
                                name,
                                driver,
                                size_bytes,
                                mountpoint,
                                labels,
                                used_by: vec![],
                            });
                        }
                        return Ok(volumes);
                    }
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }

        // ── CLI fallback ──
        let output = Self::run_docker(&["volume", "ls", "--format", "{{json .}}"])?;
        let mut volumes = Vec::new();
        for line in output.lines() {
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                volumes.push(VolumeInfo {
                    name: raw_get(&raw, "Name"),
                    driver: raw_get(&raw, "Driver"),
                    size_bytes: None,
                    mountpoint: raw_get(&raw, "Mountpoint"),
                    labels: std::collections::HashMap::new(),
                    used_by: vec![],
                });
            }
        }
        Ok(volumes)
    }

    async fn create_volume(&self, name: &str, driver: Option<&str>) -> Result<(), String> {
        let mut args = vec!["volume", "create"];
        if let Some(d) = driver {
            args.push("-d");
            args.push(d);
        }
        args.push(name);
        Self::run_docker(&args)?;
        Ok(())
    }

    async fn remove_volume(&self, name: &str, force: bool) -> Result<(), String> {
        let mut args = vec!["volume", "rm"];
        if force {
            args.push("-f");
        }
        args.push(name);
        Self::run_docker(&args)?;
        Ok(())
    }

    // ─── Networks ────────────────────────────────────────────────────────

    async fn list_networks(&self) -> Result<Vec<NetworkInfo>, String> {
        if let Some(client) = &self.rest_client {
            match client.get_json_value("/networks").await {
                Ok(val) => {
                    if let Some(arr) = val.as_array() {
                        let mut networks = Vec::with_capacity(arr.len());
                        for raw in arr {
                            let id = raw
                                .get("Id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let name = raw
                                .get("Name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let driver = raw
                                .get("Driver")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let scope = raw
                                .get("Scope")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();

                            let internal = raw
                                .get("Internal")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            // Subnet / gateway from IPAM config
                            let (subnet, gateway) = raw
                                .pointer("/IPAM/Config")
                                .and_then(|c| c.as_array())
                                .and_then(|configs| configs.first())
                                .map(|cfg| {
                                    let s = cfg
                                        .get("Subnet")
                                        .and_then(|v| v.as_str())
                                        .map(String::from);
                                    let g = cfg
                                        .get("Gateway")
                                        .and_then(|v| v.as_str())
                                        .map(String::from);
                                    (s, g)
                                })
                                .unwrap_or((None, None));

                            // Connected containers
                            let containers = raw
                                .get("Containers")
                                .and_then(|c| c.as_object())
                                .map(|obj| {
                                    obj.iter()
                                        .map(|(cid, info)| NetworkContainer {
                                            container_id: cid.clone(),
                                            container_name: info
                                                .get("Name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                            ipv4_address: info
                                                .get("IPv4Address")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                            mac_address: info
                                                .get("MacAddress")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();

                            networks.push(NetworkInfo {
                                id,
                                name,
                                driver,
                                scope,
                                subnet,
                                gateway,
                                internal,
                                containers,
                            });
                        }
                        return Ok(networks);
                    }
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }

        // ── CLI fallback ──
        let output = Self::run_docker(&["network", "ls", "--format", "{{json .}}"])?;
        let mut networks = Vec::new();
        for line in output.lines() {
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                networks.push(NetworkInfo {
                    id: raw_get(&raw, "ID"),
                    name: raw_get(&raw, "Name"),
                    driver: raw_get(&raw, "Driver"),
                    scope: raw_get(&raw, "Scope"),
                    subnet: None,
                    gateway: None,
                    internal: false,
                    containers: vec![],
                });
            }
        }
        Ok(networks)
    }

    async fn create_network(
        &self,
        name: &str,
        driver: &str,
        subnet: Option<&str>,
        gateway: Option<&str>,
    ) -> Result<(), String> {
        let mut args = vec!["network", "create", "-d", driver];
        if let Some(s) = subnet {
            args.push("--subnet");
            args.push(s);
        }
        if let Some(g) = gateway {
            args.push("--gateway");
            args.push(g);
        }
        args.push(name);
        Self::run_docker(&args)?;
        Ok(())
    }

    async fn remove_network(&self, id: &str) -> Result<(), String> {
        Self::run_docker(&["network", "rm", id])?;
        Ok(())
    }

    // ─── Cleanup ─────────────────────────────────────────────────────────

    async fn get_disk_usage(&self) -> Result<DiskUsageSummary, String> {
        if let Some(client) = &self.rest_client {
            match client.get_json_value("/system/df").await {
                Ok(val) => {
                    let containers = parse_df_array(&val, "Containers", "SizeRw");
                    let images = parse_df_array(&val, "Images", "Size");
                    let build_cache = parse_df_array(&val, "BuildCache", "Size");

                    // Volumes use nested UsageData.Size
                    let (volumes_count, volumes_size) = val
                        .get("Volumes")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            let count = arr.len();
                            let size: u64 = arr
                                .iter()
                                .filter_map(|item| {
                                    item.get("UsageData")
                                        .and_then(|u| u.get("Size"))
                                        .and_then(|s| s.as_u64())
                                })
                                .sum();
                            (count, size)
                        })
                        .unwrap_or((0, 0));

                    return Ok(DiskUsageSummary {
                        containers_count: containers.count,
                        containers_size_bytes: containers.size,
                        images_count: images.count,
                        images_size_bytes: images.size,
                        volumes_count,
                        volumes_size_bytes: volumes_size,
                        build_cache_size_bytes: build_cache.size,
                        total_reclaimable_bytes: containers.size
                            + images.size
                            + volumes_size
                            + build_cache.size,
                    });
                }
                Err(_) => { /* fall through to CLI */ }
            }
        }

        // ── CLI fallback ──
        let output = Self::run_docker(&["system", "df", "--format", "{{json .}}"])?;
        let mut summary = DiskUsageSummary {
            containers_count: 0,
            containers_size_bytes: 0,
            images_count: 0,
            images_size_bytes: 0,
            volumes_count: 0,
            volumes_size_bytes: 0,
            build_cache_size_bytes: 0,
            total_reclaimable_bytes: 0,
        };
        for line in output.lines() {
            if let Ok(raw) = serde_json::from_str::<serde_json::Value>(line.trim()) {
                let stype = raw_get(&raw, "Type");
                let size = parse_size(&raw_get(&raw, "Size"));
                let reclaimable = parse_size(&raw_get(&raw, "Reclaimable"));
                let count: usize = raw_get(&raw, "TotalCount").parse().unwrap_or(0);
                match stype.as_str() {
                    "Containers" => {
                        summary.containers_count = count;
                        summary.containers_size_bytes = size;
                    }
                    "Images" => {
                        summary.images_count = count;
                        summary.images_size_bytes = size;
                    }
                    "Local Volumes" => {
                        summary.volumes_count = count;
                        summary.volumes_size_bytes = size;
                    }
                    "Build Cache" => {
                        summary.build_cache_size_bytes = size;
                    }
                    _ => {}
                }
                summary.total_reclaimable_bytes += reclaimable;
            }
        }
        Ok(summary)
    }

    async fn prune_containers(&self) -> Result<u64, String> {
        let output = Self::run_docker(&["container", "prune", "-f"])?;
        Ok(parse_size(&output))
    }

    async fn prune_images(&self, dangling_only: bool) -> Result<u64, String> {
        let mut args = vec!["image", "prune", "-f"];
        if !dangling_only {
            args.push("-a");
        }
        let output = Self::run_docker(&args)?;
        Ok(parse_size(&output))
    }

    async fn prune_volumes(&self) -> Result<u64, String> {
        let output = Self::run_docker(&["volume", "prune", "-f"])?;
        Ok(parse_size(&output))
    }

    async fn prune_networks(&self) -> Result<u64, String> {
        let output = Self::run_docker(&["network", "prune", "-f"])?;
        Ok(parse_size(&output))
    }

    async fn prune_build_cache(&self) -> Result<u64, String> {
        let output = Self::run_docker(&["builder", "prune", "-f"])?;
        Ok(parse_size(&output))
    }

    async fn system_prune_all(&self) -> Result<DiskUsageSummary, String> {
        Self::run_docker(&["system", "prune", "-f"])?;
        self.get_disk_usage().await
    }

    // ─── Host Metrics ────────────────────────────────────────────────────

    async fn get_host_metrics(&self) -> Result<HostMetrics, String> {
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        sys.refresh_cpu_all();
        let cpu = sys.global_cpu_usage() as f64;
        Ok(HostMetrics {
            cpu_usage_percent: cpu,
            cpu_cores: sys.cpus().len(),
            cpu_per_core: sys.cpus().iter().map(|c| c.cpu_usage() as f64).collect(),
            memory_used_bytes: sys.used_memory(),
            memory_total_bytes: sys.total_memory(),
            swap_used_bytes: sys.used_swap(),
            swap_total_bytes: sys.total_swap(),
            disk_used_bytes: 0,
            disk_total_bytes: 0,
            uptime_seconds: System::uptime(),
            hostname: System::host_name().unwrap_or_default(),
            os_name: System::name().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
//  Helper Functions
// ═══════════════════════════════════════════════════════════════════════════

/// Extract a string field from a `serde_json::Value` (CLI format).
fn raw_get(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .map(|v| v.as_str().unwrap_or("").to_string())
        .unwrap_or_default()
}

/// Parse a flat comma-separated label string from `docker ps --format '{{json .}}'`.
///
/// Docker CLI template output serialises the Labels map as:
/// ```text
/// "com.docker.compose.project=myapp,com.docker.compose.service=web,maintainer=..."
/// ```
/// This function splits on `,` and then on the first `=` to reconstruct a HashMap.
fn parse_flat_labels(label_str: &str) -> std::collections::HashMap<String, String> {
    let mut labels = std::collections::HashMap::new();
    for part in label_str.split(',') {
        if let Some((key, value)) = part.split_once('=') {
            labels.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    labels
}

/// Parse a human-readable size string like "1.5GB", "500MB", "1KB", "1234".
fn parse_size(s: &str) -> u64 {
    let s = s.trim().to_uppercase();
    if s.is_empty() {
        return 0;
    }
    let num: f64 = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .parse()
        .unwrap_or(0.0);
    if s.ends_with("GB") || s.ends_with("G") {
        return (num * 1_000_000_000.0) as u64;
    }
    if s.ends_with("MB") || s.ends_with("M") {
        return (num * 1_000_000.0) as u64;
    }
    if s.ends_with("KB") || s.ends_with("K") {
        return (num * 1_000.0) as u64;
    }
    if s.ends_with("B") {
        return num as u64;
    }
    s.parse().unwrap_or(0)
}

/// Parse a memory usage string like "45MiB / 512MiB".
fn parse_mem(s: &str) -> u64 {
    s.split('/')
        .next()
        .unwrap_or("0")
        .trim()
        .replace("MiB", "")
        .replace("GiB", "000")
        .parse::<f64>()
        .map(|v| (v * 1_048_576.0) as u64)
        .unwrap_or(0)
}

/// Parse a network I/O string like "1.5kB / 800B".  `rx=true` reads the
/// left half, `rx=false` reads the right half.
fn parse_net(s: &str, rx: bool) -> u64 {
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() < 2 {
        return 0;
    }
    let target = if rx { parts[0] } else { parts[1] };
    parse_size(target.trim())
}

/// Split a Docker repo-tag string (e.g. "nginx:latest") into (repo, tag).
/// Handles edge cases like "ubuntu:22.04" by splitting on the last colon.
fn split_repo_tag(s: &str) -> (String, String) {
    if let Some(last_colon) = s.rfind(':') {
        let repo = &s[..last_colon];
        let tag = &s[last_colon + 1..];
        (repo.to_string(), tag.to_string())
    } else {
        (s.to_string(), "latest".to_string())
    }
}

/// Result of parsing one resource type from `/system/df`.
struct DfParseResult {
    count: usize,
    size: u64,
}

/// Parse a named array from the Docker `/system/df` response and sum the
/// given size field from each element.
fn parse_df_array(root: &serde_json::Value, array_key: &str, size_field: &str) -> DfParseResult {
    match root.get(array_key).and_then(|a| a.as_array()) {
        Some(arr) => {
            let count = arr.len();
            let size: u64 = arr
                .iter()
                .filter_map(|item| item.get(size_field).and_then(|v| v.as_u64()))
                .sum();
            DfParseResult { count, size }
        }
        None => DfParseResult { count: 0, size: 0 },
    }
}

// ─── File Explorer Helpers ──────────────────────────────────────────────

/// Parse a single line from `ls -la --time-style=+%s` output into a
/// `FileMetadata` entry.  Returns `None` for lines that cannot be parsed
/// (e.g. the "total N" header, malformed output).
///
/// Expected columns (whitespace-separated):
///   0: permissions   (e.g. "-rw-r--r--", "drwxr-xr-x", "lrwxrwxrwx")
///   1: link count
///   2: owner
///   3: group
///   4: size in bytes
///   5: epoch timestamp (from --time-style=+%s)
///   6..: file name (may contain spaces)
///
/// For symlinks, `ls` outputs "name -> target" — we strip the " -> target"
/// suffix so the name is just the symlink name.
fn parse_ls_line(line: &str, base_path: &str) -> Option<FileMetadata> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 7 {
        return None;
    }
    let raw_name = parts[6..].join(" ");
    // Strip " -> target" suffix from symlinks
    let name = match raw_name.find(" -> ") {
        Some(pos) => raw_name[..pos].to_string(),
        None => raw_name,
    };
    Some(FileMetadata {
        name: name.clone(),
        full_path: format!("{}/{}", base_path.trim_end_matches('/'), name),
        is_dir: parts[0].starts_with('d'),
        is_symlink: parts[0].starts_with('l'),
        size_bytes: parts[4].parse().unwrap_or(0),
        permissions: parts[0].to_string(),
        owner: parts[2].to_string(),
        group: parts[3].to_string(),
        updated_at: parts[5].parse().unwrap_or(0),
    })
}

// ═══════════════════════════════════════════════════════════════════════════
//  Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_regular_file() {
        let line = "-rw-r--r--  1 root root    532 1728000002 config.json";
        let meta = parse_ls_line(line, "/app").expect("should parse");

        assert!(!meta.is_dir);
        assert!(!meta.is_symlink);
        assert_eq!(meta.name, "config.json");
        assert_eq!(meta.full_path, "/app/config.json");
        assert_eq!(meta.size_bytes, 532);
        assert_eq!(meta.permissions, "-rw-r--r--");
        assert_eq!(meta.owner, "root");
        assert_eq!(meta.group, "root");
        assert_eq!(meta.updated_at, 1728000002);
    }

    #[test]
    fn test_parse_directory() {
        let line = "drwxr-xr-x  2 dotnet dotnet  4096 1728000003 data";
        let meta = parse_ls_line(line, "/app").expect("should parse");

        assert!(meta.is_dir);
        assert!(!meta.is_symlink);
        assert_eq!(meta.name, "data");
        assert_eq!(meta.full_path, "/app/data");
        assert_eq!(meta.size_bytes, 4096);
        assert_eq!(meta.owner, "dotnet");
        assert_eq!(meta.group, "dotnet");
    }

    #[test]
    fn test_parse_symlink() {
        // Symlink: name should NOT include the " -> target" suffix
        let line = "lrwxrwxrwx  1 root root      7 1728000000 bin -> usr/bin";
        let meta = parse_ls_line(line, "/").expect("should parse");

        assert!(meta.is_symlink);
        assert!(!meta.is_dir);
        assert_eq!(meta.name, "bin", "symlink name must strip ' -> target'");
        assert_eq!(meta.full_path, "/bin");
        assert_eq!(meta.size_bytes, 7);
        assert_eq!(meta.permissions, "lrwxrwxrwx");
    }

    #[test]
    fn test_parse_file_with_spaces() {
        let line = "-rw-r--r--  1 root root   123 1728000000 my file.txt";
        let meta = parse_ls_line(line, "/").expect("should parse");

        assert_eq!(meta.name, "my file.txt");
        assert_eq!(meta.full_path, "/my file.txt");
    }

    #[test]
    fn test_parse_hidden_file() {
        let line = "-rw-r--r--  1 root root     0 1728000000 .dockerignore";
        let meta = parse_ls_line(line, "/build").expect("should parse");

        assert_eq!(meta.name, ".dockerignore");
        assert_eq!(meta.full_path, "/build/.dockerignore");
    }

    #[test]
    fn test_parse_large_file_size() {
        let line = "-rwxr-xr-x  1 user group 9876543210 1728000000 bigfile.bin";
        let meta = parse_ls_line(line, "/data").expect("should parse");

        assert_eq!(meta.size_bytes, 9876543210);
        assert_eq!(meta.updated_at, 1728000000);
    }

    #[test]
    fn test_parse_total_line_returns_none() {
        let line = "total 16944";
        assert!(parse_ls_line(line, "/").is_none(), "'total' line must be rejected");
    }

    #[test]
    fn test_parse_current_dir() {
        let line = "drwxr-xr-x  1 root root 4096 1728000000 .";
        let meta = parse_ls_line(line, "/app").expect("should parse");

        assert!(meta.is_dir);
        assert_eq!(meta.name, ".");
        assert_eq!(meta.full_path, "/app/.");
    }

    #[test]
    fn test_parse_parent_dir() {
        let line = "drwxr-xr-x  1 root root 4096 1728000001 ..";
        let meta = parse_ls_line(line, "/app").expect("should parse");

        assert!(meta.is_dir);
        assert_eq!(meta.name, "..");
        assert_eq!(meta.full_path, "/app/..");
    }

    #[test]
    fn test_parse_file_with_root_path() {
        let line = "drwxr-xr-x  1 root root 4096 1728000000 app";
        let meta = parse_ls_line(line, "/").expect("should parse");

        assert_eq!(meta.full_path, "/app");
    }

    #[test]
    fn test_parse_nonroot_owner() {
        let line = "-rw-rw-r--  1 dotnet dotnet   632 1781105833 appsettings.json";
        let meta = parse_ls_line(line, "/app").expect("should parse");

        assert_eq!(meta.owner, "dotnet", "owner must be just the user, not user:group");
        assert_eq!(meta.group, "dotnet");
    }
}
