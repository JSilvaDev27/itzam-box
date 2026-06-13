// ItzamBox — Docker Multi-Distro Installer Commands
// Copyright (C) 2026 SodigTech — GPL-3.0
//
// Auto-detects the Linux distribution, checks Docker status,
// and installs Docker Engine using the native package manager.
// Progress is emitted via Tauri events for real-time UI updates.

use serde::{Deserialize, Serialize};
use std::process::Command;
use std::path::Path;
use tauri::Emitter;
use tauri::AppHandle;

// ─── Data Types ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LinuxDistro {
    pub id: String,               // "ubuntu", "debian", "fedora", "arch", "opensuse-tumbleweed"
    pub name: String,             // "Ubuntu 24.04 LTS"
    pub version: String,          // "24.04"
    pub id_like: Option<String>,  // "debian" (for Ubuntu, Mint, etc.)
    pub package_manager: String,  // "apt", "dnf", "yum", "pacman", "zypper"
    pub supported: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DockerInstallStatus {
    pub docker_installed: bool,
    pub docker_version: Option<String>,
    pub compose_available: bool,
    pub compose_version: Option<String>,
    pub service_running: bool,
    pub user_in_docker_group: bool,
    pub socket_exists: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct InstallProgress {
    step: usize,
    total_steps: usize,
    message: String,
    status: String, // "in_progress", "completed", "error"
}

// ─── Distro Detection ──────────────────────────────────────────────────────

/// Determine the package manager from distro identifiers.
fn resolve_package_manager(id: &str, id_like: Option<&str>) -> &'static str {
    match id {
        "ubuntu" | "debian" | "linuxmint" | "pop" | "elementary" | "kali" => "apt",
        "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => "dnf",
        "arch" | "manjaro" | "endeavouros" | "arcolinux" | "garuda" => "pacman",
        "opensuse-tumbleweed" | "opensuse-leap" | "sles" | "opensuse" => "zypper",
        _ => {
            match id_like {
                Some("debian") => "apt",
                Some("fedora") => "dnf",
                Some("rhel") => "dnf",
                Some("arch") => "pacman",
                Some("suse") => "zypper",
                _ => "unknown",
            }
        }
    }
}

/// Read a single field value from `/etc/os-release`.
///
/// The file uses `KEY="VALUE"` or `KEY=VALUE` syntax.
fn os_release_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if let Some(stripped) = line.strip_prefix(&format!("{}=", field)) {
            let val = stripped.trim_matches('"').trim_matches('\'');
            return Some(val.to_string());
        }
    }
    None
}

/// Detect the Linux distribution by parsing `/etc/os-release`.
fn detect_distro_from_os_release() -> Option<(String, String, String, Option<String>)> {
    let path = Path::new("/etc/os-release");
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;

    let id = os_release_field(&content, "ID")?;
    let name = os_release_field(&content, "NAME")?;
    let version = os_release_field(&content, "VERSION_ID").unwrap_or_default();
    let id_like = os_release_field(&content, "ID_LIKE");

    Some((id, name, version, id_like))
}

/// Fallback: try `lsb_release -a` to get distro info.
fn detect_distro_from_lsb() -> Option<(String, String, String, Option<String>)> {
    let output = Command::new("lsb_release").arg("-a").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut id = String::new();
    let mut desc = String::new();
    let mut release = String::new();

    for line in stdout.lines() {
        if let Some(val) = line.strip_prefix("Distributor ID:\t") {
            id = val.trim().to_lowercase();
        }
        if let Some(val) = line.strip_prefix("Description:\t") {
            desc = val.trim().to_string();
        }
        if let Some(val) = line.strip_prefix("Release:\t") {
            release = val.trim().to_string();
        }
    }
    if id.is_empty() {
        return None;
    }
    Some((id, desc, release, None))
}

/// Detect the current Linux distribution and return structured info.
fn do_detect_linux_distro() -> Result<LinuxDistro, String> {
    // Try /etc/os-release first, fallback to lsb_release
    let (id, name, version, id_like) = detect_distro_from_os_release()
        .or_else(detect_distro_from_lsb)
        .ok_or_else(|| {
            "Could not detect Linux distribution. Only Linux is supported.".to_string()
        })?;

    let id_clean = id.to_lowercase();
    let pm = resolve_package_manager(&id_clean, id_like.as_deref());
    let supported = pm != "unknown";

    Ok(LinuxDistro {
        id: id_clean,
        name: name.clone(),
        version: version.clone(),
        id_like: id_like.clone(),
        package_manager: pm.to_string(),
        supported,
    })
}

