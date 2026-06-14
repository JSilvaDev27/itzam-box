// ItzamBox — Container Templates Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ContainerTemplate {
    pub id: Option<i64>,
    pub name: String,
    pub description: String,
    pub image: String,
    pub default_ports: String,
    pub default_volumes: String,
    pub default_env: String,
    pub default_network: String,
    pub default_restart: String,
    pub default_command: Option<String>,
    pub is_builtin: bool,
    pub category: String,
    pub icon: String,
}

/// List all container templates, ordered by built-in first, then category, then name.
#[tauri::command]
pub async fn list_templates(state: State<'_, AppState>) -> Result<Vec<ContainerTemplate>, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    let mut stmt = db
        .prepare(
            "SELECT id, name, description, image, default_ports, default_volumes, \
             default_env, default_network, default_restart, default_command, \
             is_builtin, category, icon \
             FROM container_templates \
             ORDER BY is_builtin DESC, category, name",
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ContainerTemplate {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                image: row.get(3)?,
                default_ports: row.get(4)?,
                default_volumes: row.get(5)?,
                default_env: row.get(6)?,
                default_network: row.get(7)?,
                default_restart: row.get(8)?,
                default_command: row.get(9)?,
                is_builtin: row.get::<_, i32>(10)? != 0,
                category: row.get(11)?,
                icon: row.get(12)?,
            })
        })
        .map_err(|e| format!("Failed to query templates: {}", e))?;

    let mut templates = Vec::new();
    for row in rows {
        templates.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
    }

    Ok(templates)
}

/// Get a single template by ID.
#[tauri::command]
pub async fn get_template(
    state: State<'_, AppState>,
    id: i64,
) -> Result<ContainerTemplate, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    db.query_row(
        "SELECT id, name, description, image, default_ports, default_volumes, \
         default_env, default_network, default_restart, default_command, \
         is_builtin, category, icon \
         FROM container_templates WHERE id = ?1",
        rusqlite::params![id],
        |row| {
            Ok(ContainerTemplate {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                description: row.get(2)?,
                image: row.get(3)?,
                default_ports: row.get(4)?,
                default_volumes: row.get(5)?,
                default_env: row.get(6)?,
                default_network: row.get(7)?,
                default_restart: row.get(8)?,
                default_command: row.get(9)?,
                is_builtin: row.get::<_, i32>(10)? != 0,
                category: row.get(11)?,
                icon: row.get(12)?,
            })
        },
    )
    .map_err(|e| format!("Template not found: {}", e))
}

/// Save (insert or update) a container template.
/// If `template.id` is `Some`, performs an UPDATE.
/// If `template.id` is `None`, performs an INSERT (always sets is_builtin=0 for user-created).
#[tauri::command]
pub async fn save_template(
    state: State<'_, AppState>,
    template: ContainerTemplate,
) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(id) = template.id {
        // UPDATE existing template (but never update built-ins)
        let is_builtin: i32 = db
            .query_row(
                "SELECT is_builtin FROM container_templates WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .map_err(|_| format!("Template with id {} not found", id))?;

        if is_builtin != 0 {
            return Err("Cannot modify built-in templates".to_string());
        }

        db.execute(
            "UPDATE container_templates SET \
             name = ?1, description = ?2, image = ?3, \
             default_ports = ?4, default_volumes = ?5, default_env = ?6, \
             default_network = ?7, default_restart = ?8, default_command = ?9, \
             category = ?10, icon = ?11 \
             WHERE id = ?12 AND is_builtin = 0",
            rusqlite::params![
                template.name,
                template.description,
                template.image,
                template.default_ports,
                template.default_volumes,
                template.default_env,
                template.default_network,
                template.default_restart,
                template.default_command,
                template.category,
                template.icon,
                id,
            ],
        )
        .map_err(|e| format!("Failed to update template: {}", e))?;

        Ok(id)
    } else {
        // INSERT new user template (always non-built-in)
        db.execute(
            "INSERT INTO container_templates \
             (name, description, image, default_ports, default_volumes, default_env, \
              default_network, default_restart, default_command, is_builtin, category, icon) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, ?10, ?11)",
            rusqlite::params![
                template.name,
                template.description,
                template.image,
                template.default_ports,
                template.default_volumes,
                template.default_env,
                template.default_network,
                template.default_restart,
                template.default_command,
                template.category,
                template.icon,
            ],
        )
        .map_err(|e| format!("Failed to insert template: {}", e))?;

        Ok(db.last_insert_rowid())
    }
}

