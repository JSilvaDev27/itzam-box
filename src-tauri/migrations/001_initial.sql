-- ItzamBox — Initial Database Schema
-- Copyright (C) 2026 SodigTech — GPL-3.0
-- Migration: 001_initial.sql

CREATE TABLE IF NOT EXISTS system_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS pinned_containers (
    container_id TEXT PRIMARY KEY,
    alias TEXT,
    pinned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS terminal_commands (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel TEXT NOT NULL,
    command TEXT NOT NULL,
    executed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS scan_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    image_name TEXT NOT NULL,
    critical_count INTEGER DEFAULT 0,
    high_count INTEGER DEFAULT 0,
    medium_count INTEGER DEFAULT 0,
    low_count INTEGER DEFAULT 0,
    report_json TEXT,
    scanned_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS host_metrics_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    cpu_percent REAL,
    memory_used_bytes INTEGER,
    memory_total_bytes INTEGER,
    disk_used_bytes INTEGER,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS container_metrics_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    container_id TEXT NOT NULL,
    cpu_percent REAL,
    memory_usage_bytes INTEGER,
    network_rx_bytes INTEGER,
    network_tx_bytes INTEGER,
    recorded_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS registries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    username TEXT,
    auth_token TEXT,
    is_default BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS compose_projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    last_used_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