#[tauri::command]
pub async fn detect_linux_distro() -> Result<LinuxDistro, String> {
    do_detect_linux_distro()
}

// ─── Docker Status Check ───────────────────────────────────────────────────

fn run_cmd(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run `{}`: {}", cmd, e))?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(stderr)
    }
}

fn is_user_in_group(group: &str) -> bool {
    let output = Command::new("groups").output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.split_whitespace().any(|g| g == group)
        }
        _ => false,
    }
}

fn is_service_active(service: &str) -> bool {
    let output = Command::new("systemctl")
        .args(["is-active", service])
        .output();
    match output {
        Ok(out) => out.status.success(),
        _ => false,
    }
}

#[tauri::command]
pub async fn check_docker_installed() -> Result<DockerInstallStatus, String> {
    let docker_version = run_cmd("docker", &["--version"]).ok();
    let docker_installed = docker_version.is_some();

    let compose_version = run_cmd("docker", &["compose", "version"]).ok();
    let compose_available = compose_version.is_some();

    let service_running = is_service_active("docker");

    let user_in_docker_group = is_user_in_group("docker");

    let socket_exists = Path::new("/var/run/docker.sock").exists();

    Ok(DockerInstallStatus {
        docker_installed,
        docker_version,
        compose_available,
        compose_version,
        service_running,
        user_in_docker_group,
        socket_exists,
    })
}

// ─── Privilege Escalation ──────────────────────────────────────────────────

/// Find the available privilege escalation tool.
/// Returns `(command, prefix_args)`.
fn find_escalation() -> Result<(&'static str, Vec<&'static str>), String> {
    // Try pkexec first (GUI auth dialog)
    if Path::new("/usr/bin/pkexec").exists()
        || Path::new("/bin/pkexec").exists()
    {
        return Ok(("pkexec", vec![]));
    }
    // Fallback to sudo -n (non-interactive)
    if Path::new("/usr/bin/sudo").exists()
        || Path::new("/bin/sudo").exists()
    {
        return Ok(("sudo", vec!["-n"]));
    }
    Err("No privilege escalation tool found. Please install pkexec or sudo.".to_string())
}

/// Run a command with elevated privileges (pkexec or sudo).
/// Returns stdout on success.
#[allow(dead_code)]
fn run_elevated(cmd_name: &str, args: &[&str]) -> Result<String, String> {
    let (escalator, prefix) = find_escalation()?;

    let mut full_args: Vec<&str> = Vec::with_capacity(prefix.len() + 1 + args.len());
    full_args.extend_from_slice(&prefix);
    full_args.push(cmd_name);
    full_args.extend_from_slice(args);

    let output = Command::new(escalator)
        .args(&full_args)
        .output()
        .map_err(|e| {
            format!(
                "Failed to execute `{} {} {}`: {}",
                escalator,
                cmd_name,
                args.join(" "),
                e
            )
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(stdout)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!(
            "Command `{} {}` failed (exit: {}): {}",
            cmd_name,
            args.join(" "),
            output.status.code().unwrap_or(-1),
            stderr
        ))
    }
}

/// Run a command with elevated privileges, ignoring the output (only
/// cares about success).  Useful for commands that produce no meaningful
/// stdout.
fn run_elevated_status(cmd_name: &str, args: &[&str]) -> Result<(), String> {
    let (escalator, prefix) = find_escalation()?;

    let mut full_args: Vec<&str> = Vec::with_capacity(prefix.len() + 1 + args.len());
    full_args.extend_from_slice(&prefix);
    full_args.push(cmd_name);
    full_args.extend_from_slice(args);

    let output = Command::new(escalator)
        .args(&full_args)
        .output()
        .map_err(|e| {
            format!(
                "Failed to execute `{} {} {}`: {}",
                escalator,
                cmd_name,
                args.join(" "),
                e
            )
        })?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(format!(
            "Command `{} {}` failed (exit: {}): {}",
            cmd_name,
            args.join(" "),
            output.status.code().unwrap_or(-1),
            stderr
        ))
    }
}

// ─── Installation Progress Emitter ─────────────────────────────────────────

fn emit_progress(app: &AppHandle, step: usize, total: usize, msg: &str, status: &str) {
    let payload = InstallProgress {
        step,
        total_steps: total,
        message: msg.to_string(),
        status: status.to_string(),
    };
    app.emit("installer-progress", payload).ok();
}

// ─── Installation Flows ────────────────────────────────────────────────────

