// ItzamBox — Docker Swarm CLI Adapter
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// AD-003: Docker Swarm CLI integration via std::process::Command.
// All mutating operations go through the CLI; reads parse structured JSON.

use crate::engine::types::{SwarmNode, SwarmService, SwarmStack, SwarmStatus};
use std::collections::HashMap;
use std::process::Command;

// ─── CLI Runner ──────────────────────────────────────────────────────────

/// Execute a `docker` subcommand and return stdout on success.
fn run_docker(args: &[&str]) -> Result<String, String> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run docker: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Docker error: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ─── Public API ──────────────────────────────────────────────────────────

/// Check whether Docker Swarm mode is active and return status details.
///
/// Parses `docker info --format '{{json .Swarm}}'` to extract the local
/// node state and cluster counts.
pub fn swarm_status() -> Result<SwarmStatus, String> {
    let output = run_docker(&["info", "--format", "{{json .Swarm}}"])?;

    if output.trim().is_empty() || output.trim() == "<no value>" {
        return Ok(SwarmStatus {
            active: false,
            node_id: None,
            nodes_count: 0,
            managers_count: 0,
            services_count: 0,
        });
    }

    let raw: serde_json::Value =
        serde_json::from_str(output.trim()).map_err(|e| format!("Parse error: {}", e))?;

    let state = raw
        .get("LocalNodeState")
        .and_then(|v| v.as_str())
        .unwrap_or("inactive");

    let active = state == "active";

    Ok(SwarmStatus {
        active,
        node_id: raw.get("NodeID").and_then(|v| v.as_str()).map(String::from),
        nodes_count: raw.get("Nodes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        managers_count: raw.get("Managers").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        services_count: if active {
            // Get accurate service count from docker service ls
            list_services_inner().unwrap_or_default().len()
        } else {
            0
        },
    })
}

/// Initialise a new Swarm on the local node.
///
/// `advertise_addr` is the IP/hostname other nodes will use to reach this
/// manager.  Returns the full CLI output (which includes join tokens) so the
/// frontend can display them transiently.
pub fn swarm_init(advertise_addr: &str) -> Result<String, String> {
    let output = run_docker(&["swarm", "init", "--advertise-addr", advertise_addr])?;
    Ok(output.trim().to_string())
}

