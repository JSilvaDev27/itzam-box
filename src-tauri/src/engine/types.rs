// ItzamBox — Engine Data Types
// Copyright (C) 2026 SodigTech — GPL-3.0

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: Vec<PortMapping>,
    pub cpu_percentage: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub pid: u32,
    pub restart_count: u32,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub labels: std::collections::HashMap<String, String>,
    pub compose_project: Option<String>,
    pub compose_service: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PortMapping {
    pub host_ip: String,
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageInfo {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size_bytes: u64,
    pub created_at: i64,
    pub architecture: String,
    pub os: String,
    pub layer_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageLayerInfo {
    pub digest: String,
    pub size_bytes: u64,
    pub command: String,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub size_bytes: Option<u64>,
    pub mountpoint: String,
    pub labels: std::collections::HashMap<String, String>,
    pub used_by: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub internal: bool,
    pub containers: Vec<NetworkContainer>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkContainer {
    pub container_id: String,
    pub container_name: String,
    pub ipv4_address: String,
    pub mac_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileMetadata {
    pub name: String,
    pub full_path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size_bytes: u64,
    pub permissions: String,
    pub owner: String,
    pub group: String,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerStats {
    pub container_id: String,
    pub cpu_percentage: f64,
    pub memory_usage_bytes: u64,
    pub memory_limit_bytes: u64,
    pub memory_percentage: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
    pub pids: u32,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HostMetrics {
    pub cpu_usage_percent: f64,
    pub cpu_cores: usize,
    pub cpu_per_core: Vec<f64>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub uptime_seconds: u64,
    pub hostname: String,
    pub os_name: String,
    pub kernel_version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DockerEvent {
    pub event_type: String,
    pub action: String,
    pub actor_id: String,
    pub actor_name: String,
    pub timestamp: i64,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiskUsageSummary {
    pub containers_count: usize,
    pub containers_size_bytes: u64,
    pub images_count: usize,
    pub images_size_bytes: u64,
    pub volumes_count: usize,
    pub volumes_size_bytes: u64,
    pub build_cache_size_bytes: u64,
    pub total_reclaimable_bytes: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VulnerabilityReport {
    pub image_name: String,
    pub scanned_at: i64,
    pub total: usize,
    pub critical: Vec<Vulnerability>,
    pub high: Vec<Vulnerability>,
    pub medium: Vec<Vulnerability>,
    pub low: Vec<Vulnerability>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Vulnerability {
    pub id: String,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: String,
}
