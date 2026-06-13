# ItzamBox 🐳

> Local, free and open-source alternative to Docker Desktop.
> Built with **Tauri v2 + Vue 3 + Rust + SQLite3**.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)

---

## Description

**ItzamBox** is a native, high-performance desktop application for managing Docker containers, images, volumes, and networks from a premium graphical interface. Designed as a 100% free alternative to Docker Desktop with no commercial licensing restrictions.

### Key Features (MVP Phase 1)
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
sudo dpkg -i itzambox_1.0.0_amd64.deb

# Fedora / RHEL
sudo rpm -i itzambox-1.0.0-1.x86_64.rpm
```

### From source code

```bash
# Prerequisites
# - Node.js 18+
# - Rust 1.77+
# - Docker Engine 24.0+
# - System deps: libwebkit2gtk-4.1-dev, libgtk-3-dev

git clone git@github.com:JSilvaDev27/itzam-box.git
cd itzam-box
npm install
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

### Integrated Terminal
- Click the terminal bar at the bottom (or press `Ctrl+T`)
- Supports multiple tabs (Host shell + per-container terminals)
- GPU-accelerated rendering via xterm.js WebGL addon

### Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+K` | Command palette |
| `Ctrl+1`..`Ctrl+5` | Navigate sections |
| `Ctrl+T` | Host terminal |
| `Ctrl+Shift+T` | Toggle theme |
| `Ctrl+R` | Refresh |
| `Ctrl+,` | Settings |
| `Escape` | Close panel |

---

## Project Architecture

```
┌──────────────────────────────────────────┐
│  Vue 3 UI (Presentation)                │
├──────────────────────────────────────────┤
│  Tauri IPC Commands (Application)       │
├──────────────────────────────────────────┤
│  ContainerEngine Trait (Domain)          │
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
| Database | SQLite3 (rusqlite, WAL mode) |
| PTY | portable-pty |
| Host Metrics | sysinfo |

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
