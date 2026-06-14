// ItzamBox — Vulnerability Scanner Commands
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// Integrates Trivy / Grype to scan container images for known CVEs.
// Emits progress via Tauri events, stores results in SQLite.

use crate::engine::types::{Vulnerability, VulnerabilityReport};
use crate::AppState;
use serde::Deserialize;
use std::process::Command;
use tauri::Emitter;
use tauri::{AppHandle, State};

// ─── Scanner Detection ────────────────────────────────────────────────────

/// Detect which vulnerability scanner is installed on the host.
/// Returns `"trivy"`, `"grype"`, or `None` if neither is found.
#[tauri::command]
pub async fn detect_scanner() -> Result<Option<String>, String> {
    if is_tool_installed("trivy") {
        let version = get_tool_version("trivy", "version").unwrap_or_else(|| "unknown".into());
        return Ok(Some(format!("trivy v{}", version)));
    }
    if is_tool_installed("grype") {
        let version = get_tool_version("grype", "version").unwrap_or_else(|| "unknown".into());
        return Ok(Some(format!("grype v{}", version)));
    }
    Ok(None)
}

fn is_tool_installed(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_tool_version(tool: &str, arg: &str) -> Option<String> {
    let out = Command::new(tool).arg(arg).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    // Extract first "x.y.z" or similar version-like token
    s.lines().next().map(|l| l.trim().to_string())
}

// ─── Trivy JSON Parsing ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct TrivyOutput {
    #[serde(rename = "Results")]
    results: Vec<TrivyResult>,
}

#[derive(Deserialize)]
struct TrivyResult {
    #[serde(rename = "Vulnerabilities")]
    vulnerabilities: Option<Vec<TrivyVuln>>,
}

#[derive(Deserialize)]
struct TrivyVuln {
    #[serde(rename = "VulnerabilityID")]
    vulnerability_id: String,
    #[serde(rename = "PkgName")]
    pkg_name: String,
    #[serde(rename = "InstalledVersion")]
    installed_version: String,
    #[serde(rename = "FixedVersion")]
    fixed_version: Option<String>,
    #[serde(rename = "Severity")]
    severity: String,
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Description")]
    description: String,
}

// ─── Grype JSON Parsing ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct GrypeOutput {
    matches: Vec<GrypeMatch>,
}

#[derive(Deserialize)]
struct GrypeMatch {
    vulnerability: GrypeVuln,
    artifact: GrypeArtifact,
}

#[derive(Deserialize)]
struct GrypeVuln {
    id: String,
    severity: String,
    description: Option<String>,
}

#[derive(Deserialize)]
struct GrypeArtifact {
    name: String,
    version: String,
}

// ─── Progress Event ───────────────────────────────────────────────────────

#[derive(Clone, serde::Serialize)]
struct ScanProgress {
    step: String,
    message: String,
}

// ─── Scan Image Command ───────────────────────────────────────────────────

/// Scan a container image for vulnerabilities using the available scanner.
/// Emits `scan-progress` events for real-time UI feedback.
#[tauri::command]
pub async fn scan_image(
    app: AppHandle,
    state: State<'_, AppState>,
    image_name: String,
) -> Result<VulnerabilityReport, String> {
    // 1. Detect scanner
    emit_progress(&app, "detect", "Detecting available scanner…").ok();
    let scanner = if is_tool_installed("trivy") {
        "trivy"
    } else if is_tool_installed("grype") {
        "grype"
    } else {
        return Err("No vulnerability scanner found. Install Trivy (recommended) or Grype.".into());
    };

    emit_progress(
        &app,
        "scan",
        &format!("Scanning {} with {}…", image_name, scanner),
    )
    .ok();

    // 2. Run the scan
    let output = if scanner == "trivy" {
        run_trivy_scan(&image_name)?
    } else {
        run_grype_scan(&image_name)?
    };

    emit_progress(&app, "parse", "Parsing vulnerability results…").ok();

    // 3. Parse JSON output into vulnerabilities
    let all_vulns = if scanner == "trivy" {
        parse_trivy_output(&output)?
    } else {
        parse_grype_output(&output)?
    };

    // 4. Categorize by severity (case-insensitive)
    let critical: Vec<Vulnerability> = all_vulns
        .iter()
        .filter(|v| v.severity.to_lowercase() == "critical")
        .cloned()
        .collect();
    let high: Vec<Vulnerability> = all_vulns
        .iter()
        .filter(|v| v.severity.to_lowercase() == "high")
        .cloned()
        .collect();
    let medium: Vec<Vulnerability> = all_vulns
        .iter()
        .filter(|v| v.severity.to_lowercase() == "medium")
        .cloned()
        .collect();
    let low: Vec<Vulnerability> = all_vulns
        .iter()
        .filter(|v| v.severity.to_lowercase() == "low")
        .cloned()
        .collect();

    let total = all_vulns.len();

    let report = VulnerabilityReport {
        image_name: image_name.clone(),
        scanned_at: chrono::Utc::now().timestamp(),
        total,
        critical,
        high,
        medium,
        low,
    };

    // 5. Save to database
    emit_progress(&app, "save", "Saving scan results…").ok();
    if let Ok(db) = state.db.lock() {
        let report_json = serde_json::to_string(&report).map_err(|e| e.to_string())?;
        let critical_count = report.critical.len() as i64;
        let high_count = report.high.len() as i64;
        let medium_count = report.medium.len() as i64;
        let low_count = report.low.len() as i64;

        let result = db.execute(
            "INSERT INTO scan_history (image_name, critical_count, high_count, medium_count, low_count, report_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![image_name, critical_count, high_count, medium_count, low_count, report_json],
        );
        if let Err(e) = result {
            log::error!("Failed to save scan result: {}", e);
        }
    } else {
        log::error!("Failed to lock database for scan result");
    }

    emit_progress(
        &app,
        "done",
        &format!("Scan complete — {} vulnerabilities found.", total),
    )
    .ok();

    Ok(report)
}

