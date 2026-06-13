// ItzamBox — Application Library Entry Point
// Copyright (C) 2026 SodigTech — GPL-3.0

pub mod db;
pub mod engine;
pub mod commands;
pub mod utils;

use std::sync::{Arc, Mutex};
use std::path::PathBuf;

use db::manager::setup_database;
use engine::docker_linux::DockerLinuxEngine;
use engine::traits::ContainerEngine;

/// Shared application state managed by Tauri
pub struct AppState {
    pub engine: Arc<dyn ContainerEngine>,
    pub db: Arc<Mutex<rusqlite::Connection>>,
}

/// Resolve the application data directory for SQLite database
fn app_data_dir() -> PathBuf {
    // Use HOME/.itzambox for data storage
    if let Ok(home) = std::env::var("HOME") {
        let dir = PathBuf::from(home).join(".itzambox");
        std::fs::create_dir_all(&dir).ok();
        return dir;
    }
    // Fallback to current directory
    PathBuf::from(".")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = app_data_dir().join("itzambox.db");

    // Initialize SQLite database
    let db = setup_database(db_path).expect("Failed to initialize database");

    // Initialize Docker Engine adapter
    let engine = DockerLinuxEngine::new();

    let state = AppState {
        engine: Arc::new(engine),
        db: Arc::new(Mutex::new(db)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::containers::list_containers,
            commands::images::list_images,
            commands::settings::get_config,
            commands::settings::set_config,
            commands::host_metrics::get_host_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running ItzamBox");
}
