// ItzamBox — Time-Series Metrics Storage & Compaction Engine
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::types::{ContainerMetricsPoint, ContainerStats, HostMetrics, MetricsDataPoint};
use rusqlite::{params, Connection};
use std::io::Write;
use std::path::Path;

// ─── Insertion ───────────────────────────────────────────────────────────

/// Persist a single host-metrics snapshot into the raw table.
pub fn insert_host_metrics(conn: &Connection, m: &HostMetrics) -> Result<(), String> {
    conn.execute(
        "INSERT INTO host_metrics_history (cpu_percent, memory_used_bytes, \
         memory_total_bytes, disk_used_bytes, recorded_at) \
         VALUES (?1, ?2, ?3, ?4, strftime('%s','now'))",
        params![
            m.cpu_usage_percent,
            m.memory_used_bytes as i64,
            m.memory_total_bytes as i64,
            m.disk_used_bytes as i64,
        ],
    )
    .map_err(|e| format!("insert_host_metrics failed: {}", e))?;
    Ok(())
}

/// Persist a single container-stats snapshot into the raw table.
pub fn insert_container_stats(conn: &Connection, s: &ContainerStats) -> Result<(), String> {
    conn.execute(
        "INSERT INTO container_metrics_history \
         (container_id, cpu_percent, memory_usage_bytes, \
          network_rx_bytes, network_tx_bytes, recorded_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, strftime('%s','now'))",
        params![
            s.container_id,
            s.cpu_percentage,
            s.memory_usage_bytes as i64,
            s.network_rx_bytes as i64,
            s.network_tx_bytes as i64,
        ],
    )
    .map_err(|e| format!("insert_container_stats failed: {}", e))?;
    Ok(())
}

// ─── Compaction ──────────────────────────────────────────────────────────

/// Aggregate raw rows older than `older_than_secs` into 5-minute buckets.
/// Returns the number of buckets created.
pub fn compact_5min(conn: &Connection, older_than_secs: i64) -> Result<usize, String> {
    let cutoff = epoch_now() - older_than_secs;
    // Find the latest bucket already compacted so we don't duplicate work.
    let last_bucket: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(bucket_start), 0) FROM host_metrics_5min",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Mark the start of the compaction window: anything before cutoff,
    // but after the last compacted bucket.
    let window_start = last_bucket.max(0);
    let window_end = cutoff;

    if window_end <= window_start {
        return Ok(0);
    }

    // Insert aggregated 5-minute buckets.
    let rows = conn
        .execute(
            "INSERT OR IGNORE INTO host_metrics_5min \
         (bucket_start, avg_cpu_percent, max_cpu_percent, \
          avg_memory_used_bytes, max_memory_used_bytes, \
          avg_disk_used_bytes, sample_count) \
         SELECT \
           (CAST(strftime('%s', recorded_at) AS INTEGER) / 300) * 300 AS bucket, \
           ROUND(AVG(cpu_percent), 2), \
           ROUND(MAX(cpu_percent), 2), \
           CAST(AVG(COALESCE(memory_used_bytes, 0)) AS INTEGER), \
           CAST(MAX(COALESCE(memory_used_bytes, 0)) AS INTEGER), \
           CAST(AVG(COALESCE(disk_used_bytes, 0)) AS INTEGER), \
           COUNT(*) \
         FROM host_metrics_history \
         WHERE CAST(strftime('%s', recorded_at) AS INTEGER) > ?1 \
           AND CAST(strftime('%s', recorded_at) AS INTEGER) <= ?2 \
         GROUP BY bucket \
         ORDER BY bucket",
            params![window_start, window_end],
        )
        .map_err(|e| format!("compact_5min aggregation failed: {}", e))?;

    // Delete the raw rows that were just compacted.
    conn.execute(
        "DELETE FROM host_metrics_history \
         WHERE CAST(strftime('%s', recorded_at) AS INTEGER) > ?1 \
           AND CAST(strftime('%s', recorded_at) AS INTEGER) <= ?2",
        params![window_start, window_end],
    )
    .map_err(|e| format!("compact_5min delete failed: {}", e))?;

    Ok(rows)
}