// ─── Scanner Execution ────────────────────────────────────────────────────

fn run_trivy_scan(image_name: &str) -> Result<String, String> {
    let output = Command::new("trivy")
        .args([
            "image",
            "--format",
            "json",
            "--severity",
            "CRITICAL,HIGH,MEDIUM,LOW",
            image_name,
        ])
        .output()
        .map_err(|e| format!("Failed to execute Trivy: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Trivy scan failed: {}", stderr.trim()));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 from Trivy: {}", e))
}

fn run_grype_scan(image_name: &str) -> Result<String, String> {
    let output = Command::new("grype")
        .args([image_name, "-o", "json"])
        .output()
        .map_err(|e| format!("Failed to execute Grype: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Grype scan failed: {}", stderr.trim()));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 from Grype: {}", e))
}

// ─── JSON Parsers ─────────────────────────────────────────────────────────

fn parse_trivy_output(json_str: &str) -> Result<Vec<Vulnerability>, String> {
    let output: TrivyOutput = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse Trivy output: {}", e))?;

    let mut vulns = Vec::new();
    for result in output.results {
        if let Some(vulnerabilities) = result.vulnerabilities {
            for v in vulnerabilities {
                vulns.push(Vulnerability {
                    id: v.vulnerability_id,
                    package: v.pkg_name,
                    installed_version: v.installed_version,
                    fixed_version: v.fixed_version.filter(|s| !s.is_empty()),
                    severity: v.severity.to_lowercase(),
                    title: v.title,
                    description: v.description,
                });
            }
        }
    }
    Ok(vulns)
}

fn parse_grype_output(json_str: &str) -> Result<Vec<Vulnerability>, String> {
    let output: GrypeOutput = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse Grype output: {}", e))?;

    let vulns: Vec<Vulnerability> = output
        .matches
        .into_iter()
        .map(|m| {
            // Map severity: Grype uses "Critical", "High", etc.
            let severity = normalize_grype_severity(&m.vulnerability.severity);
            let vuln_id = m.vulnerability.id;
            Vulnerability {
                id: vuln_id.clone(),
                package: m.artifact.name,
                installed_version: m.artifact.version,
                fixed_version: None, // Grype doesn't always provide this in the main match
                severity,
                title: vuln_id, // Grype uses ID as title fallback
                description: m.vulnerability.description.unwrap_or_default(),
            }
        })
        .collect();

    Ok(vulns)
}

fn normalize_grype_severity(s: &str) -> String {
    let lower = s.to_lowercase();
    match lower.as_str() {
        "critical" => "critical".into(),
        "high" => "high".into(),
        "medium" => "medium".into(),
        "low" => "low".into(),
        "negligible" => "low".into(),
        "unknown" => "low".into(),
        _ => lower,
    }
}

// ─── Event Emission Helper ────────────────────────────────────────────────

fn emit_progress(app: &AppHandle, step: &str, message: &str) -> Result<(), String> {
    app.emit(
        "scan-progress",
        ScanProgress {
            step: step.to_string(),
            message: message.to_string(),
        },
    )
    .map_err(|e| format!("Failed to emit progress: {}", e))
}

// ─── Scan History ─────────────────────────────────────────────────────────

/// Retrieve the last 10 scan history entries for a given image.
#[tauri::command]
pub async fn get_scan_history(
    state: State<'_, AppState>,
    image_name: String,
) -> Result<Vec<VulnerabilityReport>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare(
            "SELECT report_json FROM scan_history WHERE image_name = ?1 ORDER BY scanned_at DESC LIMIT 10",
        )
        .map_err(|e| format!("DB prepare failed: {}", e))?;

    let rows = stmt
        .query_map(rusqlite::params![image_name], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })
        .map_err(|e| format!("DB query failed: {}", e))?;

    let mut reports = Vec::new();
    for json in rows.flatten() {
        if let Ok(report) = serde_json::from_str::<VulnerabilityReport>(&json) {
            reports.push(report);
        }
    }

    Ok(reports)
}
