// ItzamBox — Engine Data Types
// Copyright (C) 2026 SodigTech — GPL-3.0

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PortMapping {
    pub host_ip: String,
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImageLayerInfo {
    pub digest: String,
    pub size_bytes: u64,
    pub command: String,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub size_bytes: Option<u64>,
    pub mountpoint: String,
    pub labels: std::collections::HashMap<String, String>,
    pub used_by: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NetworkContainer {
    pub container_id: String,
    pub container_name: String,
    pub ipv4_address: String,
    pub mac_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DockerEvent {
    pub event_type: String,
    pub action: String,
    pub actor_id: String,
    pub actor_name: String,
    pub timestamp: i64,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct VulnerabilityReport {
    pub image_name: String,
    pub scanned_at: i64,
    pub total: usize,
    pub critical: Vec<Vulnerability>,
    pub high: Vec<Vulnerability>,
    pub medium: Vec<Vulnerability>,
    pub low: Vec<Vulnerability>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Vulnerability {
    pub id: String,
    pub package: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: String,
}

// ─── Kubernetes Types ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KubectlStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub kubeconfig_parsed: bool,
    pub contexts_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sContext {
    pub name: String,
    pub cluster: String,
    pub user: String,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sPod {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub restarts: i32,
    pub age: String,
    pub node: String,
    pub ip: Option<String>,
    pub containers: Vec<K8sContainer>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sContainer {
    pub name: String,
    pub image: String,
    pub ports: Vec<String>,
    pub ready: bool,
    pub restart_count: i32,
    pub readiness_probe: Option<String>,
    pub liveness_probe: Option<String>,
    pub resources_limits: Option<HashMap<String, String>>,
    pub resources_requests: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sDeployment {
    pub name: String,
    pub namespace: String,
    pub ready: String,
    pub up_to_date: i32,
    pub available: i32,
    pub age: String,
    pub strategy: String,
    pub selector: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sService {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub external_ip: Option<String>,
    pub ports: Vec<String>,
    pub age: String,
    pub selector: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sConfigMap {
    pub name: String,
    pub namespace: String,
    pub keys_count: usize,
    pub age: String,
    pub data_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sSecretMeta {
    pub name: String,
    pub namespace: String,
    pub secret_type: String,
    pub keys_count: usize,
    pub age: String,
    pub data_keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct K8sEvent {
    pub timestamp: String,
    pub event_type: String,
    pub reason: String,
    pub message: String,
}

// ─── Swarm Types ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SwarmStatus {
    pub active: bool,
    pub node_id: Option<String>,
    pub nodes_count: usize,
    pub managers_count: usize,
    pub services_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SwarmNode {
    pub id: String,
    pub hostname: String,
    pub role: String,
    pub status: String,
    pub availability: String,
    pub engine_version: String,
    pub ip_address: String,
    pub cpu_cores: Option<i32>,
    pub memory_bytes: Option<i64>,
    pub labels: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SwarmService {
    pub id: String,
    pub name: String,
    pub mode: String,
    pub replicas: String,
    pub image: String,
    pub ports: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SwarmStack {
    pub name: String,
    pub services_count: usize,
    pub orchestrator: String,
}

// ─── Backup Types ─────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BackupSnapshot {
    pub id: i64,
    pub name: String,
    pub source_volume: String,
    pub destination_path: String,
    pub compressed_size_bytes: i64,
    pub original_size_bytes: i64,
    pub sha256_checksum: String,
    pub status: String, // Success, Failed, InProgress
    pub duration_seconds: Option<i32>,
    pub created_at: String, // ISO 8601
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BackupJob {
    pub id: i64,
    pub name: String,
    pub frequency: String, // hourly, daily, weekly, custom
    pub cron_expression: Option<String>,
    pub source_volumes: Vec<String>,
    pub destination_path: String,
    pub retention_count: i32,
    pub enabled: bool,
    pub next_run: Option<String>,
    pub last_run: Option<String>,
    pub last_status: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BackupJobConfig {
    pub name: String,
    pub frequency: String,
    pub cron_expression: Option<String>,
    pub source_volumes: Vec<String>,
    pub destination_path: String,
    pub retention_count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BackupProgress {
    pub job_id: String,
    pub bytes_processed: i64,
    pub total_bytes: i64,
    pub percentage: u8,
    pub status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BackupHistory {
    pub id: i64,
    pub job_id: Option<i64>,
    pub name: String,
    pub source_volume: String,
    pub destination_path: String,
    pub compressed_size_bytes: i64,
    pub original_size_bytes: i64,
    pub sha256_checksum: String,
    pub status: String,
    pub failure_reason: Option<String>,
    pub duration_seconds: Option<i32>,
    pub created_at: String,
}

// ─── Metrics / Time-Series Types ─────────────────────────────────────────

/// A single point from any metrics table (raw / 5-min / 30-min aggregates).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MetricsDataPoint {
    pub timestamp: i64,
    pub cpu_percent: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub sample_count: u32,
}

/// A per-container metrics point (raw or 5-min aggregate).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContainerMetricsPoint {
    pub timestamp: i64,
    pub container_id: String,
    pub cpu_percent: f64,
    pub memory_usage_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub block_read_bytes: u64,
    pub block_write_bytes: u64,
    pub sample_count: u32,
}

/// Describes a time range for metrics queries.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MetricRange {
    pub from: i64,
    pub to: i64,
    pub point_count: usize,
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: round-trip serialize then deserialize
    fn roundtrip<T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + PartialEq>(
        value: &T,
    ) -> T {
        let json = serde_json::to_string(value).expect("serialization failed");
        serde_json::from_str(&json).expect("deserialization failed")
    }

    #[test]
    fn test_container_info_roundtrip() {
        let info = ContainerInfo {
            id: "abc123def".into(),
            name: "my-nginx".into(),
            image: "nginx:latest".into(),
            status: "Up 2 hours".into(),
            state: "running".into(),
            ports: vec![PortMapping {
                host_ip: "0.0.0.0".into(),
                host_port: 8080,
                container_port: 80,
                protocol: "tcp".into(),
            }],
            cpu_percentage: 2.5,
            memory_usage_bytes: 50_000_000,
            memory_limit_bytes: 1_000_000_000,
            network_rx_bytes: 1024,
            network_tx_bytes: 2048,
            pid: 12345,
            restart_count: 0,
            created_at: 1718200000,
            started_at: Some(1718200100),
            labels: {
                let mut m = std::collections::HashMap::new();
                m.insert("env".into(), "production".into());
                m
            },
            compose_project: Some("webapp".into()),
            compose_service: Some("nginx".into()),
        };

        let restored = roundtrip(&info);
        assert_eq!(restored.id, info.id);
        assert_eq!(restored.name, "my-nginx");
        assert_eq!(restored.state, "running");
        assert_eq!(restored.ports.len(), 1);
        assert_eq!(restored.ports[0].host_port, 8080);
        assert_eq!(restored.compose_project, Some("webapp".into()));
    }

    #[test]
    fn test_image_info_roundtrip() {
        let info = ImageInfo {
            id: "sha256:abc123".into(),
            repository: "nginx".into(),
            tag: "latest".into(),
            size_bytes: 187_000_000,
            created_at: 1718200000,
            architecture: "amd64".into(),
            os: "linux".into(),
            layer_count: 7,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("nginx"));
        assert!(json.contains("latest"));
        assert!(json.contains("187000000"));

        let restored: ImageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.repository, "nginx");
        assert_eq!(restored.layer_count, 7);
    }

    #[test]
    fn test_volume_info_roundtrip() {
        let info = VolumeInfo {
            name: "my-data".into(),
            driver: "local".into(),
            size_bytes: Some(500_000_000),
            mountpoint: "/var/lib/docker/volumes/my-data/_data".into(),
            labels: std::collections::HashMap::new(),
            used_by: vec!["abc123".into(), "def456".into()],
        };

        let restored = roundtrip(&info);
        assert_eq!(restored.name, "my-data");
        assert_eq!(restored.used_by.len(), 2);
    }

    #[test]
    fn test_network_info_roundtrip() {
        let info = NetworkInfo {
            id: "net-001".into(),
            name: "bridge".into(),
            driver: "bridge".into(),
            scope: "local".into(),
            subnet: Some("172.17.0.0/16".into()),
            gateway: Some("172.17.0.1".into()),
            internal: false,
            containers: vec![NetworkContainer {
                container_id: "abc123".into(),
                container_name: "web-01".into(),
                ipv4_address: "172.17.0.2".into(),
                mac_address: "02:42:ac:11:00:02".into(),
            }],
        };

        let restored = roundtrip(&info);
        assert_eq!(restored.driver, "bridge");
        assert_eq!(restored.containers[0].container_name, "web-01");
    }

    #[test]
    fn test_host_metrics_roundtrip() {
        let metrics = HostMetrics {
            cpu_usage_percent: 45.2,
            cpu_cores: 8,
            cpu_per_core: vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
            memory_used_bytes: 8_000_000_000,
            memory_total_bytes: 16_000_000_000,
            swap_used_bytes: 1_000_000_000,
            swap_total_bytes: 4_000_000_000,
            disk_used_bytes: 100_000_000_000,
            disk_total_bytes: 500_000_000_000,
            uptime_seconds: 86400,
            hostname: "dev-machine".into(),
            os_name: "Ubuntu 24.04".into(),
            kernel_version: "6.8.0-31-generic".into(),
        };

        let restored = roundtrip(&metrics);
        assert_eq!(restored.hostname, "dev-machine");
        assert_eq!(restored.cpu_cores, 8);
        assert!((restored.cpu_usage_percent - 45.2).abs() < 0.01);
    }

    #[test]
    fn test_disk_usage_summary_roundtrip() {
        let disk = DiskUsageSummary {
            containers_count: 5,
            containers_size_bytes: 1_000_000,
            images_count: 12,
            images_size_bytes: 5_000_000_000,
            volumes_count: 3,
            volumes_size_bytes: 500_000_000,
            build_cache_size_bytes: 200_000_000,
            total_reclaimable_bytes: 2_500_000_000,
        };

        let restored = roundtrip(&disk);
        assert_eq!(restored.containers_count, 5);
        assert_eq!(restored.images_count, 12);
    }

    #[test]
    fn test_container_stats_roundtrip() {
        let stats = ContainerStats {
            container_id: "abc123".into(),
            cpu_percentage: 15.7,
            memory_usage_bytes: 100_000_000,
            memory_limit_bytes: 500_000_000,
            memory_percentage: 20.0,
            network_rx_bytes: 5000,
            network_tx_bytes: 3000,
            block_read_bytes: 1_000_000,
            block_write_bytes: 500_000,
            pids: 12,
            timestamp: 1718200000,
        };

        let restored = roundtrip(&stats);
        assert_eq!(restored.container_id, "abc123");
        assert!((restored.memory_percentage - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_vulnerability_report_roundtrip() {
        let report = VulnerabilityReport {
            image_name: "node:18-alpine".into(),
            scanned_at: 1718200000,
            total: 4,
            critical: vec![Vulnerability {
                id: "CVE-2024-1234".into(),
                package: "openssl".into(),
                installed_version: "1.1.1".into(),
                fixed_version: Some("1.1.2".into()),
                severity: "critical".into(),
                title: "Buffer overflow in OpenSSL".into(),
                description: "A buffer overflow vulnerability...".into(),
            }],
            high: vec![],
            medium: vec![],
            low: vec![],
        };

        let restored = roundtrip(&report);
        assert_eq!(restored.total, 4);
        assert_eq!(restored.critical.len(), 1);
        assert_eq!(restored.critical[0].id, "CVE-2024-1234");
    }

    #[test]
    fn test_port_mapping_defaults() {
        let pm = PortMapping {
            host_ip: "127.0.0.1".into(),
            host_port: 3000,
            container_port: 3000,
            protocol: "tcp".into(),
        };

        let restored = roundtrip(&pm);
        assert_eq!(restored.host_port, 3000);
        assert_eq!(restored.protocol, "tcp");
    }

    #[test]
    fn test_file_metadata_roundtrip() {
        let file = FileMetadata {
            name: "app.conf".into(),
            full_path: "/etc/nginx/conf.d/app.conf".into(),
            is_dir: false,
            is_symlink: false,
            size_bytes: 2048,
            permissions: "-rw-r--r--".into(),
            owner: "root".into(),
            group: "root".into(),
            updated_at: 1718200000,
        };

        let restored = roundtrip(&file);
        assert_eq!(restored.name, "app.conf");
        assert!(!restored.is_dir);
    }

    #[test]
    fn test_none_values_roundtrip() {
        let info = ContainerInfo {
            id: "test".into(),
            name: "test".into(),
            image: "alpine".into(),
            status: "Created".into(),
            state: "created".into(),
            ports: vec![],
            cpu_percentage: 0.0,
            memory_usage_bytes: 0,
            memory_limit_bytes: 0,
            network_rx_bytes: 0,
            network_tx_bytes: 0,
            pid: 0,
            restart_count: 0,
            created_at: 0,
            started_at: None,
            labels: std::collections::HashMap::new(),
            compose_project: None,
            compose_service: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        let restored: ContainerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.started_at, None);
        assert_eq!(restored.compose_project, None);
    }

    #[test]
    fn test_json_array_of_containers() {
        let containers = vec![
            ContainerInfo {
                id: "c1".into(),
                name: "web".into(),
                image: "nginx".into(),
                status: "Up".into(),
                state: "running".into(),
                ports: vec![],
                cpu_percentage: 0.0,
                memory_usage_bytes: 0,
                memory_limit_bytes: 0,
                network_rx_bytes: 0,
                network_tx_bytes: 0,
                pid: 1,
                restart_count: 0,
                created_at: 0,
                started_at: None,
                labels: std::collections::HashMap::new(),
                compose_project: None,
                compose_service: None,
            },
            ContainerInfo {
                id: "c2".into(),
                name: "db".into(),
                image: "postgres".into(),
                status: "Up".into(),
                state: "running".into(),
                ports: vec![],
                cpu_percentage: 0.0,
                memory_usage_bytes: 0,
                memory_limit_bytes: 0,
                network_rx_bytes: 0,
                network_tx_bytes: 0,
                pid: 2,
                restart_count: 0,
                created_at: 0,
                started_at: None,
                labels: std::collections::HashMap::new(),
                compose_project: None,
                compose_service: None,
            },
        ];

        let json = serde_json::to_string(&containers).unwrap();
        let restored: Vec<ContainerInfo> = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].name, "web");
        assert_eq!(restored[1].name, "db");
    }
}