/// Install Docker on Debian-based distros (apt).
async fn install_apt(app: AppHandle, distro: &LinuxDistro) -> Result<(), String> {
    let total: usize = 11;
    let mut step: usize = 0;

    step += 1;
    emit_progress(&app, step, total, "Updating package lists...", "in_progress");
    run_elevated_status("apt-get", &["update"])?;

    step += 1;
    emit_progress(&app, step, total, "Installing prerequisites...", "in_progress");
    run_elevated_status("apt-get", &["install", "-y", "ca-certificates", "curl"])?;

    step += 1;
    emit_progress(&app, step, total, "Creating keyrings directory...", "in_progress");
    run_elevated_status("install", &["-m", "0755", "-d", "/etc/apt/keyrings"])?;

    step += 1;
    emit_progress(&app, step, total, "Downloading Docker GPG key...", "in_progress");
    // curl needs to run as root to write to /etc/apt/keyrings
    run_elevated_status(
        "curl",
        &[
            "-fsSL",
            "https://download.docker.com/linux/ubuntu/gpg",
            "-o",
            "/etc/apt/keyrings/docker.asc",
        ],
    )?;

    step += 1;
    emit_progress(&app, step, total, "Setting GPG key permissions...", "in_progress");
    run_elevated_status("chmod", &["a+r", "/etc/apt/keyrings/docker.asc"])?;

    step += 1;
    emit_progress(
        &app,
        step,
        total,
        "Adding Docker apt repository...",
        "in_progress",
    );
    // Determine the distro codename for the apt source
    let os_release_content =
        std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    let codename = os_release_field(&os_release_content, "VERSION_CODENAME")
        .or_else(|| os_release_field(&os_release_content, "UBUNTU_CODENAME"))
        .unwrap_or_else(|| {
            // Fallback: map version to codename for common distros
            match distro.id.as_str() {
                "ubuntu" => match distro.version.as_str() {
                    "24.04" => "noble".to_string(),
                    "22.04" => "jammy".to_string(),
                    "20.04" => "focal".to_string(),
                    _ => "noble".to_string(),
                },
                "debian" => match distro.version.as_str() {
                    "12" => "bookworm".to_string(),
                    "11" => "bullseye".to_string(),
                    _ => "bookworm".to_string(),
                },
                _ => "noble".to_string(),
            }
        });

    let arch = run_cmd("dpkg", &["--print-architecture"])
        .unwrap_or_else(|_| "amd64".to_string());

    // Build the repo string
    let repo_line = format!(
        "deb [arch={} signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu {} stable",
        arch, codename
    );

    // Write to docker.list using elevated shell
    run_elevated_status(
        "sh",
        &[
            "-c",
            &format!("echo '{}' > /etc/apt/sources.list.d/docker.list", repo_line),
        ],
    )?;

    step += 1;
    emit_progress(&app, step, total, "Updating package lists with Docker repo...", "in_progress");
    run_elevated_status("apt-get", &["update"])?;

    step += 1;
    emit_progress(&app, step, total, "Installing Docker Engine...", "in_progress");
    run_elevated_status(
        "apt-get",
        &[
            "install",
            "-y",
            "docker-ce",
            "docker-ce-cli",
            "containerd.io",
            "docker-buildx-plugin",
            "docker-compose-plugin",
        ],
    )?;

    step += 1;
    emit_progress(&app, step, total, "Starting Docker service...", "in_progress");
    run_elevated_status("systemctl", &["enable", "--now", "docker"])?;

    step += 1;
    emit_progress(&app, step, total, "Adding user to docker group...", "in_progress");
    // Get the current user from $SUDO_USER or $USER
    let user = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "root".to_string());
    run_elevated_status("usermod", &["-aG", "docker", &user])?;

    step += 1;
    emit_progress(
        &app,
        step,
        total,
        "Installation complete!",
        "completed",
    );

    Ok(())
}

/// Install Docker on Fedora/RHEL-based distros (dnf).
async fn install_dnf(app: AppHandle, _distro: &LinuxDistro) -> Result<(), String> {
    let total: usize = 5;
    let mut step: usize = 0;

    step += 1;
    emit_progress(&app, step, total, "Installing dnf-plugins-core...", "in_progress");
    run_elevated_status("dnf", &["-y", "install", "dnf-plugins-core"])?;

    step += 1;
    emit_progress(&app, step, total, "Adding Docker repository...", "in_progress");
    run_elevated_status(
        "dnf",
        &[
            "config-manager",
            "--add-repo",
            "https://download.docker.com/linux/fedora/docker-ce.repo",
        ],
    )?;

    step += 1;
    emit_progress(&app, step, total, "Installing Docker Engine...", "in_progress");
    run_elevated_status(
        "dnf",
        &[
            "install",
            "-y",
            "docker-ce",
            "docker-ce-cli",
            "containerd.io",
            "docker-buildx-plugin",
            "docker-compose-plugin",
        ],
    )?;

    step += 1;
    emit_progress(&app, step, total, "Starting Docker service...", "in_progress");
    run_elevated_status("systemctl", &["enable", "--now", "docker"])?;

    step += 1;
    emit_progress(&app, step, total, "Adding user to docker group...", "in_progress");
    let user = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "root".to_string());
    run_elevated_status("usermod", &["-aG", "docker", &user])?;

    emit_progress(&app, total, total, "Installation complete!", "completed");
    Ok(())
}