/// Aggregate 5-minute buckets older than `older_than_secs` into 30-minute buckets.
/// Returns the number of buckets created.
pub fn compact_30min(conn: &Connection, older_than_secs: i64) -> Result<usize, String> {
    let cutoff = epoch_now() - older_than_secs;
    let last_bucket: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(bucket_start), 0) FROM host_metrics_30min",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let window_start = last_bucket.max(0);
    let window_end = cutoff;

    if window_end <= window_start {
        return Ok(0);
    }

    let rows = conn
        .execute(
            "INSERT OR IGNORE INTO host_metrics_30min \
         (bucket_start, avg_cpu_percent, max_cpu_percent, \
          avg_memory_used_bytes, max_memory_used_bytes, \
          avg_disk_used_bytes, sample_count) \
         SELECT \
           (bucket_start / 1800) * 1800 AS bucket, \
           ROUND(AVG(avg_cpu_percent), 2), \
           ROUND(MAX(max_cpu_percent), 2), \
           CAST(AVG(avg_memory_used_bytes) AS INTEGER), \
           CAST(MAX(max_memory_used_bytes) AS INTEGER), \
           CAST(AVG(avg_disk_used_bytes) AS INTEGER), \
           SUM(sample_count) \
         FROM host_metrics_5min \
         WHERE bucket_start > ?1 AND bucket_start <= ?2 \
         GROUP BY bucket \
         ORDER BY bucket",
            params![window_start, window_end],
        )
        .map_err(|e| format!("compact_30min aggregation failed: {}", e))?;

    // Delete the compacted 5-min rows.
    conn.execute(
        "DELETE FROM host_metrics_5min WHERE bucket_start > ?1 AND bucket_start <= ?2",
        params![window_start, window_end],
    )
    .map_err(|e| format!("compact_30min delete failed: {}", e))?;

    Ok(rows)
}

/// Purge 30-minute data older than `older_than_days` days.
/// Returns the number of rows deleted.
pub fn purge_old_data(conn: &Connection, older_than_days: u32) -> Result<usize, String> {
    let cutoff = epoch_now() - (older_than_days as i64 * 86_400);
    let rows = conn
        .execute(
            "DELETE FROM host_metrics_30min WHERE bucket_start <= ?1",
            params![cutoff],
        )
        .map_err(|e| format!("purge_old_data failed: {}", e))?;
    Ok(rows)
}

// ─── Query ───────────────────────────────────────────────────────────────

/// Query host metrics over a time range, auto-selecting the correct table
/// based on the span duration.
///
/// - <= 24 h  → raw `host_metrics_history`
/// -  1-7  d  → `host_metrics_5min`
/// -  7-30 d  → `host_metrics_30min`
pub fn query_range(conn: &Connection, from: i64, to: i64) -> Result<Vec<MetricsDataPoint>, String> {
    let span_secs = to - from;
    let points = if span_secs <= 86_400 {
        query_raw(conn, from, to)?
    } else if span_secs <= 604_800 {
        query_5min(conn, from, to)?
    } else {
        query_30min(conn, from, to)?
    };
    Ok(points)
}

/// Query per-container metrics over a time range.
pub fn query_container_range(
    conn: &Connection,
    container_id: &str,
    from: i64,
    to: i64,
) -> Result<Vec<ContainerMetricsPoint>, String> {
    let span_secs = to - from;
    if span_secs <= 86_400 {
        query_container_raw(conn, container_id, from, to)
    } else {
        query_container_5min(conn, container_id, from, to)
    }
}

// ─── Export ──────────────────────────────────────────────────────────────

