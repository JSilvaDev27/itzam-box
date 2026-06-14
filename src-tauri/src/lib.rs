// ItzamBox — Application Library Entry Point
// Copyright (C) 2026 SodigTech — GPL-3.0

pub mod commands;
pub mod db;
pub mod engine;
pub mod utils;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use commands::events::EventStreamState;
use commands::terminal::PtyManager;
use db::manager::setup_database;
use engine::docker_linux::DockerLinuxEngine;
use engine::traits::ContainerEngine;
use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

/// Shared application state managed by Tauri
pub struct AppState {
    pub engine: Arc<dyn ContainerEngine>,
    pub db: Arc<Mutex<rusqlite::Connection>>,
    pub db_path: PathBuf,
    pub rt: tokio::runtime::Handle,
}

fn app_data_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        let dir = PathBuf::from(home).join(".itzambox");
        std::fs::create_dir_all(&dir).ok();
        return dir;
    }
    PathBuf::from(".")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    let rt_handle = rt.handle().clone();

    let db_path = app_data_dir().join("itzambox.db");
    let db_path_clone = db_path.clone();
    let db = setup_database(db_path).expect("Failed to initialize database");
    let engine = DockerLinuxEngine::new();
    let db_arc = Arc::new(Mutex::new(db));

    let state = AppState {
        engine: Arc::new(engine),
        db: Arc::clone(&db_arc),
        db_path: db_path_clone,
        rt: rt_handle.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .target(Target::new(TargetKind::Stdout))
                .target(Target::new(TargetKind::Folder {
                    path: app_data_dir(),
                    file_name: Some("itzambox".into()),
                }))
                .level(log::LevelFilter::Info)
                .max_file_size(1_000_000) // 1 MB
                .rotation_strategy(RotationStrategy::KeepSome(5))
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{} [{}] {}",
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                        record.level(),
                        message
                    ))
                })
                .build(),
        )
        .setup(move |app| {
            log::info!("ItzamBox started — engine initialized");

            // Seed built-in container templates on first run
            let state = app.state::<AppState>();
            let rt_handle = state.rt.clone();
            let db_seed = Arc::clone(&state.db);
            let app_handle = app.handle().clone();
            rt_handle.block_on(async move {
                let needs_seed = match db_seed.lock() {
                    Ok(db) => {
                        let count: i64 = db
                            .query_row(
                                "SELECT COUNT(*) FROM container_templates WHERE is_builtin = 1",
                                [],
                                |row| row.get(0),
                            )
                            .unwrap_or(0);
                        count == 0
                    }
                    Err(e) => {
                        log::error!("Failed to lock DB for seeding: {}", e);
                        false
                    }
                };
                if needs_seed {
                    let seed_state = app_handle.state::<AppState>();
                    if let Err(e) = commands::templates::seed_builtin_templates(seed_state).await {
                        log::error!("Failed to seed built-in templates: {}", e);
                    }
                }
            });

            // ─── Compaction Background Tasks ──────────────────────────
            let db_5min = Arc::clone(&db_arc);
            let db_30min = Arc::clone(&db_arc);
            let db_purge = Arc::clone(&db_arc);

            // 5-minute compaction: run every 5 minutes, compact raw data older than 24h.
            let rt_h = state.rt.clone();
            rt_h.spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    let db = match db_5min.lock() {
                        Ok(db) => db,
                        Err(e) => {
                            log::error!("compaction_5min lock: {}", e);
                            continue;
                        }
                    };
                    match crate::engine::metrics_history::compact_5min(&db, 86_400) {
                        Ok(n) => {
                            if n > 0 {
                                log::info!("Compaction 5min: created {} buckets", n);
                            }
                        }
                        Err(e) => log::error!("Compaction 5min failed: {}", e),
                    }
                }
            });

            // 30-minute compaction: run every 30 minutes, compact 5-min data older than 7d.
            let rt_h2 = state.rt.clone();
            rt_h2.spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1800));
                loop {
                    interval.tick().await;
                    let db = match db_30min.lock() {
                        Ok(db) => db,
                        Err(e) => {
                            log::error!("compaction_30min lock: {}", e);
                            continue;
                        }
                    };
                    match crate::engine::metrics_history::compact_30min(&db, 604_800) {
                        Ok(n) => {
                            if n > 0 {
                                log::info!("Compaction 30min: created {} buckets", n);
                            }
                        }
                        Err(e) => log::error!("Compaction 30min failed: {}", e),
                    }
                }
            });

            // Daily purge: run every 24 hours, delete data older than 30 days.
            let rt_h3 = state.rt.clone();
            rt_h3.spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86_400));
                loop {
                    interval.tick().await;
                    let db = match db_purge.lock() {
                        Ok(db) => db,
                        Err(e) => {
                            log::error!("purge lock: {}", e);
                            continue;
                        }
                    };
                    match crate::engine::metrics_history::purge_old_data(&db, 30) {
                        Ok(n) => {
                            if n > 0 {
                                log::info!("Purge: deleted {} old metric buckets", n);
                            }
                        }
                        Err(e) => log::error!("Purge failed: {}", e),
                    }
                }
            });

            // ─── Backup Scheduler ─────────────────────────────────────
            commands::backup::spawn_backup_scheduler(Arc::clone(&db_arc), state.rt.clone());

            Ok(())
        })
        .manage(state)
        .manage(PtyManager::new())
        .manage(EventStreamState::new())
        .invoke_handler(tauri::generate_handler![
            // Events
            commands::events::start_event_stream,
            commands::events::stop_event_stream,
            // Containers
            commands::containers::list_containers,
            commands::containers::inspect_container,
            commands::containers::start_container,
            commands::containers::stop_container,
            commands::containers::restart_container,
            commands::containers::pause_container,
            commands::containers::unpause_container,
            commands::containers::kill_container,
            commands::containers::rename_container,
            commands::containers::remove_container,
            commands::containers::get_container_stats,
            commands::containers::get_container_logs,
            commands::containers::create_and_run_container,
            commands::containers::export_container,
            commands::containers::commit_container,
            // Images
            commands::images::list_images,
            commands::images::pull_image,
            commands::images::remove_image,
            commands::images::tag_image,
            commands::images::inspect_image,
            commands::images::get_image_history,
            commands::images::search_dockerhub,
            commands::images::build_image,
            commands::images::save_image,
            commands::images::load_image,
            // Volumes
            commands::volumes::list_volumes,
            commands::volumes::create_volume,
            commands::volumes::remove_volume,
            // Networks
            commands::networks::list_networks,
            commands::networks::create_network,
            commands::networks::remove_network,
            // Settings
            commands::settings::get_config,
            commands::settings::set_config,
            commands::settings::export_settings,
            commands::settings::import_settings,
            // Host Metrics
            commands::host_metrics::get_host_metrics,
            // Terminal
            commands::terminal::spawn_host_terminal,
            commands::terminal::spawn_container_terminal,
            commands::terminal::pty_write,
            commands::terminal::pty_resize,
            commands::terminal::pty_close,
            // Cleanup & Engine Info
            commands::cleanup::get_disk_usage,
            commands::cleanup::get_engine_version,
            commands::cleanup::get_engine_info,
            commands::cleanup::check_engine_status,
            commands::cleanup::prune_containers,
            commands::cleanup::prune_images,
            commands::cleanup::prune_volumes,
            commands::cleanup::prune_networks,
            commands::cleanup::list_container_dir,
            // File Explorer
            commands::containers::download_file_from_container,
            commands::containers::upload_file_to_container,
            commands::containers::read_file_preview,
            // Docker Compose
            commands::compose::detect_compose_projects,
            commands::compose::parse_compose_file,
            commands::compose::compose_up,
            commands::compose::compose_down,
            commands::compose::compose_restart,
            commands::compose::compose_logs,
            commands::compose::compose_ps,
            commands::compose::read_compose_file,
            commands::compose::write_compose_file,
            commands::compose::validate_compose_file,
            commands::compose::format_compose_file,
            // Notifications
            commands::notifications::save_notification,
            commands::notifications::get_notifications,
            commands::notifications::mark_notification_read,
            commands::notifications::mark_all_read,
            commands::notifications::clear_notifications,
            // Registries
            commands::registries::list_registries,
            commands::registries::add_registry,
            commands::registries::update_registry,
            commands::registries::remove_registry,
            commands::registries::set_default_registry,
            commands::registries::docker_login,
            commands::registries::docker_logout,
            commands::registries::push_image,
            // Docker Installer
            commands::installer::detect_linux_distro,
            commands::installer::check_docker_installed,
            commands::installer::install_docker,
            commands::installer::validate_docker_install,
            // Container Templates
            commands::templates::list_templates,
            commands::templates::get_template,
            commands::templates::save_template,
            commands::templates::delete_template,
            commands::templates::seed_builtin_templates,
            // Vulnerability Scanner
            commands::scanner::detect_scanner,
            commands::scanner::scan_image,
            commands::scanner::get_scan_history,
            // Backup & Restore
            commands::backup::create_backup,
            commands::backup::list_backups,
            commands::backup::restore_backup,
            commands::backup::delete_backup,
            commands::backup::verify_checksum,
            commands::backup::cancel_backup,
            commands::backup::schedule_backup,
            commands::backup::list_backup_jobs,
            commands::backup::toggle_backup_job,
            commands::backup::delete_backup_job,
            commands::backup::export_container_data,
            commands::backup::import_to_volume,
            // Docker Swarm
            commands::swarm::swarm_status,
            commands::swarm::swarm_init,
            commands::swarm::swarm_join,
            commands::swarm::swarm_leave,
            commands::swarm::list_swarm_nodes,
            commands::swarm::inspect_swarm_node,
            commands::swarm::list_swarm_services,
            commands::swarm::inspect_swarm_service,
            commands::swarm::list_stacks,
            commands::swarm::deploy_stack,
            commands::swarm::remove_stack,
            // Kubernetes
            commands::kubernetes::detect_kubectl,
            commands::kubernetes::list_k8s_contexts,
            commands::kubernetes::set_k8s_context,
            commands::kubernetes::list_namespaces,
            commands::kubernetes::list_pods,
            commands::kubernetes::list_deployments,
            commands::kubernetes::list_services,
            commands::kubernetes::list_configmaps,
            commands::kubernetes::list_secrets,
            commands::kubernetes::get_resource_yaml,
            commands::kubernetes::get_pod_events,
            // Metrics History
            commands::metrics_history::get_host_metrics_range,
            commands::metrics_history::get_container_metrics_range,
            commands::metrics_history::export_metrics_csv,
            commands::metrics_history::export_metrics_json,
            commands::metrics_history::get_metrics_db_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ItzamBox");
}