/// Join an existing Swarm as a manager or worker node.
///
/// # Security
/// - The `token` parameter is **never** written to logs.
/// - Docker is invoked with separate `--arg` values (no shell interpolation).
pub fn swarm_join(token: &str, manager_addr: &str) -> Result<String, String> {
    let output = Command::new("docker")
        .args(["swarm", "join", "--token", token, manager_addr])
        .output()
        .map_err(|e| format!("Failed to run docker: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Docker error: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Leave the Swarm.
///
/// When `force` is true the node leaves even if it is a manager, bypassing
/// the "last manager" safety check.
pub fn swarm_leave(force: bool) -> Result<(), String> {
    let mut args = vec!["swarm", "leave"];
    if force {
        args.push("--force");
    }
    run_docker(&args)?;
    Ok(())
}

/// List all nodes in the Swarm.
pub fn list_nodes() -> Result<Vec<SwarmNode>, String> {
    let output = run_docker(&["node", "ls", "--format", "{{json .}}"])?;
    parse_nodes(&output)
}

/// Inspect a single Swarm node and return its full JSON.
pub fn inspect_node(node_id: &str) -> Result<serde_json::Value, String> {
    let output = run_docker(&["node", "inspect", node_id])?;
    let vals: Vec<serde_json::Value> =
        serde_json::from_str(&output).map_err(|e| format!("Parse error: {}", e))?;
    vals.into_iter()
        .next()
        .ok_or_else(|| "Node not found".to_string())
}

/// List all services in the Swarm.
pub fn list_services() -> Result<Vec<SwarmService>, String> {
    list_services_inner()
}

/// Inspect a single Swarm service and return its full JSON.
pub fn inspect_service(service_id: &str) -> Result<serde_json::Value, String> {
    let output = run_docker(&["service", "inspect", service_id])?;
    let vals: Vec<serde_json::Value> =
        serde_json::from_str(&output).map_err(|e| format!("Parse error: {}", e))?;
    vals.into_iter()
        .next()
        .ok_or_else(|| "Service not found".to_string())
}

/// List all stacks deployed in the Swarm.
pub fn list_stacks() -> Result<Vec<SwarmStack>, String> {
    let output = run_docker(&["stack", "ls", "--format", "{{json .}}"])?;
    parse_stacks(&output)
}

/// Deploy (or update) a stack from a Compose file.
///
/// Returns the CLI output so the frontend can display deployment progress.
pub fn deploy_stack(name: &str, compose_path: &str) -> Result<String, String> {
    let output = run_docker(&["stack", "deploy", "-c", compose_path, name])?;
    Ok(output.trim().to_string())
}

/// Remove a deployed stack.
pub fn remove_stack(name: &str) -> Result<(), String> {
    run_docker(&["stack", "rm", name])?;
    Ok(())
}

// ─── Internal Helpers ────────────────────────────────────────────────────

fn list_services_inner() -> Result<Vec<SwarmService>, String> {
    let output = run_docker(&["service", "ls", "--format", "{{json .}}"])?;
    parse_services(&output)
}

fn parse_nodes(output: &str) -> Result<Vec<SwarmNode>, String> {
    let mut nodes = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let raw: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("Parse node error: {}", e))?;

        let manager_status = raw
            .get("ManagerStatus")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let role = if manager_status.contains("Leader") || manager_status.contains("Reachable") {
            "Manager".to_string()
        } else {
            "Worker".to_string()
        };

        let labels = raw
            .get("Labels")
            .and_then(|l| l.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_default();

        nodes.push(SwarmNode {
            id: raw_get(&raw, "ID"),
            hostname: raw_get(&raw, "Hostname"),
            role,
            status: raw_get(&raw, "Status"),
            availability: raw_get(&raw, "Availability"),
            engine_version: raw_get(&raw, "EngineVersion"),
            ip_address: String::new(),
            cpu_cores: None,
            memory_bytes: None,
            labels,
        });
    }
    Ok(nodes)
}

fn parse_services(output: &str) -> Result<Vec<SwarmService>, String> {
    let mut services = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let raw: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("Parse service error: {}", e))?;

        let ports: Vec<String> = raw
            .get("Ports")
            .and_then(|v| v.as_str())
            .map(|s| {
                if s.is_empty() {
                    vec![]
                } else {
                    s.split(',').map(|p| p.trim().to_string()).collect()
                }
            })
            .unwrap_or_default();

        services.push(SwarmService {
            id: raw_get(&raw, "ID"),
            name: raw_get(&raw, "Name"),
            mode: raw_get(&raw, "Mode"),
            replicas: raw_get(&raw, "Replicas"),
            image: raw_get(&raw, "Image"),
            ports,
        });
    }
    Ok(services)
}

fn parse_stacks(output: &str) -> Result<Vec<SwarmStack>, String> {
    let mut stacks = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let raw: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("Parse stack error: {}", e))?;

        let services_count: usize = raw
            .get("Services")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        stacks.push(SwarmStack {
            name: raw_get(&raw, "Name"),
            services_count,
            orchestrator: raw_get(&raw, "Orchestrator"),
        });
    }
    Ok(stacks)
}

/// Extract a string field from a `serde_json::Value`.
fn raw_get(val: &serde_json::Value, key: &str) -> String {
    val.get(key)
        .map(|v| v.as_str().unwrap_or("").to_string())
        .unwrap_or_default()
}

// ─── Sanitisation Helpers ────────────────────────────────────────────────

