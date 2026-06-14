// ItzamBox — Kubernetes kubectl Engine (Read-Only CLI Adapter)
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// ADR-002: kubectl CLI over client library.  All commands are read-only.
// Blocklist prevents execution of 20 dangerous subcommands.

use crate::engine::types::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
use tokio::time::{timeout, Duration};

// ─── Blocklist ─────────────────────────────────────────────────────────────

/// Subcommands that are **never** allowed, regardless of context.
const KUBECTL_BLOCKLIST: &[&str] = &[
    "apply",
    "delete",
    "create",
    "edit",
    "patch",
    "replace",
    "rollout",
    "scale",
    "expose",
    "port-forward",
    "exec",
    "cp",
    "drain",
    "cordon",
    "uncordon",
    "taint",
    "attach",
    "proxy",
    "debug",
    "auth",
];

const KUBECTL_TIMEOUT_SECS: u64 = 15;

// ─── Kubeconfig YAML Structures ────────────────────────────────────────────

#[derive(Deserialize)]
#[allow(dead_code)]
struct KubeconfigYaml {
    #[serde(default)]
    contexts: Vec<KubeconfigContextEntry>,
    #[serde(default)]
    clusters: Vec<KubeconfigClusterEntry>,
    #[serde(default)]
    users: Vec<KubeconfigUserEntry>,
    #[serde(rename = "current-context")]
    #[serde(default)]
    current_context: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct KubeconfigContextEntry {
    name: String,
    context: KubeconfigContextDetail,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct KubeconfigContextDetail {
    cluster: String,
    user: String,
    #[serde(default)]
    namespace: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct KubeconfigClusterEntry {
    name: String,
    cluster: serde_yaml::Value,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct KubeconfigUserEntry {
    name: String,
    user: serde_yaml::Value,
}

// ─── kubectl Binary Detection ──────────────────────────────────────────────

/// Return `true` iff the `kubectl` binary is on `$PATH`.
pub fn detect_kubectl_binary() -> bool {
    Command::new("which")
        .arg("kubectl")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Return the installed kubectl version string (first line of `kubectl version --client`).
pub fn kubectl_version() -> Option<String> {
    let out = Command::new("kubectl")
        .args(["version", "--client", "-o", "json"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    // Parse "clientVersion.gitVersion" from JSON
    #[derive(Deserialize)]
    struct ClientVersion {
        #[serde(rename = "clientVersion")]
        client_version: GitVersion,
    }
    #[derive(Deserialize)]
    struct GitVersion {
        #[serde(rename = "gitVersion")]
        git_version: String,
    }
    let parsed: ClientVersion = serde_json::from_slice(&out.stdout).ok()?;
    Some(parsed.client_version.git_version)
}

/// Convenience: check everything and return a status struct.
pub async fn detect_kubectl() -> KubectlStatus {
    let installed = detect_kubectl_binary();
    let version = if installed { kubectl_version() } else { None };
    let (kubeconfig_parsed, contexts_count) = if installed {
        match parse_kubeconfig() {
            Ok(ctxs) => (true, ctxs.len()),
            Err(_) => (false, 0),
        }
    } else {
        (false, 0)
    };

    KubectlStatus {
        installed,
        version,
        kubeconfig_parsed,
        contexts_count,
    }
}

// ─── Kubeconfig Parsing ────────────────────────────────────────────────────

/// Read and parse the kubeconfig file, returning the list of contexts.
pub fn parse_kubeconfig() -> Result<Vec<K8sContext>, String> {
    let path = resolve_kubeconfig_path();
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read kubeconfig at {}: {}", path.display(), e))?;

    let kc: KubeconfigYaml = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse kubeconfig YAML: {}", e))?;

    let current = &kc.current_context;

    let contexts: Vec<K8sContext> = kc
        .contexts
        .iter()
        .map(|entry| K8sContext {
            name: entry.name.clone(),
            cluster: entry.context.cluster.clone(),
            user: entry.context.user.clone(),
            is_active: entry.name == *current,
        })
        .collect();

    if contexts.is_empty() {
        return Err("No contexts found in kubeconfig".into());
    }

    Ok(contexts)
}

fn resolve_kubeconfig_path() -> std::path::PathBuf {
    if let Ok(path) = std::env::var("KUBECONFIG") {
        if !path.is_empty() {
            return std::path::PathBuf::from(path);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home).join(".kube").join("config");
    }
    std::path::PathBuf::from(".kube/config")
}

// ─── Argument Validation (Blocklist Enforcement) ──────────────────────────

/// Validate that **none** of the args refer to a blocked subcommand.
/// Returns `Ok(())` if all args pass.
pub fn validate_kubectl_args(args: &[String]) -> Result<(), String> {
    for arg in args {
        let normalized = arg.to_lowercase();
        if KUBECTL_BLOCKLIST.iter().any(|blocked| {
            normalized == *blocked || normalized.starts_with(&format!("{}=", blocked))
        }) {
            return Err(format!(
                "Blocked kubectl subcommand: '{}' — only read-only commands are allowed",
                arg
            ));
        }
    }
    Ok(())
}

// ─── Core Executor ─────────────────────────────────────────────────────────

/// Run `kubectl` with the given args, parse JSON output, return as `serde_json::Value`.
/// The subcommand blocklist is enforced before execution.
/// A 15-second timeout protects against hung clusters.
pub async fn execute_kubectl(args: &[&str]) -> Result<serde_json::Value, String> {
    // Convert to owned Strings for blocklist validation and the closure
    let owned: Vec<String> = args.iter().map(|a| a.to_string()).collect();
    validate_kubectl_args(&owned)?;

    let args_clone = owned.clone();

    let fut = tokio::task::spawn_blocking(move || {
        let output = Command::new("kubectl")
            .args(&args_clone)
            .output()
            .map_err(|e| format!("Failed to execute kubectl: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let combined = if stderr.is_empty() { stdout } else { stderr };
            return Err(format!(
                "kubectl error (exit code {:?}): {}",
                output.status.code(),
                combined
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return Ok(serde_json::Value::Null);
        }

        serde_json::from_str(&stdout)
            .map_err(|e| format!("Failed to parse kubectl JSON output: {}", e))
    });

    timeout(Duration::from_secs(KUBECTL_TIMEOUT_SECS), fut)
        .await
        .map_err(|_| format!("kubectl command timed out after {}s", KUBECTL_TIMEOUT_SECS))?
        .map_err(|e| format!("Task panic: {}", e))?
}

// ─── Resource Listing ──────────────────────────────────────────────────────

/// Parse the kubectl JSON list response items array.
fn parse_items(json: &serde_json::Value) -> Vec<serde_json::Value> {
    json.get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

/// Extract metadata name from a resource JSON object.
fn meta_name(item: &serde_json::Value) -> String {
    item["metadata"]["name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string()
}

/// Extract age from metadata.creationTimestamp.
fn calc_age(item: &serde_json::Value) -> String {
    item["metadata"]["creationTimestamp"]
        .as_str()
        .map(|ts| {
            // Attempt to parse RFC 3339 and compute relative age
            if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts) {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(parsed);
                let secs = duration.num_seconds().max(0);
                if secs < 60 {
                    format!("{}s", secs)
                } else if secs < 3600 {
                    format!("{}m", secs / 60)
                } else if secs < 86400 {
                    format!("{}h", secs / 3600)
                } else {
                    format!("{}d", secs / 86400)
                }
            } else {
                ts.to_string()
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Extract namespace from metadata.
fn meta_namespace(item: &serde_json::Value) -> String {
    item["metadata"]["namespace"]
        .as_str()
        .unwrap_or("default")
        .to_string()
}

/// Extract labels from metadata.
fn meta_labels(item: &serde_json::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(labels) = item["metadata"]["labels"].as_object() {
        for (k, v) in labels {
            if let Some(val) = v.as_str() {
                map.insert(k.clone(), val.to_string());
            }
        }
    }
    map
}

// ─── Contexts ──────────────────────────────────────────────────────────────

/// List available contexts (parsed from kubeconfig).
pub async fn list_contexts() -> Result<Vec<K8sContext>, String> {
    parse_kubeconfig()
}

/// Switch the active kubectl context.
pub async fn set_context(context_name: &str) -> Result<(), String> {
    let out = Command::new("kubectl")
        .args(["config", "use-context", context_name])
        .output()
        .map_err(|e| format!("Failed to run kubectl config use-context: {}", e))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(format!("Failed to switch context: {}", stderr));
    }
    Ok(())
}

// ─── Namespaces ────────────────────────────────────────────────────────────

/// List all namespaces (sorted by name).
pub async fn list_namespaces() -> Result<Vec<String>, String> {
    let json = execute_kubectl(&["get", "namespaces", "-o", "json"]).await?;
    let items = parse_items(&json);

    let mut namespaces: Vec<String> = items
        .iter()
        .map(|item| {
            item["metadata"]["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string()
        })
        .collect();

    namespaces.sort();
    namespaces.dedup();
    Ok(namespaces)
}

// ─── Pods ──────────────────────────────────────────────────────────────────

/// List pods in the given namespace (or all namespaces if `namespace` is None).
pub async fn list_pods(namespace: Option<&str>) -> Result<Vec<K8sPod>, String> {
    let mut args = vec!["get", "pods", "-o", "json"];
    if let Some(ns) = namespace {
        args.push("-n");
        args.push(ns);
    }

    let json = execute_kubectl(&args).await?;
    let items = parse_items(&json);

    let pods: Vec<K8sPod> = items
        .iter()
        .map(|item| {
            let status = item["status"]["phase"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string();

            // Check for more specific status from containerStatuses
            let (restarts, containers) = parse_container_statuses(item);
            let effective_status = if status == "Running" {
                derive_pod_status(item)
            } else {
                status
            };

            K8sPod {
                name: meta_name(item),
                namespace: meta_namespace(item),
                status: effective_status,
                restarts,
                age: calc_age(item),
                node: item["spec"]["nodeName"].as_str().unwrap_or("").to_string(),
                ip: item["status"]["podIP"].as_str().map(|s| s.to_string()),
                containers,
                labels: meta_labels(item),
                annotations: extract_annotations(item),
            }
        })
        .collect();

    Ok(pods)
}

/// Extract container statuses from pod JSON.
fn parse_container_statuses(item: &serde_json::Value) -> (i32, Vec<K8sContainer>) {
    let mut total_restarts = 0i32;
    let mut containers = Vec::new();

    if let Some(statuses) = item["status"]["containerStatuses"].as_array() {
        for cs in statuses {
            let restart_count = cs["restartCount"].as_i64().unwrap_or(0) as i32;
            total_restarts += restart_count;

            let ready = cs["ready"].as_bool().unwrap_or(false);

            // Parse ports from spec
            let container_name = cs["name"].as_str().unwrap_or("").to_string();
            let ports = extract_container_ports(item, &container_name);

            containers.push(K8sContainer {
                name: container_name.clone(),
                image: cs["image"].as_str().unwrap_or("").to_string(),
                ports,
                ready,
                restart_count,
                readiness_probe: extract_probe(item, &container_name, "readinessProbe"),
                liveness_probe: extract_probe(item, &container_name, "livenessProbe"),
                resources_limits: extract_resources(item, &container_name, "limits"),
                resources_requests: extract_resources(item, &container_name, "requests"),
            });
        }
    }

    (total_restarts, containers)
}

/// Derive a more specific status from container statuses (e.g., CrashLoopBackOff).
fn derive_pod_status(item: &serde_json::Value) -> String {
    if let Some(statuses) = item["status"]["containerStatuses"].as_array() {
        for cs in statuses {
            if let Some(state) = cs["state"].as_object() {
                for (key, _val) in state {
                    match key.as_str() {
                        "waiting" => {
                            let reason = cs["state"]["waiting"]["reason"]
                                .as_str()
                                .unwrap_or("Waiting");
                            return reason.to_string();
                        }
                        "terminated" => {
                            return "Terminated".to_string();
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    "Running".to_string()
}

/// Extract container ports from the pod spec.
fn extract_container_ports(item: &serde_json::Value, container_name: &str) -> Vec<String> {
    let mut ports = Vec::new();
    if let Some(containers) = item["spec"]["containers"].as_array() {
        for c in containers {
            if c["name"].as_str() == Some(container_name) {
                if let Some(port_list) = c["ports"].as_array() {
                    for p in port_list {
                        let container_port = p["containerPort"].as_i64().unwrap_or(0);
                        let protocol = p["protocol"].as_str().unwrap_or("TCP");
                        ports.push(format!("{}/{}", container_port, protocol));
                    }
                }
                break;
            }
        }
    }
    ports
}

/// Extract probe command string.
fn extract_probe(
    item: &serde_json::Value,
    container_name: &str,
    probe_key: &str,
) -> Option<String> {
    if let Some(containers) = item["spec"]["containers"].as_array() {
        for c in containers {
            if c["name"].as_str() == Some(container_name) {
                if let Some(probe) = c.get(probe_key) {
                    return Some(serde_json::to_string(probe).unwrap_or_else(|_| "{}".to_string()));
                }
                break;
            }
        }
    }
    None
}

/// Extract resource limits/requests as key-value pairs.
fn extract_resources(
    item: &serde_json::Value,
    container_name: &str,
    resource_key: &str,
) -> Option<HashMap<String, String>> {
    if let Some(containers) = item["spec"]["containers"].as_array() {
        for c in containers {
            if c["name"].as_str() == Some(container_name) {
                if let Some(resources) = c["resources"].get(resource_key) {
                    if let Some(obj) = resources.as_object() {
                        let mut map = HashMap::new();
                        for (k, v) in obj {
                            map.insert(k.clone(), v.to_string());
                        }
                        return Some(map);
                    }
                }
                break;
            }
        }
    }
    None
}

/// Extract annotations from metadata.
fn extract_annotations(item: &serde_json::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(anns) = item["metadata"]["annotations"].as_object() {
        for (k, v) in anns {
            if let Some(val) = v.as_str() {
                map.insert(k.clone(), val.to_string());
            }
        }
    }
    map
}

// ─── Deployments ───────────────────────────────────────────────────────────

/// List deployments in the given namespace (or all namespaces).
pub async fn list_deployments(namespace: Option<&str>) -> Result<Vec<K8sDeployment>, String> {
    let mut args = vec!["get", "deployments", "-o", "json"];
    if let Some(ns) = namespace {
        args.push("-n");
        args.push(ns);
    }

    let json = execute_kubectl(&args).await?;
    let items = parse_items(&json);

    let deployments: Vec<K8sDeployment> = items
        .iter()
        .map(|item| {
            let status = &item["status"];
            let ready_replicas = status["readyReplicas"].as_i64().unwrap_or(0);
            let replicas = item["spec"]["replicas"].as_i64().unwrap_or(0);
            let available = status["availableReplicas"].as_i64().unwrap_or(0);
            let updated = status["updatedReplicas"].as_i64().unwrap_or(0);

            let strategy = item["spec"]["strategy"]["type"]
                .as_str()
                .unwrap_or("RollingUpdate")
                .to_string();

            let selector = extract_selector(item);

            K8sDeployment {
                name: meta_name(item),
                namespace: meta_namespace(item),
                ready: format!("{}/{}", ready_replicas, replicas),
                up_to_date: updated as i32,
                available: available as i32,
                age: calc_age(item),
                strategy,
                selector,
            }
        })
        .collect();

    Ok(deployments)
}

/// Extract selector match labels.
fn extract_selector(item: &serde_json::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(sel) = item["spec"]["selector"]["matchLabels"].as_object() {
        for (k, v) in sel {
            if let Some(val) = v.as_str() {
                map.insert(k.clone(), val.to_string());
            }
        }
    }
    map
}

// ─── Services ──────────────────────────────────────────────────────────────

/// List services in the given namespace (or all namespaces).
pub async fn list_services(namespace: Option<&str>) -> Result<Vec<K8sService>, String> {
    let mut args = vec!["get", "services", "-o", "json"];
    if let Some(ns) = namespace {
        args.push("-n");
        args.push(ns);
    }

    let json = execute_kubectl(&args).await?;
    let items = parse_items(&json);

    let services: Vec<K8sService> = items
        .iter()
        .map(|item| {
            let spec = &item["spec"];
            let cluster_ip = spec["clusterIP"].as_str().unwrap_or("None").to_string();
            let external_ip = spec["externalIPs"]
                .as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .or_else(|| {
                    // Try loadBalancer.ingress[0].ip or hostname
                    item["status"]["loadBalancer"]["ingress"]
                        .as_array()
                        .and_then(|arr| arr.first())
                        .and_then(|ing| ing["ip"].as_str().or_else(|| ing["hostname"].as_str()))
                })
                .map(|s| s.to_string());

            let ports: Vec<String> = spec["ports"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .map(|p| {
                            let port = p["port"].as_i64().unwrap_or(0);
                            let proto = p["protocol"].as_str().unwrap_or("TCP");
                            format!("{}/{}", port, proto)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let selector = extract_service_selector(item);

            K8sService {
                name: meta_name(item),
                namespace: meta_namespace(item),
                service_type: spec["type"].as_str().unwrap_or("ClusterIP").to_string(),
                cluster_ip,
                external_ip,
                ports,
                age: calc_age(item),
                selector,
            }
        })
        .collect();

    Ok(services)
}

fn extract_service_selector(item: &serde_json::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(sel) = item["spec"]["selector"].as_object() {
        for (k, v) in sel {
            if let Some(val) = v.as_str() {
                map.insert(k.clone(), val.to_string());
            }
        }
    }
    map
}

// ─── ConfigMaps ────────────────────────────────────────────────────────────

/// List ConfigMaps in the given namespace (or all namespaces).
pub async fn list_configmaps(namespace: Option<&str>) -> Result<Vec<K8sConfigMap>, String> {
    let mut args = vec!["get", "configmaps", "-o", "json"];
    if let Some(ns) = namespace {
        args.push("-n");
        args.push(ns);
    }

    let json = execute_kubectl(&args).await?;
    let items = parse_items(&json);

    let configmaps: Vec<K8sConfigMap> = items
        .iter()
        .map(|item| {
            let data_keys: Vec<String> = item["data"]
                .as_object()
                .map(|obj| obj.keys().cloned().collect())
                .unwrap_or_default();

            K8sConfigMap {
                name: meta_name(item),
                namespace: meta_namespace(item),
                keys_count: data_keys.len(),
                age: calc_age(item),
                data_keys,
            }
        })
        .collect();

    Ok(configmaps)
}

// ─── Secrets (Metadata Only) ───────────────────────────────────────────────

/// List secrets metadata in the given namespace (or all namespaces).
/// Secret **values** are NEVER returned — only key names and metadata.
pub async fn list_secrets(namespace: Option<&str>) -> Result<Vec<K8sSecretMeta>, String> {
    let mut args = vec!["get", "secrets", "-o", "json"];
    if let Some(ns) = namespace {
        args.push("-n");
        args.push(ns);
    }

    let json = execute_kubectl(&args).await?;
    let items = parse_items(&json);

    let secrets: Vec<K8sSecretMeta> = items
        .iter()
        .map(|item| {
            let data_keys: Vec<String> = item["data"]
                .as_object()
                .map(|obj| obj.keys().cloned().collect())
                .unwrap_or_default();

            let secret_type = item["type"].as_str().unwrap_or("Opaque").to_string();

            K8sSecretMeta {
                name: meta_name(item),
                namespace: meta_namespace(item),
                secret_type,
                keys_count: data_keys.len(),
                age: calc_age(item),
                data_keys,
            }
        })
        .collect();

    Ok(secrets)
}

// ─── Resource YAML Output ─────────────────────────────────────────────────

/// Get a single resource as YAML string (for viewing).
/// If the resource is a Secret, the YAML is redacted before returning.
pub async fn get_resource_yaml(namespace: &str, kind: &str, name: &str) -> Result<String, String> {
    let json = execute_kubectl(&["get", kind, name, "-n", namespace, "-o", "json"]).await?;

    let yaml_str =
        serde_yaml::to_string(&json).map_err(|e| format!("Failed to serialize to YAML: {}", e))?;

    // Redact secrets before returning
    if kind.to_lowercase() == "secret" {
        Ok(redact_secret_yaml(&yaml_str))
    } else {
        Ok(yaml_str)
    }
}

// ─── Pod Events ────────────────────────────────────────────────────────────

/// Extract events from a pod (parsed from `kubectl describe pod` output).
pub async fn get_pod_events(namespace: &str, pod_name: &str) -> Result<Vec<K8sEvent>, String> {
    let output = execute_kubectl(&["describe", "pod", pod_name, "-n", namespace]).await?;

    // Describe returns a single JSON object with the full spec+status
    // Events are not in the JSON output from `kubectl describe pod -o json`
    // We use the structured JSON and try to find conditions
    let mut events = Vec::new();

    // Extract conditions as pseudo-events
    if let Some(conditions) = output["status"]["conditions"].as_array() {
        for cond in conditions {
            let c_type = cond["type"].as_str().unwrap_or("Unknown");
            let status = cond["status"].as_str().unwrap_or("Unknown");
            let reason = cond["reason"].as_str().unwrap_or("");
            let message = cond["message"].as_str().unwrap_or("");
            let last_time = cond["lastTransitionTime"].as_str().unwrap_or("unknown");

            let event_type = if status == "True" {
                "Normal"
            } else {
                "Warning"
            };

            events.push(K8sEvent {
                timestamp: last_time.to_string(),
                event_type: event_type.to_string(),
                reason: format!("{}: {}", c_type, reason),
                message: message.to_string(),
            });
        }
    }

    Ok(events)
}

// ─── Secret Redaction ──────────────────────────────────────────────────────

/// Parse YAML, replace all `.data.*` and `.stringData.*` values with `[REDACTED]`,
/// then re-serialize.
pub fn redact_secret_yaml(yaml: &str) -> String {
    let mut value: serde_yaml::Value = match serde_yaml::from_str(yaml) {
        Ok(v) => v,
        Err(_) => return yaml.to_string(), // Return as-is if we can't parse
    };

    redact_yaml_value(&mut value);
    serde_yaml::to_string(&value).unwrap_or_else(|_| yaml.to_string())
}

fn redact_yaml_value(value: &mut serde_yaml::Value) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            // Only redact data/stringData for Secrets, not ConfigMaps
            let kind = map
                .get(serde_yaml::Value::String("kind".into()))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let is_secret = kind.eq_ignore_ascii_case("Secret");

            // Check if this mapping has "data" or "stringData" keys
            let has_data = map.contains_key(serde_yaml::Value::String("data".into()))
                || map.contains_key(serde_yaml::Value::String("stringData".into()));

            for (k, v) in map.iter_mut() {
                if is_secret && has_data {
                    if let Some(key_str) = k.as_str() {
                        if key_str == "data" || key_str == "stringData" {
                            if let serde_yaml::Value::Mapping(data_map) = v {
                                for (_dk, dv) in data_map.iter_mut() {
                                    *dv = serde_yaml::Value::String("[REDACTED]".to_string());
                                }
                            }
                            continue;
                        }
                    }
                }
                redact_yaml_value(v);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for v in seq.iter_mut() {
                redact_yaml_value(v);
            }
        }
        _ => {}
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Blocklist Tests ───

    #[test]
    fn test_kubectl_blocklist_rejects_apply() {
        let result = validate_kubectl_args(&["apply".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Blocked"));
    }

    #[test]
    fn test_kubectl_blocklist_rejects_delete() {
        let result = validate_kubectl_args(&["delete".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Blocked"));
    }

    #[test]
    fn test_kubectl_blocklist_rejects_create() {
        let result = validate_kubectl_args(&["create".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_edit() {
        let result = validate_kubectl_args(&["edit".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_patch() {
        let result = validate_kubectl_args(&["patch".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_replace() {
        let result = validate_kubectl_args(&["replace".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_rollout() {
        let result = validate_kubectl_args(&["rollout".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_scale() {
        let result = validate_kubectl_args(&["scale".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_expose() {
        let result = validate_kubectl_args(&["expose".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_port_forward() {
        let result = validate_kubectl_args(&["port-forward".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_exec() {
        let result = validate_kubectl_args(&["exec".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_cp() {
        let result = validate_kubectl_args(&["cp".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_drain() {
        let result = validate_kubectl_args(&["drain".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_cordon() {
        let result = validate_kubectl_args(&["cordon".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_uncordon() {
        let result = validate_kubectl_args(&["uncordon".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_taint() {
        let result = validate_kubectl_args(&["taint".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_attach() {
        let result = validate_kubectl_args(&["attach".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_proxy() {
        let result = validate_kubectl_args(&["proxy".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_debug() {
        let result = validate_kubectl_args(&["debug".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_blocklist_rejects_auth() {
        let result = validate_kubectl_args(&["auth".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_kubectl_allows_get() {
        let result = validate_kubectl_args(&["get".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kubectl_allows_describe() {
        let result = validate_kubectl_args(&["describe".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kubectl_allows_config() {
        let result = validate_kubectl_args(&["config".to_string(), "use-context".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kubectl_allows_api_resources() {
        let result = validate_kubectl_args(&["api-resources".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kubectl_allows_top() {
        let result = validate_kubectl_args(&["top".to_string(), "pods".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kubectl_allows_version() {
        let result = validate_kubectl_args(&["version".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kubectl_allows_explain() {
        let result = validate_kubectl_args(&["explain".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kubectl_blocklist_multiple_args() {
        let result =
            validate_kubectl_args(&["get".to_string(), "pods".to_string(), "delete".to_string()]);
        assert!(result.is_err());
    }

    // ─── Kubeconfig Parsing Tests ───

    #[test]
    fn test_parse_kubeconfig_missing_file() {
        // Temporarily set KUBECONFIG to a non-existent file
        std::env::set_var("KUBECONFIG", "/tmp/nonexistent_kubeconfig_itzambox_test");
        let result = parse_kubeconfig();
        assert!(result.is_err());
        std::env::remove_var("KUBECONFIG");
    }

    #[test]
    fn test_parse_kubeconfig_valid() {
        let dir = std::env::temp_dir().join("itzambox_kube_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("config");
        let yaml = r#"
apiVersion: v1
kind: Config
current-context: prod
contexts:
- context:
    cluster: prod-cluster
    user: admin
  name: prod
- context:
    cluster: staging-cluster
    user: developer
  name: staging
clusters:
- cluster:
    server: https://api.prod.example.com
  name: prod-cluster
- cluster:
    server: https://api.staging.example.com
  name: staging-cluster
users:
- name: admin
  user:
    token: fake-token
- name: developer
  user:
    token: fake-token-dev
"#;
        std::fs::write(&path, yaml).expect("write test kubeconfig");
        std::env::set_var("KUBECONFIG", path.to_str().unwrap());

        let result = parse_kubeconfig();
        assert!(
            result.is_ok(),
            "parse_kubeconfig failed: {:?}",
            result.err()
        );
        let contexts = result.unwrap();
        assert_eq!(contexts.len(), 2);
        assert_eq!(contexts[0].name, "prod");
        assert_eq!(contexts[0].cluster, "prod-cluster");
        assert!(contexts[0].is_active);
        assert!(!contexts[1].is_active);
        assert_eq!(contexts[1].name, "staging");

        std::env::remove_var("KUBECONFIG");
        std::fs::remove_file(path).ok();
        std::fs::remove_dir(dir).ok();
    }

    #[test]
    fn test_parse_kubeconfig_empty_file() {
        let dir = std::env::temp_dir().join("itzambox_kube_empty_test");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("config_empty");
        std::fs::write(&path, "").expect("write empty kubeconfig");
        std::env::set_var("KUBECONFIG", path.to_str().unwrap());

        let result = parse_kubeconfig();
        assert!(result.is_err());

        std::env::remove_var("KUBECONFIG");
        std::fs::remove_file(path).ok();
        std::fs::remove_dir(dir).ok();
    }

    // ─── Secret Redaction Tests ───

    #[test]
    fn test_redact_secret_yaml() {
        let yaml = r#"
apiVersion: v1
kind: Secret
metadata:
  name: my-secret
type: Opaque
data:
  password: c3VwZXItc2VjcmV0
  username: YWRtaW4=
stringData:
  api_key: sk-live-abc123
"#;
        let redacted = redact_secret_yaml(yaml);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("c3VwZXItc2VjcmV0"));
        assert!(!redacted.contains("sk-live-abc123"));
        assert!(!redacted.contains("YWRtaW4="));
        // Verify the structure is preserved
        assert!(redacted.contains("password"));
        assert!(redacted.contains("username"));
        assert!(redacted.contains("api_key"));
    }

    #[test]
    fn test_redact_secret_yaml_no_data_field() {
        let yaml = r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: my-config
data:
  app.config: "some-value"
"#;
        let redacted = redact_secret_yaml(yaml);
        // ConfigMap should not have values redacted
        assert!(redacted.contains("some-value"));
    }

    #[test]
    fn test_redact_secret_yaml_invalid() {
        let yaml = "not: valid: yaml: [[[";
        let redacted = redact_secret_yaml(yaml);
        assert_eq!(redacted, yaml); // Return as-is on parse failure
    }

    // ─── Pod JSON Parsing Tests ───

    #[test]
    fn test_parse_items_empty() {
        let json: serde_json::Value = serde_json::from_str(r#"{"items": []}"#).unwrap();
        let items = parse_items(&json);
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_items_missing_key() {
        let json: serde_json::Value = serde_json::from_str(r#"{}"#).unwrap();
        let items = parse_items(&json);
        assert!(items.is_empty());
    }

    #[test]
    fn test_calc_age_with_timestamp() {
        let item: serde_json::Value =
            serde_json::from_str(r#"{"metadata": {"creationTimestamp": "2026-06-14T12:00:00Z"}}"#)
                .unwrap();
        // The age will be relative to "now" — just verify it's a formatted string
        let age = calc_age(&item);
        assert!(!age.is_empty());
        assert!(
            age.ends_with('s') || age.ends_with('m') || age.ends_with('h') || age.ends_with('d')
        );
    }

    #[test]
    fn test_calc_age_missing_timestamp() {
        let item: serde_json::Value = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(calc_age(&item), "unknown");
    }

    #[test]
    fn test_extract_annotations() {
        let item: serde_json::Value = serde_json::from_str(
            r#"{
                "metadata": {
                    "annotations": {
                        "kubectl.kubernetes.io/last-applied-configuration": "fake"
                    }
                }
            }"#,
        )
        .unwrap();
        let anns = extract_annotations(&item);
        assert_eq!(anns.len(), 1);
        assert_eq!(
            anns.get("kubectl.kubernetes.io/last-applied-configuration")
                .unwrap(),
            "fake"
        );
    }

    #[test]
    fn test_derive_pod_status_waiting() {
        let item: serde_json::Value = serde_json::from_str(
            r#"{
                "status": {
                    "phase": "Running",
                    "containerStatuses": [{
                        "name": "main",
                        "state": {
                            "waiting": {
                                "reason": "CrashLoopBackOff"
                            }
                        },
                        "restartCount": 5,
                        "ready": false,
                        "image": "nginx:latest"
                    }]
                }
            }"#,
        )
        .unwrap();
        assert_eq!(derive_pod_status(&item), "CrashLoopBackOff");
    }

    #[test]
    fn test_derive_pod_status_running() {
        let item: serde_json::Value = serde_json::from_str(
            r#"{
                "status": {
                    "phase": "Running",
                    "containerStatuses": [{
                        "name": "main",
                        "state": {"running": {}},
                        "restartCount": 0,
                        "ready": true,
                        "image": "nginx:latest"
                    }]
                }
            }"#,
        )
        .unwrap();
        assert_eq!(derive_pod_status(&item), "Running");
    }
}
