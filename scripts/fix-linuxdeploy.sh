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

RSVG_OK=false

# ── 1. Check if it's already available via pkg-config ──────────────────────
if pkg-config --exists librsvg-2.0 2>/dev/null; then
    echo "[fix-linuxdeploy] ✅ librsvg-2.0 found via pkg-config ($(pkg-config --modversion librsvg-2.0))"
    RSVG_OK=true
fi

# ── 2. Try installing the real package ─────────────────────────────────────
if [ "$RSVG_OK" = false ]; then
    if command -v sudo &>/dev/null; then
        echo "[fix-linuxdeploy] Attempting to install librsvg2-dev via sudo..."
        if sudo apt-get install -y librsvg2-dev 2>/dev/null; then
            echo "[fix-linuxdeploy] ✅ Installed librsvg2-dev successfully."
            RSVG_OK=true
        fi
    fi
fi

if [ "$RSVG_OK" = false ]; then
    if command -v pkexec &>/dev/null; then
        echo "[fix-linuxdeploy] Attempting to install librsvg2-dev via pkexec..."
        if pkexec apt-get install -y librsvg2-dev 2>/dev/null; then
            echo "[fix-linuxdeploy] ✅ Installed librsvg2-dev via pkexec."
            RSVG_OK=true
        fi
    fi
fi

# ── 3. Fallback: generate a local .pc file ─────────────────────────────────
if [ "$RSVG_OK" = false ]; then
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
        RSVG_OK=true
    else
        echo "[fix-linuxdeploy] ❌ Failed to make librsvg-2.0 resolvable. Check system dependencies."
        # Non-fatal: continue to GTK plugin patch
    fi
fi

# ── 4. Patch the GTK plugin to auto-apply WebKit fix on every build ──────────
# The GTK plugin already patches libwebkit*.so to replace /usr with ././,
# but it does NOT create the lib → usr/lib symlink at the AppDir root.
# Without this symlink, WebKit's hardcoded relative path to helper processes
# (././/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess) fails
# unless CWD happens to be the AppDir root.
echo ""
echo "[fix-linuxdeploy] Ensuring GTK plugin applies WebKit AppImage fix..."

GTK_PLUGIN="${HOME}/.cache/tauri/linuxdeploy-plugin-gtk.sh"
WEBKIT_MARKER="### ITZAMBOX WEBKIT FIX ###"

if [ ! -f "$GTK_PLUGIN" ]; then
    echo "[fix-linuxdeploy] ⚠️  GTK plugin not found at $GTK_PLUGIN"
    echo "[fix-linuxdeploy] ⚠️  WebKit fix will be applied post-build by fix-appimage-webkit.sh"
else
    if grep -q "$WEBKIT_MARKER" "$GTK_PLUGIN" 2>/dev/null; then
        echo "[fix-linuxdeploy] ℹ️  GTK plugin already patched with WebKit fix (skipping)"
    else
        echo "[fix-linuxdeploy] 🔧 Patching GTK plugin to auto-create WebKit fix..."
        cat >> "$GTK_PLUGIN" << 'WEBKITFIX'

### ITZAMBOX WEBKIT FIX ###
# WebKitGTK 4.1 has hardcoded relative paths (././/lib/...) for helper processes
# (WebKitNetworkProcess, WebKitWebProcess). The GTK plugin replaces /usr with ././
# in libwebkit*.so, but the AppDir root needs a lib → usr/lib symlink so that
# these relative paths resolve regardless of how the process is launched.
echo "itzambox webkit: Creating lib → usr/lib symlink at AppDir root"
ln -sf usr/lib "$APPDIR/lib"

# Also create the env hook for WebKit runtime stability
cat > "$APPDIR/apprun-hooks/61-webkit-env.sh" << 'WEBKITHOOK'
#! /usr/bin/env bash
# 61-webkit-env.sh — WebKit env vars for AppImage runtime stability
export WEBKIT_DISABLE_COMPOSITING_MODE=1
WEBKITHOOK
chmod +x "$APPDIR/apprun-hooks/61-webkit-env.sh"
echo "itzambox webkit: Created apprun-hooks/61-webkit-env.sh"

# Create CWD-independent wrapper for direct binary execution
# When the binary is run directly (not through AppRun), the CWD may not be
# the AppDir root, so WebKit's relative paths won't resolve. The wrapper
# chdirs to the AppDir root before execing the real binary.
BINARY_PATH="$APPDIR/usr/bin/itzambox"
BINARY_REAL="$APPDIR/usr/bin/itzambox.bin"
if [ -f "$BINARY_PATH" ] && [ ! -f "$BINARY_REAL" ]; then
    echo "itzambox webkit: Installing CWD-independent wrapper"
    mv "$BINARY_PATH" "$BINARY_REAL"
    cat > "$BINARY_PATH" << 'WRAPPER'
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
APPDIR_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$APPDIR_ROOT"
export WEBKIT_DISABLE_COMPOSITING_MODE=1
exec "$SCRIPT_DIR/itzambox.bin" "$@"
WRAPPER
    chmod +x "$BINARY_PATH"
    echo "itzambox webkit: Installed wrapper at usr/bin/itzambox → itzambox.bin"
elif [ -f "$BINARY_REAL" ]; then
    echo "itzambox webkit: Wrapper already installed"
fi
### END ITZAMBOX WEBKIT FIX ###
WEBKITFIX
        echo "[fix-linuxdeploy] ✅ GTK plugin patched with WebKit fix"
    fi
fi
