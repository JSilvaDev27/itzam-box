// ItzamBox — Kubernetes Tauri Commands
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// All commands are read-only.  Inputs are sanitized at the boundary before
// being passed to the engine.  Secret values are redacted before returning.

use crate::engine::kubernetes;
use crate::engine::types::*;
use crate::utils::sanitizer;

// ─── Detection & Status ────────────────────────────────────────────────────

/// Detect kubectl binary availability, version, and kubeconfig parse status.
#[tauri::command]
pub async fn detect_kubectl() -> Result<KubectlStatus, String> {
    Ok(kubernetes::detect_kubectl().await)
}

/// Parse kubeconfig YAML and return all contexts with active flag.
#[tauri::command]
pub async fn list_k8s_contexts() -> Result<Vec<K8sContext>, String> {
    kubernetes::list_contexts().await
}

/// Switch the active kubectl context.
#[tauri::command]
pub async fn set_k8s_context(context: String) -> Result<(), String> {
    let ctx = sanitizer::sanitize_context_name(&context)?;
    kubernetes::set_context(&ctx).await
}

// ─── Namespaces ────────────────────────────────────────────────────────────

/// List all namespaces in the active cluster.
#[tauri::command]
pub async fn list_namespaces() -> Result<Vec<String>, String> {
    kubernetes::list_namespaces().await
}

// ─── Pods ──────────────────────────────────────────────────────────────────

/// List pods in the given namespace (or all namespaces if null).
#[tauri::command]
pub async fn list_pods(namespace: Option<String>) -> Result<Vec<K8sPod>, String> {
    match &namespace {
        Some(ns) => {
            let sanitized = sanitizer::sanitize_k8s_namespace(ns)?;
            kubernetes::list_pods(Some(&sanitized)).await
        }
        None => kubernetes::list_pods(None).await,
    }
}

// ─── Deployments ───────────────────────────────────────────────────────────

/// List deployments in the given namespace (or all namespaces if null).
#[tauri::command]
pub async fn list_deployments(namespace: Option<String>) -> Result<Vec<K8sDeployment>, String> {
    match &namespace {
        Some(ns) => {
            let sanitized = sanitizer::sanitize_k8s_namespace(ns)?;
            kubernetes::list_deployments(Some(&sanitized)).await
        }
        None => kubernetes::list_deployments(None).await,
    }
}

// ─── Services ──────────────────────────────────────────────────────────────

/// List services in the given namespace (or all namespaces if null).
#[tauri::command]
pub async fn list_services(namespace: Option<String>) -> Result<Vec<K8sService>, String> {
    match &namespace {
        Some(ns) => {
            let sanitized = sanitizer::sanitize_k8s_namespace(ns)?;
            kubernetes::list_services(Some(&sanitized)).await
        }
        None => kubernetes::list_services(None).await,
    }
}

// ─── ConfigMaps ────────────────────────────────────────────────────────────

/// List ConfigMaps in the given namespace (or all namespaces if null).
#[tauri::command]
pub async fn list_configmaps(namespace: Option<String>) -> Result<Vec<K8sConfigMap>, String> {
    match &namespace {
        Some(ns) => {
            let sanitized = sanitizer::sanitize_k8s_namespace(ns)?;
            kubernetes::list_configmaps(Some(&sanitized)).await
        }
        None => kubernetes::list_configmaps(None).await,
    }
}

// ─── Secrets (Metadata Only) ───────────────────────────────────────────────

/// List secrets metadata in the given namespace (or all namespaces if null).
/// Secret VALUES are never returned — only key names.
#[tauri::command]
pub async fn list_secrets(namespace: Option<String>) -> Result<Vec<K8sSecretMeta>, String> {
    match &namespace {
        Some(ns) => {
            let sanitized = sanitizer::sanitize_k8s_namespace(ns)?;
            kubernetes::list_secrets(Some(&sanitized)).await
        }
        None => kubernetes::list_secrets(None).await,
    }
}

// ─── Resource YAML ─────────────────────────────────────────────────────────

/// Get a single resource's YAML output.  Secrets are REDACTED.
#[tauri::command]
pub async fn get_resource_yaml(
    namespace: String,
    kind: String,
    name: String,
) -> Result<String, String> {
    let ns = sanitizer::sanitize_k8s_namespace(&namespace)?;
    let k = sanitizer::sanitize_k8s_resource_name(&kind)?;
    let n = sanitizer::sanitize_k8s_resource_name(&name)?;
    kubernetes::get_resource_yaml(&ns, &k, &n).await
}

// ─── Pod Events ────────────────────────────────────────────────────────────

/// Get events/conditions for a specific pod.
#[tauri::command]
pub async fn get_pod_events(namespace: String, name: String) -> Result<Vec<K8sEvent>, String> {
    let ns = sanitizer::sanitize_k8s_namespace(&namespace)?;
    let pod_name = sanitizer::sanitize_k8s_resource_name(&name)?;
    kubernetes::get_pod_events(&ns, &pod_name).await
}
