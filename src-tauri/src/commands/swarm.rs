// ItzamBox — Swarm Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0

use crate::engine::swarm;
use crate::engine::types::{SwarmNode, SwarmService, SwarmStack, SwarmStatus};
use crate::utils::sanitizer::sanitize_path;
use tauri::command;

// ─── Status ──────────────────────────────────────────────────────────────

/// Return the current Docker Swarm status (active/inactive, node info, counts).
#[command]
pub fn swarm_status() -> Result<SwarmStatus, String> {
    swarm::swarm_status()
}

// ─── Init / Join / Leave ─────────────────────────────────────────────────

/// Initialise a new Swarm on the local node.
///
/// `advertise_addr` is validated to be non-empty.  The CLI output (containing
/// join tokens) is returned for transient display in the UI.
#[command]
pub fn swarm_init(advertise_addr: String) -> Result<String, String> {
    if advertise_addr.trim().is_empty() {
        return Err("Advertise address must not be empty".into());
    }
    log::info!("swarm_init: advertise_addr={}", advertise_addr);
    swarm::swarm_init(advertise_addr.trim())
}

/// Join an existing Swarm as a worker or manager node.
///
/// # Security
/// The join token is **never** written to logs.  If the operation fails the
/// error message is sanitised to remove any token-like content.
#[command]
pub fn swarm_join(token: String, manager_addr: String) -> Result<String, String> {
    if token.trim().is_empty() {
        return Err("Join token must not be empty".into());
    }
    if manager_addr.trim().is_empty() {
        return Err("Manager address must not be empty".into());
    }
    // Token is NEVER logged — explicitly redacted in debug output
    log::info!("swarm_join: manager_addr={}", manager_addr);
    log::debug!(
        "swarm_join: token=[REDACTED], manager_addr={}",
        manager_addr
    );
    match swarm::swarm_join(token.trim(), manager_addr.trim()) {
        Ok(output) => Ok(output),
        Err(e) => {
            // Sanitise any token-like content from the error before returning
            let sanitised = sanitise_error(&e);
            Err(sanitised)
        }
    }
}

/// Leave the Swarm.  When `force` is true, leave even as a last manager.
#[command]
pub fn swarm_leave(force: bool) -> Result<(), String> {
    log::info!("swarm_leave: force={}", force);
    swarm::swarm_leave(force)
}

// ─── Nodes ───────────────────────────────────────────────────────────────

/// List all nodes in the Swarm.
#[command]
pub fn list_swarm_nodes() -> Result<Vec<SwarmNode>, String> {
    swarm::list_nodes()
}

/// Inspect a single Swarm node and return its full JSON.
#[command]
pub fn inspect_swarm_node(node_id: String) -> Result<serde_json::Value, String> {
    if node_id.trim().is_empty() {
        return Err("Node ID must not be empty".into());
    }
    swarm::inspect_node(node_id.trim())
}

// ─── Services ────────────────────────────────────────────────────────────

/// List all services in the Swarm.
#[command]
pub fn list_swarm_services() -> Result<Vec<SwarmService>, String> {
    swarm::list_services()
}

/// Inspect a single Swarm service and return its full JSON.
#[command]
pub fn inspect_swarm_service(service_id: String) -> Result<serde_json::Value, String> {
    if service_id.trim().is_empty() {
        return Err("Service ID must not be empty".into());
    }
    swarm::inspect_service(service_id.trim())
}

// ─── Stacks ──────────────────────────────────────────────────────────────

/// List all deployed stacks.
#[command]
pub fn list_stacks() -> Result<Vec<SwarmStack>, String> {
    swarm::list_stacks()
}

/// Deploy (or update) a stack from a Compose file.
///
/// - `name` is validated against the Docker stack naming rules.
/// - `compose_path` is sanitised to prevent path traversal.
#[command]
pub fn deploy_stack(name: String, compose_path: String) -> Result<String, String> {
    let name = swarm::validate_stack_name(name.trim())?;
    let compose_path = sanitize_path(compose_path.trim())?;
    log::info!("deploy_stack: name={}, compose_path={}", name, compose_path);
    swarm::deploy_stack(&name, &compose_path)
}

/// Remove a deployed stack.
#[command]
pub fn remove_stack(name: String) -> Result<(), String> {
    let name = swarm::validate_stack_name(name.trim())?;
    log::info!("remove_stack: name={}", name);
    swarm::remove_stack(&name)
}

// ─── Internal ────────────────────────────────────────────────────────────

/// Remove Swarm join tokens from an error message to prevent accidental
/// logging or display of sensitive credentials.
fn sanitise_error(msg: &str) -> String {
    // Docker swarm tokens start with "SWMTKN-1-" followed by base64-ish data
    let mut result = msg.to_string();
    // Match any occurrence of the SWMTKN pattern and redact it
    if let Some(start) = result.find("SWMTKN-") {
        // Find end of token — tokens end at a space, comma, or end of string
        let remaining = &result[start..];
        let end = remaining
            .find([' ', ',', ')', '"'])
            .unwrap_or(remaining.len());
        let token = &remaining[..end];
        result = result.replace(token, "[REDACTED]");
    }
    result
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitise_error_with_token() {
        let err =
            "Error response from daemon: rpc error: code = 16 desc = SWMTKN-1-abc123def is invalid";
        let sanitised = sanitise_error(err);
        assert!(!sanitised.contains("SWMTKN-1-abc123def"));
        assert!(sanitised.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitise_error_without_token() {
        let err = "Error: this is a normal error message";
        let sanitised = sanitise_error(err);
        assert_eq!(sanitised, err);
    }

    #[test]
    fn test_sanitise_error_empty() {
        assert_eq!(sanitise_error(""), "");
    }

    #[test]
    fn test_deploy_stack_validation_empty_name() {
        let result = deploy_stack("".into(), "/tmp/compose.yml".into());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_deploy_stack_validation_invalid_name() {
        let result = deploy_stack("InvalidStack".into(), "/tmp/compose.yml".into());
        assert!(result.is_err());
    }
}