/// Install Docker on Arch-based distros (pacman).
async fn install_pacman(app: AppHandle, _distro: &LinuxDistro) -> Result<(), String> {
    let total: usize = 4;
    let mut step: usize = 0;

    step += 1;
    emit_progress(&app, step, total, "Updating system packages...", "in_progress");
    run_elevated_status("pacman", &["-Syu", "--noconfirm"])?;

    step += 1;
    emit_progress(&app, step, total, "Installing Docker and Docker Compose...", "in_progress");
    run_elevated_status("pacman", &["-S", "--noconfirm", "docker", "docker-compose"])?;

    step += 1;
    emit_progress(&app, step, total, "Starting Docker service...", "in_progress");
    run_elevated_status("systemctl", &["enable", "--now", "docker"])?;

    step += 1;
    emit_progress(&app, step, total, "Adding user to docker group...", "in_progress");
    let user = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "root".to_string());
    run_elevated_status("usermod", &["-aG", "docker", &user])?;

    emit_progress(&app, total, total, "Installation complete!", "completed");
    Ok(())
}

/// Install Docker on openSUSE-based distros (zypper).
async fn install_zypper(app: AppHandle, _distro: &LinuxDistro) -> Result<(), String> {
    let total: usize = 4;
    let mut step: usize = 0;

    step += 1;
    emit_progress(&app, step, total, "Refreshing repositories...", "in_progress");
    run_elevated_status("zypper", &["refresh"])?;

    step += 1;
    emit_progress(&app, step, total, "Installing Docker and Docker Compose...", "in_progress");
    run_elevated_status("zypper", &["install", "-y", "docker", "docker-compose"])?;

    step += 1;
    emit_progress(&app, step, total, "Starting Docker service...", "in_progress");
    run_elevated_status("systemctl", &["enable", "--now", "docker"])?;

    step += 1;
    emit_progress(&app, step, total, "Adding user to docker group...", "in_progress");
    let user = std::env::var("SUDO_USER")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "root".to_string());
    run_elevated_status("usermod", &["-aG", "docker", &user])?;

    emit_progress(&app, total, total, "Installation complete!", "completed");
    Ok(())
}

// ─── Install Command ───────────────────────────────────────────────────────

#[tauri::command]
pub async fn install_docker(app: AppHandle) -> Result<(), String> {
    // Only Linux is supported
    #[cfg(not(target_os = "linux"))]
    {
        emit_progress(&app, 0, 1, "Docker installer is only available on Linux", "error");
        return Err("Docker installer is only available on Linux".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        let distro = do_detect_linux_distro()?;

        if !distro.supported {
            emit_progress(
                &app,
                0,
                1,
                &format!("Unsupported distribution: {}", distro.name),
                "error",
            );
            return Err(format!(
                "Unsupported distribution: {}. Supported: Ubuntu, Debian, Fedora, RHEL, Arch, openSUSE",
                distro.name
            ));
        }

        match distro.package_manager.as_str() {
            "apt" => install_apt(app, &distro).await,
            "dnf" => install_dnf(app, &distro).await,
            "pacman" => install_pacman(app, &distro).await,
            "zypper" => install_zypper(app, &distro).await,
            other => Err(format!("Unsupported package manager: {}", other)),
        }
    }
}

// ─── Post-Install Validation ───────────────────────────────────────────────

#[tauri::command]
pub async fn validate_docker_install() -> Result<DockerInstallStatus, String> {
    // Run `docker run --rm hello-world` to verify the installation works
    let hello_world = Command::new("docker")
        .args(["run", "--rm", "hello-world"])
        .output();

    match hello_world {
        Ok(output) => {
            if !output.status.success() {
                let _stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                // Docker might fail if the user isn't in the docker group yet (needs re-login)
                // or if the daemon isn't running. Return current status instead.
                return check_docker_installed().await;
            }
            // hello-world succeeded — return full status
            check_docker_installed().await
        }
        Err(e) => {
            // docker command not found or failed to execute
            Err(format!("Docker validation failed: {}", e))
        }
    }
}