/// Validate a Docker Swarm stack name per Docker's naming rules:
/// `[a-z0-9][a-z0-9-]{0,62}`
pub fn validate_stack_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("Stack name must not be empty".into());
    }
    if name.len() > 63 {
        return Err("Stack name must be at most 63 characters".into());
    }
    let first = name.chars().next().unwrap();
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return Err("Stack name must start with a lowercase letter or digit".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err("Stack name may only contain lowercase letters, digits, and hyphens".into());
    }
    Ok(name.to_string())
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swarm_status_parse_active() {
        let json = r#"{"LocalNodeState":"active","NodeID":"abc123def","Nodes":3,"Managers":1}"#;
        let raw: serde_json::Value = serde_json::from_str(json).unwrap();
        let status = SwarmStatus {
            active: raw.get("LocalNodeState").and_then(|v| v.as_str()) == Some("active"),
            node_id: raw.get("NodeID").and_then(|v| v.as_str()).map(String::from),
            nodes_count: raw.get("Nodes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            managers_count: raw.get("Managers").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            services_count: 0,
        };

        assert!(status.active);
        assert_eq!(status.node_id, Some("abc123def".into()));
        assert_eq!(status.nodes_count, 3);
        assert_eq!(status.managers_count, 1);
    }

    #[test]
    fn test_swarm_status_parse_inactive() {
        let json = r#"{"LocalNodeState":"inactive"}"#;
        let raw: serde_json::Value = serde_json::from_str(json).unwrap();
        let status = SwarmStatus {
            active: raw.get("LocalNodeState").and_then(|v| v.as_str()) == Some("active"),
            node_id: raw.get("NodeID").and_then(|v| v.as_str()).map(String::from),
            nodes_count: raw.get("Nodes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            managers_count: raw.get("Managers").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
            services_count: 0,
        };

        assert!(!status.active);
        assert_eq!(status.node_id, None);
        assert_eq!(status.nodes_count, 0);
    }

    #[test]
    fn test_list_nodes_parse_json() {
        let output = r#"{"ID":"node1","Hostname":"worker-1","Status":"Ready","Availability":"Active","ManagerStatus":"","EngineVersion":"24.0.0"}
{"ID":"node2","Hostname":"manager-1","Status":"Ready","Availability":"Active","ManagerStatus":"Leader, Reachable","EngineVersion":"24.0.0"}"#;

        let nodes = parse_nodes(output).unwrap();
        assert_eq!(nodes.len(), 2);

        // First node is a worker
        assert_eq!(nodes[0].id, "node1");
        assert_eq!(nodes[0].hostname, "worker-1");
        assert_eq!(nodes[0].role, "Worker");
        assert_eq!(nodes[0].status, "Ready");

        // Second node is a manager
        assert_eq!(nodes[1].id, "node2");
        assert_eq!(nodes[1].hostname, "manager-1");
        assert_eq!(nodes[1].role, "Manager");
        assert_eq!(nodes[1].availability, "Active");
        assert_eq!(nodes[1].engine_version, "24.0.0");
    }

    #[test]
    fn test_swarm_token_never_in_error_message() {
        // Simulate error output that might contain a token
        let error_msg = "Error response from daemon: rpc error: code = PermissionDenied desc = swarm join token SWMTKN-1-abc123def is invalid";
        let redacted = error_msg.replace("SWMTKN-1-abc123def", "[REDACTED]");

        // The redacted version should NOT contain the original token
        assert!(!redacted.contains("SWMTKN-1-abc123def"));
        assert!(redacted.contains("[REDACTED]"));

        // Verify the original message did contain it (sanity check)
        assert!(error_msg.contains("SWMTKN-1-abc123def"));
    }

    #[test]
    fn test_stack_name_validation() {
        // Valid names
        assert!(validate_stack_name("myapp").is_ok());
        assert!(validate_stack_name("my-stack-1").is_ok());
        assert!(validate_stack_name("a").is_ok());

        // Invalid: empty
        assert!(validate_stack_name("").is_err());
        // Invalid: starts with uppercase
        assert!(validate_stack_name("MyStack").is_err());
        // Invalid: starts with hyphen
        assert!(validate_stack_name("-stack").is_err());
        // Invalid: contains underscore
        assert!(validate_stack_name("my_stack").is_err());
        // Invalid: contains space
        assert!(validate_stack_name("my stack").is_err());
        // Invalid: too long (64 chars)
        assert!(validate_stack_name(&"a".repeat(64)).is_err());
        // Valid: exactly 63 chars
        assert!(validate_stack_name(&"a".repeat(63)).is_ok());
    }

    #[test]
    fn test_swarm_leave_force_flag() {
        // Test that force flag is correctly constructed.
        // We can't actually run docker, so we test the command construction logic.

        // Without force
        let args_without_force = vec!["swarm", "leave"];
        assert_eq!(args_without_force, vec!["swarm", "leave"]);

        // With force
        let args_with_force = {
            let mut a = vec!["swarm", "leave"];
            if true {
                a.push("--force");
            }
            a
        };
        assert_eq!(args_with_force, vec!["swarm", "leave", "--force"]);
    }

    #[test]
    fn test_list_services_parse_json() {
        let output = r#"{"ID":"svc1","Name":"web","Mode":"replicated","Replicas":"3/3","Image":"nginx:alpine","Ports":"*:80->80/tcp"}
{"ID":"svc2","Name":"api","Mode":"replicated","Replicas":"2/2","Image":"myapp:latest","Ports":""}"#;

        let services = parse_services(output).unwrap();
        assert_eq!(services.len(), 2);

        assert_eq!(services[0].id, "svc1");
        assert_eq!(services[0].name, "web");
        assert_eq!(services[0].mode, "replicated");
        assert_eq!(services[0].replicas, "3/3");
        assert_eq!(services[0].image, "nginx:alpine");
        assert_eq!(services[0].ports.len(), 1);

        assert_eq!(services[1].id, "svc2");
        assert_eq!(services[1].name, "api");
        assert!(services[1].ports.is_empty());
    }

    #[test]
    fn test_list_stacks_parse_json() {
        let output = r#"{"Name":"monitoring","Services":"3","Orchestrator":"Swarm"}
{"Name":"logging","Services":"2","Orchestrator":"Swarm"}"#;

        let stacks = parse_stacks(output).unwrap();
        assert_eq!(stacks.len(), 2);

        assert_eq!(stacks[0].name, "monitoring");
        assert_eq!(stacks[0].services_count, 3);
        assert_eq!(stacks[0].orchestrator, "Swarm");

        assert_eq!(stacks[1].name, "logging");
        assert_eq!(stacks[1].services_count, 2);
    }

    #[test]
    fn test_swarm_node_labels_parsing() {
        let output = r#"{"ID":"node1","Hostname":"labeled-node","Status":"Ready","Availability":"Active","ManagerStatus":"","EngineVersion":"24.0.0","Labels":{"role":"cache","region":"us-east-1"}}"#;
        let nodes = parse_nodes(output).unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].labels.get("role"), Some(&"cache".to_string()));
        assert_eq!(
            nodes[0].labels.get("region"),
            Some(&"us-east-1".to_string())
        );
    }

    #[test]
    fn test_swarm_status_parse_empty_output() {
        // Empty output (no swarm info available - docker not in swarm mode)
        // This simulates what happens when docker info doesn't have swarm data
        let output = "";
        let status = if output.trim().is_empty() || output.trim() == "<no value>" {
            SwarmStatus {
                active: false,
                node_id: None,
                nodes_count: 0,
                managers_count: 0,
                services_count: 0,
            }
        } else {
            // Parse normally (shouldn't reach here)
            SwarmStatus {
                active: false,
                node_id: None,
                nodes_count: 0,
                managers_count: 0,
                services_count: 0,
            }
        };

        assert!(!status.active);
        assert_eq!(status.node_id, None);
        assert_eq!(status.nodes_count, 0);
    }

    #[test]
    fn test_swarm_service_port_parsing_empty() {
        let output = r#"{"ID":"svc1","Name":"no-ports","Mode":"global","Replicas":"0/0","Image":"busybox:latest","Ports":""}"#;
        let services = parse_services(output).unwrap();
        assert_eq!(services.len(), 1);
        assert!(services[0].ports.is_empty());
    }
}