/// Export metrics as RFC 4180 CSV to `dest_path`.
pub fn export_csv(conn: &Connection, from: i64, to: i64, dest_path: &str) -> Result<(), String> {
    let points = query_range(conn, from, to)?;
    let mut wtr = csv::Writer::from_path(dest_path)
        .map_err(|e| format!("cannot create CSV writer: {}", e))?;

    wtr.write_record([
        "timestamp",
        "cpu_percent",
        "memory_used_bytes",
        "memory_total_bytes",
        "disk_used_bytes",
        "sample_count",
    ])
    .map_err(|e| format!("CSV write header: {}", e))?;

    for p in &points {
        wtr.write_record([
            p.timestamp.to_string(),
            p.cpu_percent.to_string(),
            p.memory_used_bytes.to_string(),
            p.memory_total_bytes.to_string(),
            p.disk_used_bytes.to_string(),
            p.sample_count.to_string(),
        ])
        .map_err(|e| format!("CSV write row: {}", e))?;
    }
    wtr.flush().map_err(|e| format!("CSV flush: {}", e))?;
    Ok(())
}

/// Export metrics as a JSON array to `dest_path`.
pub fn export_json(conn: &Connection, from: i64, to: i64, dest_path: &str) -> Result<(), String> {
    let points = query_range(conn, from, to)?;
    let json =
        serde_json::to_string_pretty(&points).map_err(|e| format!("JSON serialize: {}", e))?;

    let mut file =
        std::fs::File::create(dest_path).map_err(|e| format!("cannot create JSON file: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("JSON write: {}", e))?;
    Ok(())
}

/// Return the SQLite database file size in bytes.
pub fn get_db_size(db_path: &Path) -> Result<u64, String> {
    let meta = std::fs::metadata(db_path).map_err(|e| format!("get_db_size: {}", e))?;
    Ok(meta.len())
}

// ─── Internal Query Helpers ──────────────────────────────────────────────

fn query_raw(conn: &Connection, from: i64, to: i64) -> Result<Vec<MetricsDataPoint>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT cpu_percent, memory_used_bytes, memory_total_bytes, \
                    disk_used_bytes, CAST(strftime('%s', recorded_at) AS INTEGER) \
             FROM host_metrics_history \
             WHERE CAST(strftime('%s', recorded_at) AS INTEGER) >= ?1 \
               AND CAST(strftime('%s', recorded_at) AS INTEGER) <= ?2 \
             ORDER BY recorded_at ASC",
        )
        .map_err(|e| format!("query_raw prepare: {}", e))?;

    let rows = stmt
        .query_map(params![from, to], |row| {
            let ts: i64 = row.get(4)?;
            Ok(MetricsDataPoint {
                timestamp: ts,
                cpu_percent: row.get::<_, f64>(0).unwrap_or(0.0),
                memory_used_bytes: row.get::<_, i64>(1).unwrap_or(0) as u64,
                memory_total_bytes: row.get::<_, i64>(2).unwrap_or(0) as u64,
                disk_used_bytes: row.get::<_, i64>(3).unwrap_or(0) as u64,
                sample_count: 1,
            })
        })
        .map_err(|e| format!("query_raw rows: {}", e))?;

    let mut points = Vec::new();
    for row in rows {
        points.push(row.map_err(|e| format!("query_raw row error: {}", e))?);
    }
    Ok(points)
}

fn query_5min(conn: &Connection, from: i64, to: i64) -> Result<Vec<MetricsDataPoint>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bucket_start, avg_cpu_percent, avg_memory_used_bytes, \
                    avg_disk_used_bytes, sample_count \
             FROM host_metrics_5min \
             WHERE bucket_start >= ?1 AND bucket_start <= ?2 \
             ORDER BY bucket_start ASC",
        )
        .map_err(|e| format!("query_5min prepare: {}", e))?;

    let rows = stmt
        .query_map(params![from, to], |row| {
            Ok(MetricsDataPoint {
                timestamp: row.get(0)?,
                cpu_percent: row.get::<_, f64>(1).unwrap_or(0.0),
                memory_used_bytes: row.get::<_, i64>(2).unwrap_or(0) as u64,
                memory_total_bytes: 0,
                disk_used_bytes: row.get::<_, i64>(3).unwrap_or(0) as u64,
                sample_count: row.get::<_, i32>(4).unwrap_or(0) as u32,
            })
        })
        .map_err(|e| format!("query_5min rows: {}", e))?;

    let mut points = Vec::new();
    for row in rows {
        points.push(row.map_err(|e| format!("query_5min row error: {}", e))?);
    }
    Ok(points)
}

