-- ItzamBox — Time-Series Compaction Tables
-- Copyright (C) 2026 SodigTech — GPL-3.0
-- Migration: 006_metrics_timeseries.sql
--
-- 3-tier compaction: raw (5s) → 5min → 30min → purge after 30d

-- 5-minute aggregates from raw host_metrics_history
CREATE TABLE IF NOT EXISTS host_metrics_5min (
    bucket_start    INTEGER PRIMARY KEY NOT NULL,
    avg_cpu_percent REAL    NOT NULL,
    max_cpu_percent REAL    NOT NULL,
    avg_memory_used_bytes INTEGER NOT NULL,
    max_memory_used_bytes INTEGER NOT NULL,
    avg_disk_used_bytes   INTEGER NOT NULL,
    avg_network_rx_bytes  INTEGER NOT NULL DEFAULT 0,
    avg_network_tx_bytes  INTEGER NOT NULL DEFAULT 0,
    avg_disk_read_bytes   INTEGER NOT NULL DEFAULT 0,
    avg_disk_write_bytes  INTEGER NOT NULL DEFAULT 0,
    sample_count    INTEGER NOT NULL
);

-- 30-minute aggregates from host_metrics_5min
CREATE TABLE IF NOT EXISTS host_metrics_30min (
    bucket_start    INTEGER PRIMARY KEY NOT NULL,
    avg_cpu_percent REAL    NOT NULL,
    max_cpu_percent REAL    NOT NULL,
    avg_memory_used_bytes INTEGER NOT NULL,
    max_memory_used_bytes INTEGER NOT NULL,
    avg_disk_used_bytes   INTEGER NOT NULL,
    avg_network_rx_bytes  INTEGER NOT NULL DEFAULT 0,
    avg_network_tx_bytes  INTEGER NOT NULL DEFAULT 0,
    avg_disk_read_bytes   INTEGER NOT NULL DEFAULT 0,
    avg_disk_write_bytes  INTEGER NOT NULL DEFAULT 0,
    sample_count    INTEGER NOT NULL
);

-- Per-container 5-minute aggregates from raw container_metrics_history
CREATE TABLE IF NOT EXISTS container_metrics_5min (
    bucket_start       INTEGER NOT NULL,
    container_id       TEXT    NOT NULL,
    avg_cpu_percent    REAL    NOT NULL,
    max_cpu_percent    REAL    NOT NULL,
    avg_memory_usage_bytes INTEGER NOT NULL,
    max_memory_usage_bytes INTEGER NOT NULL,
    avg_network_rx_bytes   INTEGER NOT NULL DEFAULT 0,
    avg_network_tx_bytes   INTEGER NOT NULL DEFAULT 0,
    avg_block_read_bytes   INTEGER NOT NULL DEFAULT 0,
    avg_block_write_bytes  INTEGER NOT NULL DEFAULT 0,
    sample_count       INTEGER NOT NULL,
    PRIMARY KEY (bucket_start, container_id)
);

-- Indexes for range queries
CREATE INDEX IF NOT EXISTS idx_host_5min_bucket
    ON host_metrics_5min(bucket_start);
CREATE INDEX IF NOT EXISTS idx_host_30min_bucket
    ON host_metrics_30min(bucket_start);
CREATE INDEX IF NOT EXISTS idx_container_5min_lookup
    ON container_metrics_5min(container_id, bucket_start);
