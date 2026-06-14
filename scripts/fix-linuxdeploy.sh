#!/usr/bin/env bash
#
# fix-linuxdeploy.sh — Permanent fix for linuxdeploy GTK plugin build failures.
#
# PROBLEM:  The AppImage bundling step in `npx tauri build` fails with
#           "failed to bundle project `failed to run linuxdeploy`" because
#           the linuxdeploy GTK plugin cannot find librsvg-2.0.pc via pkg-config.
#
# ROOT CAUSE: The `librsvg2-dev` system package is missing. This package
#             provides the pkg-config metadata file that linuxdeploy needs.
#
# FIX:      1. Try installing librsvg2-dev via apt (apt-get install -y librsvg2-dev).
#           2. If that fails (no sudo), generate the .pc file in a location
#              that pkg-config searches and export PKG_CONFIG_PATH.
#
# This script is idempotent — safe to run before every build.
# ---------------------------------------------------------------------------

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "[fix-linuxdeploy] Checking for librsvg-2.0 pkg-config metadata..."

# ── 1. Check if it's already available via pkg-config ──────────────────────
if pkg-config --exists librsvg-2.0 2>/dev/null; then
    echo "[fix-linuxdeploy] ✅ librsvg-2.0 found via pkg-config ($(pkg-config --modversion librsvg-2.0))"
    exit 0
fi

# ── 2. Try installing the real package ─────────────────────────────────────
if command -v sudo &>/dev/null; then
    echo "[fix-linuxdeploy] Attempting to install librsvg2-dev via sudo..."
    if sudo apt-get install -y librsvg2-dev 2>/dev/null; then
        echo "[fix-linuxdeploy] ✅ Installed librsvg2-dev successfully."
        exit 0
    fi
fi

if command -v pkexec &>/dev/null; then
    echo "[fix-linuxdeploy] Attempting to install librsvg2-dev via pkexec..."
    if pkexec apt-get install -y librsvg2-dev 2>/dev/null; then
        echo "[fix-linuxdeploy] ✅ Installed librsvg2-dev via pkexec."
        exit 0
    fi
fi

# ── 3. Fallback: generate a local .pc file ─────────────────────────────────
echo "[fix-linuxdeploy] ⚠️  Could not install librsvg2-dev. Generating local .pc file..."

LOCAL_PC_DIR="${HOME}/.local/share/pkgconfig"
mkdir -p "$LOCAL_PC_DIR"

# Detect the system librsvg2 runtime version for the .pc file
RSVG_VERSION="$(dpkg -l librsvg2-2 2>/dev/null | awk '/^ii/{print $3}' | cut -d+ -f1)"
RSVG_VERSION="${RSVG_VERSION:-2.58.0}"

cat > "$LOCAL_PC_DIR/librsvg-2.0.pc" << EOF
prefix=/usr
exec_prefix=\${prefix}
libdir=\${exec_prefix}/lib/x86_64-linux-gnu
includedir=\${prefix}/include/librsvg-2.0

Name: librsvg-2.0
Description: SVG rendering library
Version: ${RSVG_VERSION}
Requires: glib-2.0 gio-2.0 gdk-pixbuf-2.0 cairo
Libs: -L\${libdir} -lrsvg-2
Cflags: -I\${includedir}
EOF

export PKG_CONFIG_PATH="${LOCAL_PC_DIR}:${PKG_CONFIG_PATH:-}"

echo "[fix-linuxdeploy] ✅ Generated ${LOCAL_PC_DIR}/librsvg-2.0.pc (version ${RSVG_VERSION})"
echo "[fix-linuxdeploy] ℹ️  PKG_CONFIG_PATH set to: ${PKG_CONFIG_PATH}"

# Verify
if pkg-config --exists librsvg-2.0 2>/dev/null; then
    echo "[fix-linuxdeploy] ✅ librsvg-2.0 now resolvable via pkg-config."
else
    echo "[fix-linuxdeploy] ❌ Failed to make librsvg-2.0 resolvable. Check system dependencies."
    exit 1
fi