fn query_30min(conn: &Connection, from: i64, to: i64) -> Result<Vec<MetricsDataPoint>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bucket_start, avg_cpu_percent, avg_memory_used_bytes, \
                    avg_disk_used_bytes, sample_count \
             FROM host_metrics_30min \
             WHERE bucket_start >= ?1 AND bucket_start <= ?2 \
             ORDER BY bucket_start ASC",
        )
        .map_err(|e| format!("query_30min prepare: {}", e))?;

    let rows = stmt
        .query_map(params![from, to], |row| {
            Ok(MetricsDataPoint {
                timestamp: row.get(0)?,
                cpu_percent: row.get::<_, f64>(1).unwrap_or(0.0),
                memory_used_bytes: row.get::<_, i64>(2).unwrap_or(0) as u64,
                memory_total_bytes: 0,
                disk_used_bytes: row.get::<_, i64>(3).unwrap_or(0) as u64,
                sample_count: row.get::<_, i32>(4).unwrap_or(0) as u32,
            })
        })
        .map_err(|e| format!("query_30min rows: {}", e))?;

    let mut points = Vec::new();
    for row in rows {
        points.push(row.map_err(|e| format!("query_30min row error: {}", e))?);
    }
    Ok(points)
}

fn query_container_raw(
    conn: &Connection,
    container_id: &str,
    from: i64,
    to: i64,
) -> Result<Vec<ContainerMetricsPoint>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT container_id, cpu_percent, memory_usage_bytes, \
                    network_rx_bytes, network_tx_bytes, \
                    CAST(strftime('%s', recorded_at) AS INTEGER) \
             FROM container_metrics_history \
             WHERE container_id = ?1 \
               AND CAST(strftime('%s', recorded_at) AS INTEGER) >= ?2 \
               AND CAST(strftime('%s', recorded_at) AS INTEGER) <= ?3 \
             ORDER BY recorded_at ASC",
        )
        .map_err(|e| format!("query_container_raw prepare: {}", e))?;

    let rows = stmt
        .query_map(params![container_id, from, to], |row| {
            Ok(ContainerMetricsPoint {
                timestamp: row.get(5)?,
                container_id: row.get(0)?,
                cpu_percent: row.get::<_, f64>(1).unwrap_or(0.0),
                memory_usage_bytes: row.get::<_, i64>(2).unwrap_or(0) as u64,
                network_rx_bytes: row.get::<_, i64>(3).unwrap_or(0) as u64,
                network_tx_bytes: row.get::<_, i64>(4).unwrap_or(0) as u64,
                block_read_bytes: 0,
                block_write_bytes: 0,
                sample_count: 1,
            })
        })
        .map_err(|e| format!("query_container_raw rows: {}", e))?;

    let mut points = Vec::new();
    for row in rows {
        points.push(row.map_err(|e| format!("query_container_raw row: {}", e))?);
    }
    Ok(points)
}

