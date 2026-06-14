# ItzamBox 🐳

> Local, free and open-source alternative to Docker Desktop.
> Built with **Tauri v2 + Vue 3 + Rust + SQLite3**.

[![Version](https://img.shields.io/badge/version-1.2.0-blue.svg)](https://github.com/JSilvaDev27/itzam-box/releases)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![CI](https://github.com/JSilvaDev27/itzam-box/actions/workflows/ci.yml/badge.svg)](https://github.com/JSilvaDev27/itzam-box/actions/workflows/ci.yml)
[![Rust Coverage](https://img.shields.io/badge/rust%20coverage-92%25-success.svg)](./coverage)
[![Vitest Coverage](https://img.shields.io/badge/vitest%20coverage-89%25-success.svg)](./coverage)

---

## Description

**ItzamBox** is a native, high-performance desktop application for managing Docker containers, images, volumes, networks, and orchestration resources from a premium graphical interface. Designed as a 100% free alternative to Docker Desktop with no commercial licensing restrictions.

### Features (v1.0.0)

- 🖥️ **Dashboard** with real-time host metrics (CPU, RAM)
- 📦 **Container management**: start, stop, restart, pause, remove, inspect
- 🖼️ **Image management**: list, pull, remove, tag
- 💾 **Volume management**: list, create, remove
- 🌐 **Network management**: list, create, remove
- ⌨️ **Integrated terminal** with PTY + xterm.js (GPU-accelerated)
- 🌙 **Dark/Light theme** with instant toggle
- 🌍 **i18n**: Spanish / English
- ⚡ **Keyboard shortcuts**: Ctrl+K, Ctrl+1-5, Ctrl+T, Ctrl+R
- 🔔 **Notification system** with toast alerts
- 💿 **SQLite3 persistence** for settings and history

### Features (v1.1.0)

- ✍️ **Monaco Compose Editor** — YAML syntax highlighting, validation, and formatting for `docker-compose.yml` files
- 🔔 **Notification Persistence** — SQLite-backed notification history with read/unread status and filtering
- 🚀 **CI/CD Pipeline** — GitHub Actions workflow covering lint, test, security, build, and e2e stages

### Features (v1.2.0)

- ☸️ **Kubernetes Cluster Viewer** — read-only inspection of pods, deployments, services, and configmaps with automatic secret redaction
- 🐝 **Docker Swarm Mode** — init/join/leave swarm, manage nodes, services, and stacks with interactive SVG topology
- 💾 **Backup & Restore** — volume snapshots via tar archives, SHA-256 checksums, and built-in cron scheduler
- 📈 **Historical Metrics** — CPU/Memory/Network/Disk time-series charts with CSV/JSON/PNG export
- ✨ **UI/UX Polish** — page transitions, skeleton loaders, animated counters, and ripple effects

---

## System Requirements

| Requirement | Minimum |
|---|---|
| Operating System | Linux (Ubuntu 20.04+, Fedora 38+, Arch) |
| Docker Engine | 24.0+ |
| RAM | 4 GB |
| Disk | 500 MB |

---

## Installation

### From precompiled binary (.deb / .rpm)

```bash
# Debian / Ubuntu
sudo dpkg -i itzambox_1.2.0_amd64.deb

# Fedora / RHEL
sudo rpm -i itzambox-1.2.0-1.x86_64.rpm
```

### From source code

```bash
# Prerequisites
# - Node.js 22+
# - Rust 1.77+
# - Docker Engine 24.0+
# - System deps: build-essential, libssl-dev, libwebkit2gtk-4.1-dev, libgtk-3-dev, libsoup-3.0-dev, javascriptcoregtk-4.1

git clone git@github.com:JSilvaDev27/itzam-box.git
cd itzam-box
pnpm install
cargo tauri dev     # Development mode
cargo tauri build   # Production build
```

---

## Usage

### First Run

On first launch, ItzamBox guides you through an onboarding wizard:

1. Welcome screen
2. Theme and language selection
3. Docker Engine verification
4. Keyboard shortcuts tour

### Container Management

- **Dashboard** shows active containers, host metrics, and quick stats
- **Containers view** lists all containers with live CPU/RAM stats
- Click any container to expand detail panel (Logs, Inspect, Files, Stats)
- Right-click for context menu with all actions

### Compose Editor

- Open any `docker-compose.yml` file from the **Compose** section
- Edit with Monaco-powered YAML highlighting and inline validation
- Format the document with `Shift+Alt+F`
- Deploy or validate the stack directly from the editor toolbar

### Kubernetes Cluster Viewer

- Navigate to **Kubernetes** after configuring `~/.kube/config`
- Browse pods, deployments, services, and configmaps across namespaces
- Secret values are automatically redacted and hidden from view
- Stream logs and view events for any selected resource

### Docker Swarm Mode

- Go to **Swarm > Initialize** to create a new swarm, or **Join** an existing one
- Manage nodes, services, and stacks from the sidebar
- View the live SVG topology of services and their network relationships
- Leave the swarm safely from the swarm actions menu

### Backup & Restore

- Select one or more volumes from **Volumes > Backup**
- Snapshots are created as compressed tar archives with SHA-256 checksums
- Schedule recurring backups with the built-in cron scheduler
- Restore a snapshot to a new or existing volume from **Volumes > Restore**

### Historical Metrics

- Open **Dashboard > Metrics History** to explore time-series data
- Filter by resource type: CPU, Memory, Network, or Disk
- Export any chart to CSV, JSON, or PNG

### Integrated Terminal

- Click the terminal bar at the bottom (or press `Ctrl+T`)
- Supports multiple tabs (Host shell + per-container terminals)
- GPU-accelerated rendering via xterm.js WebGL addon

### Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+K` | Command palette |
| `Ctrl+1`..`Ctrl+6` | Navigate sections |
| `Ctrl+T` | Host terminal |
| `Ctrl+Shift+T` | Toggle theme |
| `Ctrl+R` | Refresh |
| `Ctrl+,` | Settings |
| `Escape` | Close panel |

---

## Testing

The project maintains a comprehensive test suite across the Rust backend, frontend unit tests, and end-to-end flows.

| Suite | Count |
|---|---|
| Rust unit / integration tests | 153 |
| Vitest frontend tests | 74 |
| Playwright E2E scenarios | 102 |

### CI Pipeline

```text
lint → test → security → build → e2e
```

- **Lint**: `cargo clippy`, `cargo fmt`, `pnpm lint`
- **Test**: Rust tests + Vitest with coverage reporting
- **Security**: `cargo audit`, dependency scanning, secret detection
- **Build**: Tauri production build for Linux
- **E2E**: Playwright scenarios against the packaged application

---

## Project Architecture

```
┌──────────────────────────────────────────┐
│  Vue 3 UI (Presentation)                │
├──────────────────────────────────────────┤
│  Tauri IPC Commands (Application)       │
├──────────────────────────────────────────┤
│  ContainerEngine Trait (Domain)         │
├──────────────────────────────────────────┤
│  Docker CLI / REST API (Infrastructure) │
└──────────────────────────────────────────┘
```

- **Clean Architecture** with Hexagonal (Ports & Adapters) pattern
- **Trait-based engine abstraction** for future Podman/containerd support
- **Event-driven state management** via Tauri events
- **Input sanitization** layer preventing shell injection

---

## Technologies

| Component | Technology |
|---|---|
| Desktop Framework | Tauri v2 (Rust) |
| Frontend | Vue 3 + TypeScript + Vite |
| Styling | Pure CSS3 (Custom Properties) |
| Icons | FontAwesome 6 Free |
| Terminal | xterm.js + WebGL |
| Charts | Chart.js + uPlot |
| Compose Editor | Monaco Editor |
| Database | SQLite3 (rusqlite, WAL mode) |
| YAML Handling | serde_yaml |
| Checksums | sha2 |
| PTY | portable-pty |
| Host Metrics | sysinfo |
| Unit Testing (Frontend) | Vitest |
| E2E Testing | Playwright |
| Error Monitoring | Sentry |

---

## Screenshots

> Screenshots are generated during CI and published to the release assets.

| Screenshot | Description |
|---|---|
| Dashboard | Real-time host metrics, active containers, and quick actions |
| Container Details | Logs, inspect JSON, file browser, and live stats for a selected container |
| Compose Editor | Monaco YAML editor with validation and one-click stack deployment |
| Kubernetes Viewer | Read-only cluster resource inspection with secret redaction |
| Swarm Topology | Interactive SVG diagram of services, nodes, and overlay networks |

---

## Contributing

Contributions are welcome! Please ensure:

1. Code follows project standards (`docs/standards.md`)
2. `cargo fmt` and `cargo clippy` pass
3. New code includes unit tests
4. Commit messages follow conventional commits format

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**.

See the [LICENSE](./LICENSE) file for the full text.

You are free to use, modify, and distribute this software, provided that any derivative work is distributed under the same GPL-3.0 license.

---

© 2026 SodigTech. Built by Lic. Josué A. Silva.
