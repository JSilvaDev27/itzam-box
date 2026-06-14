-- ItzamBox — K8s Resource Cache Table (Migration 004)
-- Copyright (C) 2026 SodigTech — GPL-3.0
--
-- Cache K8s resource data for offline viewing and performance.
-- Supports stale-data display when cluster is unreachable (US-20 Scenario 5).

CREATE TABLE IF NOT EXISTS k8s_resource_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    context_name TEXT NOT NULL,
    namespace TEXT NOT NULL DEFAULT 'default',
    resource_kind TEXT NOT NULL,       -- 'pod', 'deployment', 'service', 'configmap', 'secret'
    resource_name TEXT NOT NULL,
    resource_json TEXT NOT NULL,        -- Full JSON from kubectl -o json
    cached_at INTEGER NOT NULL,         -- Unix epoch seconds
    UNIQUE(context_name, namespace, resource_kind, resource_name)
);

CREATE INDEX IF NOT EXISTS idx_k8s_cache_lookup
    ON k8s_resource_cache(context_name, namespace, resource_kind);

CREATE INDEX IF NOT EXISTS idx_k8s_cache_staleness
    ON k8s_resource_cache(cached_at);
