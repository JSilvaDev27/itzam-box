// ItzamBox — Host Metrics Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// The `get_host_metrics` command now reads from a background-refreshed
// cache (HostMetricsCache) — no synchronous sysinfo calls or 200 ms sleep.
// DB persistence is handled asynchronously by the cache's batch-flush worker.

use crate::engine::types::HostMetrics;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_host_metrics(state: State<'_, AppState>) -> Result<HostMetrics, String> {
    // Reads from the background-refreshed cache → O(1), no I/O, no sleep.
    let metrics = state.engine.get_host_metrics().await?;

    // DB persistence is handled by the HostMetricsCache background worker
    // (batched every 30 s or every 10 samples), so we no longer block
    // the `std::sync::Mutex` on every command invocation.

    Ok(metrics)
}
