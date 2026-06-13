// ItzamBox — ContainerEngine Trait (Port Interface)
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::*;
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait ContainerEngine: Send + Sync {
    // ─── Diagnostics ───
    async fn check_engine_status(&self) -> Result<bool, String>;
    async fn get_engine_version(&self) -> Result<String, String>;
    async fn get_engine_info(&self) -> Result<serde_json::Value, String>;

    // ─── Container Lifecycle ───
    async fn list_containers(&self, show_all: bool) -> Result<Vec<ContainerInfo>, String>;
    async fn inspect_container(&self, id: &str) -> Result<serde_json::Value, String>;
    async fn start_container(&self, id: &str) -> Result<(), String>;
    async fn stop_container(&self, id: &str) -> Result<(), String>;
    async fn restart_container(&self, id: &str) -> Result<(), String>;
    async fn pause_container(&self, id: &str) -> Result<(), String>;
    async fn unpause_container(&self, id: &str) -> Result<(), String>;
    async fn kill_container(&self, id: &str, signal: &str) -> Result<(), String>;
    async fn rename_container(&self, id: &str, new_name: &str) -> Result<(), String>;
    async fn remove_container(
        &self,
        id: &str,
        force: bool,
        remove_volumes: bool,
    ) -> Result<(), String>;
    async fn get_container_stats(&self, id: &str) -> Result<ContainerStats, String>;

    // ─── Container Creation ───
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
    ) -> Result<String, String>;

    // ─── File Explorer ───
    async fn list_container_dir(
        &self,
        container_id: &str,
        path: &str,
    ) -> Result<Vec<FileMetadata>, String>;
    async fn download_file_from_container(
        &self,
        container_id: &str,
        remote_path: &str,
        local_dest: PathBuf,
    ) -> Result<(), String>;
    async fn upload_file_to_container(
        &self,
        container_id: &str,
        local_src: PathBuf,
        remote_dest: &str,
    ) -> Result<(), String>;
    async fn read_file_preview(
        &self,
        container_id: &str,
        remote_path: &str,
        max_bytes: usize,
    ) -> Result<String, String>;

    // ─── Images ───
    async fn list_images(&self) -> Result<Vec<ImageInfo>, String>;
    async fn inspect_image(&self, id: &str) -> Result<serde_json::Value, String>;
    async fn get_image_history(&self, id: &str) -> Result<Vec<ImageLayerInfo>, String>;
    async fn pull_image(&self, image_name: &str) -> Result<(), String>;
    async fn remove_image(&self, id: &str, force: bool) -> Result<(), String>;
    async fn tag_image(&self, id: &str, repository: &str, tag: &str) -> Result<(), String>;

    // ─── Volumes ───
    async fn list_volumes(&self) -> Result<Vec<VolumeInfo>, String>;
    async fn create_volume(&self, name: &str, driver: Option<&str>) -> Result<(), String>;
    async fn remove_volume(&self, name: &str, force: bool) -> Result<(), String>;

    // ─── Networks ───
    async fn list_networks(&self) -> Result<Vec<NetworkInfo>, String>;
    async fn create_network(
        &self,
        name: &str,
        driver: &str,
        subnet: Option<&str>,
        gateway: Option<&str>,
    ) -> Result<(), String>;
    async fn remove_network(&self, id: &str) -> Result<(), String>;

    // ─── Cleanup ───
    async fn get_disk_usage(&self) -> Result<DiskUsageSummary, String>;
    async fn prune_containers(&self) -> Result<u64, String>;
    async fn prune_images(&self, dangling_only: bool) -> Result<u64, String>;
    async fn prune_volumes(&self) -> Result<u64, String>;
    async fn prune_networks(&self) -> Result<u64, String>;
    async fn prune_build_cache(&self) -> Result<u64, String>;
    async fn system_prune_all(&self) -> Result<DiskUsageSummary, String>;

    // ─── Host Metrics ───
    async fn get_host_metrics(&self) -> Result<HostMetrics, String>;
}
