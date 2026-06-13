// ItzamBox — Docker Linux Engine (CLI implementation for MVP)
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::traits::ContainerEngine;
use crate::engine::types::*;
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;

pub struct DockerLinuxEngine;

impl DockerLinuxEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    fn is_available() -> bool {
        std::path::Path::new("/var/run/docker.sock").exists()
    }

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

    #[allow(dead_code)]
    fn parse_docker_inspect(output: &str, field: &str) -> String {
        // Simple JSON parsing for docker inspect
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

#[async_trait]
impl ContainerEngine for DockerLinuxEngine {
    // ─── Diagnostics ───
    async fn check_engine_status(&self) -> Result<bool, String> {
        Ok(Self::is_available())
    }

    async fn get_engine_version(&self) -> Result<String, String> {
        if !Self::is_available() {
            return Err("Docker daemon not available".into());
        }
        let output = Self::run_docker(&["version", "--format", "{{.Server.Version}}"])?;
        Ok(output.trim().to_string())
    }

    async fn get_engine_info(&self) -> Result<serde_json::Value, String> {
        let output = Self::run_docker(&["info", "--format", "{{json .}}"])?;
        serde_json::from_str(output.trim()).map_err(|e| format!("Parse error: {}", e))
    }

    // ─── Containers ───
    async fn list_containers(&self, show_all: bool) -> Result<Vec<ContainerInfo>, String> {
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
                    labels: std::collections::HashMap::new(),
                    compose_project: raw
                        .get("Labels")
                        .and_then(|l| l.get("com.docker.compose.project"))
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    compose_service: None,
                });
            }
        }
        Ok(containers)
    }

    async fn inspect_container(&self, id: &str) -> Result<serde_json::Value, String> {
        let output = Self::run_docker(&["inspect", id])?;
        let vals: Vec<serde_json::Value> =
            serde_json::from_str(&output).map_err(|e| format!("Parse: {}", e))?;
        vals.into_iter().next().ok_or("Container not found".into())
    }

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
    ) -> Result<String, String> {
        let mut args: Vec<String> = vec!["run".to_string()];
        if detach {
            args.push("-d".into());
        }
        if let Some(n) = name {
            args.push("--name".into());
            args.push(n.to_string());
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

    // ─── File Explorer (stubs — Phase 2) ───
    async fn list_container_dir(&self, cid: &str, path: &str) -> Result<Vec<FileMetadata>, String> {
        let output = Self::run_docker(&["exec", cid, "ls", "-la", "--time-style=+%s", path])?;
        let mut files = Vec::new();
        for line in output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 9 {
                files.push(FileMetadata {
                    name: parts[8..].join(" "),
                    full_path: format!("{}/{}", path.trim_end_matches('/'), parts[8..].join(" ")),
                    is_dir: parts[0].starts_with('d'),
                    is_symlink: parts[0].starts_with('l'),
                    size_bytes: parts[4].parse().unwrap_or(0),
                    permissions: parts[0].to_string(),
                    owner: format!("{}:{}", parts[2], parts[3]),
                    group: parts[3].to_string(),
                    updated_at: parts[5].parse().unwrap_or(0),
                });
            }
        }
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

    // ─── Images ───
    async fn list_images(&self) -> Result<Vec<ImageInfo>, String> {
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
        let output = Self::run_docker(&["image", "inspect", id])?;
        let vals: Vec<serde_json::Value> =
            serde_json::from_str(&output).map_err(|e| format!("Parse: {}", e))?;
        vals.into_iter().next().ok_or("Image not found".into())
    }
    async fn get_image_history(&self, id: &str) -> Result<Vec<ImageLayerInfo>, String> {
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

    // ─── Volumes ───
    async fn list_volumes(&self) -> Result<Vec<VolumeInfo>, String> {
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

    // ─── Networks ───
    async fn list_networks(&self) -> Result<Vec<NetworkInfo>, String> {
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

    // ─── Cleanup ───
    async fn get_disk_usage(&self) -> Result<DiskUsageSummary, String> {
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

    // ─── Host Metrics ───
    async fn get_host_metrics(&self) -> Result<HostMetrics, String> {
        use sysinfo::System;
        let mut sys = System::new_all();
        // sysinfo needs two measurements to calculate CPU usage.
        // First refresh gathers baseline, wait, then second refresh computes delta.
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

// ─── Helper Functions ───

fn raw_get(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .map(|v| v.as_str().unwrap_or("").to_string())
        .unwrap_or_default()
}

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

fn parse_mem(s: &str) -> u64 {
    // Format: "45MiB / 512MiB" or "45.2MiB / 512MiB"
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

fn parse_net(s: &str, rx: bool) -> u64 {
    // Format: "1.5kB / 800B"
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() < 2 {
        return 0;
    }
    let target = if rx { parts[0] } else { parts[1] };
    parse_size(target.trim())
}
