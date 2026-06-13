-- ItzamBox — Container Templates Schema
-- Copyright (C) 2026 SodigTech — GPL-3.0
-- Migration: 002_templates.sql

CREATE TABLE IF NOT EXISTS container_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    image TEXT NOT NULL,
    default_ports TEXT DEFAULT '[]',
    default_volumes TEXT DEFAULT '[]',
    default_env TEXT DEFAULT '[]',
    default_network TEXT DEFAULT 'bridge',
    default_restart TEXT DEFAULT 'unless-stopped',
    default_command TEXT,
    is_builtin BOOLEAN DEFAULT 0,
    category TEXT DEFAULT 'custom',
    icon TEXT DEFAULT 'fa-cube',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