/// Delete a container template. Built-in templates cannot be deleted.
#[tauri::command]
pub async fn delete_template(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    // Check if it's a built-in
    let is_builtin: i32 = db
        .query_row(
            "SELECT is_builtin FROM container_templates WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .map_err(|_| format!("Template with id {} not found", id))?;

    if is_builtin != 0 {
        return Err("Cannot delete built-in templates".to_string());
    }

    db.execute(
        "DELETE FROM container_templates WHERE id = ?1 AND is_builtin = 0",
        rusqlite::params![id],
    )
    .map_err(|e| format!("Failed to delete template: {}", e))?;

    Ok(())
}

/// Seed the 7 built-in templates if they don't already exist (INSERT OR IGNORE).
#[tauri::command]
pub async fn seed_builtin_templates(state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| format!("Lock error: {}", e))?;

    // 1. Nginx
    db.execute(
        "INSERT OR IGNORE INTO container_templates \
         (name, description, image, default_ports, default_volumes, default_env, \
          default_network, default_restart, default_command, is_builtin, category, icon) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
        rusqlite::params![
            "Nginx",
            "High-performance web server and reverse proxy",
            "nginx:latest",
            r#"[{"host":"8080","container":"80","protocol":"tcp"}]"#,
            "[]",
            "[]",
            "bridge",
            "unless-stopped",
            Option::<&str>::None,
            "web",
            "fa-globe",
        ],
    )
    .map_err(|e| format!("Failed to seed Nginx: {}", e))?;

    // 2. PostgreSQL
    db.execute(
        "INSERT OR IGNORE INTO container_templates \
         (name, description, image, default_ports, default_volumes, default_env, \
          default_network, default_restart, default_command, is_builtin, category, icon) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
        rusqlite::params![
            "PostgreSQL",
            "Powerful, open-source relational database system",
            "postgres:16",
            r#"[{"host":"5432","container":"5432","protocol":"tcp"}]"#,
            r#"["postgres-data:/var/lib/postgresql/data"]"#,
            r#"["POSTGRES_PASSWORD=changeme"]"#,
            "bridge",
            "unless-stopped",
            Option::<&str>::None,
            "database",
            "fa-database",
        ],
    )
    .map_err(|e| format!("Failed to seed PostgreSQL: {}", e))?;

    // 3. Redis
    db.execute(
        "INSERT OR IGNORE INTO container_templates \
         (name, description, image, default_ports, default_volumes, default_env, \
          default_network, default_restart, default_command, is_builtin, category, icon) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
        rusqlite::params![
            "Redis",
            "In-memory data structure store, used as cache and message broker",
            "redis:7-alpine",
            r#"[{"host":"6379","container":"6379","protocol":"tcp"}]"#,
            "[]",
            "[]",
            "bridge",
            "unless-stopped",
            Option::<&str>::None,
            "cache",
            "fa-registered",
        ],
    )
    .map_err(|e| format!("Failed to seed Redis: {}", e))?;

    // 4. MySQL
    db.execute(
        "INSERT OR IGNORE INTO container_templates \
         (name, description, image, default_ports, default_volumes, default_env, \
          default_network, default_restart, default_command, is_builtin, category, icon) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
        rusqlite::params![
            "MySQL",
            "Popular open-source relational database management system",
            "mysql:8.4",
            r#"[{"host":"3306","container":"3306","protocol":"tcp"}]"#,
            r#"["mysql-data:/var/lib/mysql"]"#,
            r#"["MYSQL_ROOT_PASSWORD=changeme"]"#,
            "bridge",
            "unless-stopped",
            Option::<&str>::None,
            "database",
            "fa-database",
        ],
    )
    .map_err(|e| format!("Failed to seed MySQL: {}", e))?;

    // 5. MongoDB
    db.execute(
        "INSERT OR IGNORE INTO container_templates \
         (name, description, image, default_ports, default_volumes, default_env, \
          default_network, default_restart, default_command, is_builtin, category, icon) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
        rusqlite::params![
            "MongoDB",
            "NoSQL document database with high scalability",
            "mongo:7",
            r#"[{"host":"27017","container":"27017","protocol":"tcp"}]"#,
            r#"["mongo-data:/data/db"]"#,
            r#"["MONGO_INITDB_ROOT_USERNAME=admin","MONGO_INITDB_ROOT_PASSWORD=changeme"]"#,
            "bridge",
            "unless-stopped",
            Option::<&str>::None,
            "database",
            "fa-leaf",
        ],
    )
    .map_err(|e| format!("Failed to seed MongoDB: {}", e))?;

    // 6. Node.js
    db.execute(
        "INSERT OR IGNORE INTO container_templates \
         (name, description, image, default_ports, default_volumes, default_env, \
          default_network, default_restart, default_command, is_builtin, category, icon) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
        rusqlite::params![
            "Node.js",
            "JavaScript runtime built on Chrome's V8 engine",
            "node:22-alpine",
            r#"[{"host":"3000","container":"3000","protocol":"tcp"}]"#,
            r#"["./app:/app"]"#,
            r#"["NODE_ENV=development"]"#,
            "bridge",
            "unless-stopped",
            Some("node"),
            "runtime",
            "fa-brands fa-node-js",
        ],
    )
    .map_err(|e| format!("Failed to seed Node.js: {}", e))?;

    // 7. Python
    db.execute(
        "INSERT OR IGNORE INTO container_templates \
         (name, description, image, default_ports, default_volumes, default_env, \
          default_network, default_restart, default_command, is_builtin, category, icon) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10, ?11)",
        rusqlite::params![
            "Python",
            "General-purpose programming language runtime",
            "python:3.12-alpine",
            r#"[{"host":"8000","container":"8000","protocol":"tcp"}]"#,
            r#"["./app:/app"]"#,
            "[]",
            "bridge",
            "unless-stopped",
            Some("python3"),
            "runtime",
            "fa-brands fa-python",
        ],
    )
    .map_err(|e| format!("Failed to seed Python: {}", e))?;

    log::info!("Built-in templates seeded successfully");
    Ok(())
}