fn query_container_5min(
    conn: &Connection,
    container_id: &str,
    from: i64,
    to: i64,
) -> Result<Vec<ContainerMetricsPoint>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT bucket_start, container_id, avg_cpu_percent, \
                    avg_memory_usage_bytes, avg_network_rx_bytes, \
                    avg_network_tx_bytes, sample_count \
             FROM container_metrics_5min \
             WHERE container_id = ?1 \
               AND bucket_start >= ?2 AND bucket_start <= ?3 \
             ORDER BY bucket_start ASC",
        )
        .map_err(|e| format!("query_container_5min prepare: {}", e))?;

    let rows = stmt
        .query_map(params![container_id, from, to], |row| {
            Ok(ContainerMetricsPoint {
                timestamp: row.get(0)?,
                container_id: row.get(1)?,
                cpu_percent: row.get::<_, f64>(2).unwrap_or(0.0),
                memory_usage_bytes: row.get::<_, i64>(3).unwrap_or(0) as u64,
                network_rx_bytes: row.get::<_, i64>(4).unwrap_or(0) as u64,
                network_tx_bytes: row.get::<_, i64>(5).unwrap_or(0) as u64,
                block_read_bytes: 0,
                block_write_bytes: 0,
                sample_count: row.get::<_, i32>(6).unwrap_or(0) as u32,
            })
        })
        .map_err(|e| format!("query_container_5min rows: {}", e))?;

    let mut points = Vec::new();
    for row in rows {
        points.push(row.map_err(|e| format!("query_container_5min row: {}", e))?);
    }
    Ok(points)
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn epoch_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::{ContainerStats, HostMetrics, MetricsDataPoint};
    use rusqlite::Connection;
    use tempfile::NamedTempFile;

    /// Build an in-memory SQLite database with all the schema we need for testing.
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS host_metrics_history (
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
            CREATE TABLE IF NOT EXISTS host_metrics_5min (
                bucket_start INTEGER PRIMARY KEY NOT NULL,
                avg_cpu_percent REAL NOT NULL,
                max_cpu_percent REAL NOT NULL,
                avg_memory_used_bytes INTEGER NOT NULL,
                max_memory_used_bytes INTEGER NOT NULL,
                avg_disk_used_bytes INTEGER NOT NULL,
                avg_network_rx_bytes INTEGER NOT NULL DEFAULT 0,
                avg_network_tx_bytes INTEGER NOT NULL DEFAULT 0,
                avg_disk_read_bytes INTEGER NOT NULL DEFAULT 0,
                avg_disk_write_bytes INTEGER NOT NULL DEFAULT 0,
                sample_count INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS host_metrics_30min (
                bucket_start INTEGER PRIMARY KEY NOT NULL,
                avg_cpu_percent REAL NOT NULL,
                max_cpu_percent REAL NOT NULL,
                avg_memory_used_bytes INTEGER NOT NULL,
                max_memory_used_bytes INTEGER NOT NULL,
                avg_disk_used_bytes INTEGER NOT NULL,
                avg_network_rx_bytes INTEGER NOT NULL DEFAULT 0,
                avg_network_tx_bytes INTEGER NOT NULL DEFAULT 0,
                avg_disk_read_bytes INTEGER NOT NULL DEFAULT 0,
                avg_disk_write_bytes INTEGER NOT NULL DEFAULT 0,
                sample_count INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS container_metrics_5min (
                bucket_start INTEGER NOT NULL,
                container_id TEXT NOT NULL,
                avg_cpu_percent REAL NOT NULL,
                max_cpu_percent REAL NOT NULL,
                avg_memory_usage_bytes INTEGER NOT NULL,
                max_memory_usage_bytes INTEGER NOT NULL,
                avg_network_rx_bytes INTEGER NOT NULL DEFAULT 0,
                avg_network_tx_bytes INTEGER NOT NULL DEFAULT 0,
                avg_block_read_bytes INTEGER NOT NULL DEFAULT 0,
                avg_block_write_bytes INTEGER NOT NULL DEFAULT 0,
                sample_count INTEGER NOT NULL,
                PRIMARY KEY (bucket_start, container_id)
            );",
        )
        .expect("schema creation");
        conn
    }

    /// Insert synthetic raw host metrics spanning several 5-minute windows.
    fn seed_raw_host_metrics(conn: &Connection, base_time: i64, count: i64) {
        for i in 0..count {
            let ts = base_time + i;
            let cpu = 10.0 + (i % 50) as f64;
            let mem = 1_000_000 + (i * 1000) % 10_000_000;
            conn.execute(
                "INSERT INTO host_metrics_history \
                 (cpu_percent, memory_used_bytes, memory_total_bytes, disk_used_bytes, recorded_at) \
                 VALUES (?1, ?2, ?3, ?4, datetime(?5, 'unixepoch'))",
                rusqlite::params![cpu, mem, 16_000_000_000i64, 200_000_000_000i64, ts],
            )
            .expect("seed insert");
        }
    }

    #[test]
    fn test_compaction_5min_aggregation() {
        let conn = setup_test_db();
        let base = 1_700_000_000i64; // arbitrary epoch timestamp
                                     // Insert 100 raw records spanning ~100 seconds (covers two 5-min buckets).
        seed_raw_host_metrics(&conn, base, 100);

        let buckets = compact_5min(&conn, 0).expect("compact_5min");
        // 100 seconds of data at base → bucket for base/300 * 300 and (base+...)/300 * 300
        // Since base=1700000000, 1700000000 / 300 = 5666666 remainder 200, so bucket = 5666666*300 = 1699999800
        // The next bucket would be at 1700000100
        // With 100 records spanning 100 seconds from base (base to base+99), we'd have:
        // - Records at ts 1700000000 to 1700000099: first bucket (1700000000/300 = 5666666.666... so bucket is 1700000000 - 200 = 1699999800)
        // Actually: 1700000000 / 300 = 5666666.666... floor = 5666666, bucket = 5666666 * 300 = 1699999800
        // 1700000099 / 300 = 5666666.996... floor = 5666666, bucket = 1699999800
        // So all 100 records go into one bucket
        assert!(buckets >= 1, "Expected at least 1 bucket, got {}", buckets);

        // Verify the bucket has correct aggregates.
        let avg_cpu: f64 = conn
            .query_row(
                "SELECT avg_cpu_percent FROM host_metrics_5min LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("read avg_cpu");
        assert!(
            (avg_cpu - 34.5).abs() < 1.0,
            "Expected avg_cpu ~34.5, got {}",
            avg_cpu
        );

        // Verify raw rows were deleted.
        let raw_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM host_metrics_history", [], |row| {
                row.get(0)
            })
            .expect("count raw");
        assert_eq!(raw_count, 0, "Expected raw rows to be deleted");
    }

    #[test]
    fn test_compaction_30min_aggregation() {
        let conn = setup_test_db();
        let base = 1_700_000_000i64;

        // Insert synthetic 5-min buckets that span 2+ hours.
        for i in 0..25 {
            let bucket = base + i * 300; // every 5 minutes
            conn.execute(
                "INSERT INTO host_metrics_5min \
                 (bucket_start, avg_cpu_percent, max_cpu_percent, \
                  avg_memory_used_bytes, max_memory_used_bytes, \
                  avg_disk_used_bytes, sample_count) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    bucket,
                    20.0 + (i % 10) as f64,
                    30.0 + (i % 10) as f64,
                    4_000_000_000i64,
                    5_000_000_000i64,
                    100_000_000_000i64,
                    60,
                ],
            )
            .expect("seed 5min");
        }

        let buckets = compact_30min(&conn, 0).expect("compact_30min");
        // 25 * 5min = 125 min, so ~4-5 thirty-min buckets.
        assert!(buckets >= 4, "Expected at least 4 buckets, got {}", buckets);

        // Verify 5-min rows are deleted.
        let count_5min: i64 = conn
            .query_row("SELECT COUNT(*) FROM host_metrics_5min", [], |row| {
                row.get(0)
            })
            .expect("count 5min");
        assert_eq!(count_5min, 0, "Expected 5-min rows to be deleted");

        // Verify 30-min data is present.
        let count_30min: i64 = conn
            .query_row("SELECT COUNT(*) FROM host_metrics_30min", [], |row| {
                row.get(0)
            })
            .expect("count 30min");
        assert!(count_30min >= 4, "Expected >= 4 30-min rows");
    }

    #[test]
    fn test_query_range_selects_correct_table() {
        let conn = setup_test_db();
        let base = 1_700_000_000i64;

        // Insert raw data spanning 1800 seconds (30 min) from base.
        seed_raw_host_metrics(&conn, base, 1800);

        // Query for a 30-minute range → span ≤ 24h, uses raw table.
        let points_raw = query_range(&conn, base, base + 1800).expect("query 30min span");
        assert!(!points_raw.is_empty(), "Expected points from raw table");
        // Raw points have sample_count == 1.
        for p in &points_raw {
            assert_eq!(p.sample_count, 1, "Expected raw sample_count=1");
        }

        // Compact all raw data into 5-min buckets.
        compact_5min(&conn, 0).expect("compact_5min");

        // Query for a 3-day range → span > 24h, uses 5-min table.
        // The data only covers the first 1800s, but the range is wide enough
        // to trigger the 5-min table.
        let points_5min = query_range(&conn, base, base + 260_000).expect("query 3day span");
        assert!(!points_5min.is_empty(), "Expected points from 5-min table");
        // 5-min aggregates have sample_count summing multiple raw points.
        for p in &points_5min {
            assert!(
                p.sample_count > 1,
                "Expected aggregated sample_count > 1, got {}",
                p.sample_count
            );
        }
    }

    #[test]
    fn test_csv_export_format() {
        let conn = setup_test_db();
        let base = 1_700_000_000i64;
        seed_raw_host_metrics(&conn, base, 10);

        let tmp = NamedTempFile::new().expect("temp file");
        let path = tmp.path().to_str().unwrap().to_string();

        export_csv(&conn, base - 10, base + 20, &path).expect("export_csv");

        let content = std::fs::read_to_string(&path).expect("read csv");
        assert!(
            content.starts_with("timestamp,cpu_percent,"),
            "CSV should start with header row"
        );
        // Should have 10 data rows + 1 header.
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 11, "Expected 10 data rows + 1 header");
        // All lines should have exactly 6 fields.
        for (idx, line) in lines.iter().enumerate() {
            let fields: Vec<&str> = line.split(',').collect();
            assert_eq!(
                fields.len(),
                6,
                "Line {} should have 6 fields: {}",
                idx,
                line
            );
        }
    }

    #[test]
    fn test_json_export_format() {
        let conn = setup_test_db();
        let base = 1_700_000_000i64;
        seed_raw_host_metrics(&conn, base, 5);

        let tmp = NamedTempFile::new().expect("temp file");
        let path = tmp.path().to_str().unwrap().to_string();

        export_json(&conn, base - 10, base + 20, &path).expect("export_json");

        let content = std::fs::read_to_string(&path).expect("read json");
        let deserialized: Vec<MetricsDataPoint> =
            serde_json::from_str(&content).expect("valid JSON array");
        assert_eq!(deserialized.len(), 5, "Expected 5 points in JSON");
        assert!(
            deserialized[0].cpu_percent > 0.0,
            "cpu_percent should be positive"
        );
    }

    #[test]
    fn test_insert_host_metrics_persists() {
        let conn = setup_test_db();
        let m = HostMetrics {
            cpu_usage_percent: 42.5,
            cpu_cores: 4,
            cpu_per_core: vec![40.0, 45.0, 42.0, 38.0],
            memory_used_bytes: 8_000_000_000,
            memory_total_bytes: 16_000_000_000,
            swap_used_bytes: 500_000_000,
            swap_total_bytes: 2_000_000_000,
            disk_used_bytes: 200_000_000_000,
            disk_total_bytes: 500_000_000_000,
            uptime_seconds: 86400,
            hostname: "test".into(),
            os_name: "Linux".into(),
            kernel_version: "6.8".into(),
        };

        insert_host_metrics(&conn, &m).expect("insert");

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM host_metrics_history", [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(count, 1, "Expected 1 row inserted");

        let cpu: f64 = conn
            .query_row(
                "SELECT cpu_percent FROM host_metrics_history LIMIT 1",
                [],
                |row| row.get(0),
            )
            .expect("read cpu");
        assert!((cpu - 42.5).abs() < 0.01, "Expected cpu_percent 42.5");
    }

    #[test]
    fn test_insert_container_stats_persists() {
        let conn = setup_test_db();
        let s = ContainerStats {
            container_id: "abc123".into(),
            cpu_percentage: 12.3,
            memory_usage_bytes: 256_000_000,
            memory_limit_bytes: 512_000_000,
            memory_percentage: 50.0,
            network_rx_bytes: 10_000,
            network_tx_bytes: 5_000,
            block_read_bytes: 1_000_000,
            block_write_bytes: 500_000,
            pids: 10,
            timestamp: 1_700_000_000,
        };

        insert_container_stats(&conn, &s).expect("insert");

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM container_metrics_history",
                [],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(count, 1, "Expected 1 row inserted");
    }
}
