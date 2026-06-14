#!/usr/bin/env bash
# =============================================================================
# fix-appimage-webkit.sh — Post-build fix for WebKit helper binaries in AppImage
# =============================================================================
#
# PROBLEM:
#   WebKitGTK 4.1 has hardcoded relative paths (././/lib/x86_64-linux-gnu/...)
#   for its helper processes (WebKitNetworkProcess, WebKitWebProcess).
#   Inside the AppDir, these files are at usr/lib/... but WebKit looks for
#   lib/... relative to CWD. The AppRun.wrapped binary sets CWD to usr/
#   via chdir(), which makes it work, but running the binary directly fails.
#
# FIX:
#   1. Create lib → usr/lib symlink at AppDir root (belt: catches direct runs)
#   2. Create apprun-hooks/61-webkit-env.sh to export WEBKIT_EXEC_PATH and
#      WEBKIT_INJECTED_BUNDLE_DIR (suspenders: explicit is better)
#   3. Repackage the AppImage with the fixed AppDir
#
# USAGE:
#   After running `npx tauri build`:
#     bash scripts/fix-appimage-webkit.sh
#
#   Or combine:
#     npx tauri build && bash scripts/fix-appimage-webkit.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
APPIMAGE_DIR="${PROJECT_DIR}/src-tauri/target/release/bundle/appimage"
TAURI_CONFIG="${PROJECT_DIR}/src-tauri/tauri.conf.json"

# ── 1. Find the AppDir ───────────────────────────────────────────────────────
APPDIR="$APPIMAGE_DIR/ItzamBox.AppDir"

if [ ! -d "$APPDIR" ]; then
    echo "[fix-appimage-webkit] ❌ AppDir not found at $APPDIR"
    echo "[fix-appimage-webkit] ❌ Run 'npx tauri build' first"
    exit 1
fi

echo "[fix-appimage-webkit] ✅ Found AppDir at $APPDIR"

# ── 2. Create lib → usr/lib symlink ──────────────────────────────────────────
if [ -L "$APPDIR/lib" ]; then
    current_target="$(readlink "$APPDIR/lib")"
    echo "[fix-appimage-webkit] ℹ️  Symlink $APPDIR/lib already exists → $current_target"
elif [ -d "$APPDIR/lib" ]; then
    echo "[fix-appimage-webkit] ⚠️  $APPDIR/lib is a real directory — skipping symlink"
else
    ln -sf usr/lib "$APPDIR/lib"
    echo "[fix-appimage-webkit] ✅ Created symlink: $APPDIR/lib → usr/lib"
fi

# Verify the symlink resolves
if [ -f "$APPDIR/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess" ]; then
    echo "[fix-appimage-webkit] ✅ WebKitNetworkProcess accessible via lib/ symlink"
else
    echo "[fix-appimage-webkit] ⚠️  WebKitNetworkProcess NOT accessible via lib/ symlink"
fi

# ── 2b. Create CWD-independent wrapper for direct binary execution ──────────
# When the binary is run directly (not through AppRun which chdirs to AppDir root),
# the CWD is whatever directory the user's shell is in. WebKit's hardcoded
# relative path (././/lib/...) won't resolve. We install a wrapper that
# chdirs to the AppDir root before execing the real binary.
BINARY_PATH="$APPDIR/usr/bin/itzambox"
BINARY_REAL="$APPDIR/usr/bin/itzambox.bin"

if [ -f "$BINARY_PATH" ] && [ ! -f "$BINARY_REAL" ]; then
    echo "[fix-appimage-webkit] 🔧 Installing CWD-independent wrapper for direct binary execution..."

    # Rename the real binary
    mv "$BINARY_PATH" "$BINARY_REAL"

    # Create wrapper script
    cat > "$BINARY_PATH" << 'WRAPPER'
#!/usr/bin/env bash
# =============================================================================
# itzambox — CWD-independent wrapper for AppImage execution
# =============================================================================
# This wrapper ensures WebKit helper processes can be found regardless of
# the current working directory. AppRun.wrapped chdirs to the AppDir root,
# but when running the binary directly, the CWD may not be the AppDir root.
#
# The wrapper:
#   1. Determines its own location (AppDir/usr/bin/)
#   2. Changes CWD to the AppDir root (../.. relative to usr/bin/)
#   3. Sets WEBKIT_EXEC_PATH to help WebKit find its helper processes
#   4. Execs the real binary
# =============================================================================
set -euo pipefail

# Resolve the AppDir root (two levels up from usr/bin/)
SCRIPT_DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
APPDIR_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Change to AppDir root so WebKit's relative paths resolve via lib → usr/lib
cd "$APPDIR_ROOT"

# Set WebKit env vars for runtime stability
export WEBKIT_DISABLE_COMPOSITING_MODE=1

# Exec the real binary, preserving all arguments
exec "$SCRIPT_DIR/itzambox.bin" "$@"
WRAPPER
    chmod +x "$BINARY_PATH"

    echo "[fix-appimage-webkit] ✅ Installed wrapper: $BINARY_PATH"
    echo "[fix-appimage-webkit] ✅ Real binary renamed to: $BINARY_REAL"
    echo "[fix-appimage-webkit] ℹ️  Wrapper automatically chdirs to AppDir root before execution"
elif [ -f "$BINARY_REAL" ]; then
    echo "[fix-appimage-webkit] ℹ️  Wrapper already installed (itzambox.bin exists)"
fi

# ── 3. Create WebKit env var hook in apprun-hooks/ ───────────────────────────
HOOKS_DIR="$APPDIR/apprun-hooks"
mkdir -p "$HOOKS_DIR"

# The 61- prefix ensures this runs after linuxdeploy-plugin-gtk.sh which
# has a 6- prefix (alphabetical order: 61-webkit runs after 6-gtk)
cat > "$HOOKS_DIR/61-webkit-env.sh" << 'WEBKIT_HOOK'
#! /usr/bin/env bash
# 61-webkit-env.sh — WebKit env vars for AppImage runtime stability
# This hook is sourced by AppRun (generated by linuxdeploy)
#
# WebKitGTK 4.1 on Linux uses hardcoded relative paths (././/lib/...)
# for helper processes. The AppRun.wrapped binary changes CWD to usr/,
# so these resolve correctly. We also set WEBKIT_DISABLE_COMPOSITING_MODE=1
# to prevent GPU compositing issues inside AppImage sandbox.
# Note: WEBKIT_EXEC_PATH is not available in this WebKit build — the
# lib → usr/lib symlink at the AppDir root handles path resolution.

export WEBKIT_DISABLE_COMPOSITING_MODE=1
WEBKIT_HOOK

chmod +x "$HOOKS_DIR/61-webkit-env.sh"
echo "[fix-appimage-webkit] ✅ Created hook: $HOOKS_DIR/61-webkit-env.sh"

# ── 4. Repackage the AppImage ────────────────────────────────────────────────
# Find the product name and version from tauri.conf.json
PRODUCT_NAME="ItzamBox"
if command -v python3 &>/dev/null && [ -f "$TAURI_CONFIG" ]; then
    PRODUCT_NAME=$(python3 -c "
import json
with open('$TAURI_CONFIG') as f:
    cfg = json.load(f)
print(cfg.get('productName', 'ItzamBox'))
" 2>/dev/null || echo "ItzamBox")
fi

# Find version from Cargo.toml
VERSION="1.3.0"
CARGO_TOML="${PROJECT_DIR}/src-tauri/Cargo.toml"
if [ -f "$CARGO_TOML" ]; then
    VERSION=$(grep '^version = ' "$CARGO_TOML" | head -1 | sed 's/version = "\(.*\)"/\1/' 2>/dev/null || echo "1.3.0")
fi

ARCH="x86_64"
OUTPUT_APPIMAGE="${APPIMAGE_DIR}/${PRODUCT_NAME}_${VERSION}_${ARCH}.AppImage"
OUTPUT_TMP="${APPIMAGE_DIR}/.${PRODUCT_NAME}_${VERSION}_${ARCH}.AppImage.tmp"

echo "[fix-appimage-webkit] 🏗️  Repackaging AppImage..."
echo "[fix-appimage-webkit]    Output: $OUTPUT_APPIMAGE"

# Check if appimagetool is available
APPIMAGETOOL=""
for tool in /home/josue/.local/bin/appimagetool /usr/local/bin/appimagetool /usr/bin/appimagetool; do
    if command -v "$tool" &>/dev/null; then
        APPIMAGETOOL="$tool"
        break
    fi
done

if [ -z "$APPIMAGETOOL" ] && command -v appimagetool &>/dev/null; then
    APPIMAGETOOL="appimagetool"
fi

if [ -z "$APPIMAGETOOL" ]; then
    echo "[fix-appimage-webkit] ⚠️  appimagetool not found — can't repackage AppImage"
    echo "[fix-appimage-webkit] ⚠️  AppDir is fixed but new AppImage needs manual repackaging"
    echo "[fix-appimage-webkit] ℹ️  Install with: sudo apt install appimagetool"
    echo "[fix-appimage-webkit] ℹ️  Then run: appimagetool '$APPDIR' '$OUTPUT_APPIMAGE'"
    echo ""
    echo "[fix-appimage-webkit] ✅ AppDir patched successfully at:"
    echo "    $APPDIR"
    echo "    Symlink: lib → usr/lib"
    echo "    Hook:    apprun-hooks/61-webkit-env.sh"
    echo ""
    echo "    NEXT BUILD will include these fixes automatically."
    exit 0
fi

if [ -f "$OUTPUT_APPIMAGE" ]; then
    echo "[fix-appimage-webkit] Backing up existing AppImage..."
    mv "$OUTPUT_APPIMAGE" "${OUTPUT_APPIMAGE}.bak"
fi

# Repackage using appimagetool
"$APPIMAGETOOL" "$APPDIR" "$OUTPUT_TMP" 2>&1 | while IFS= read -r line; do
    echo "[appimagetool] $line"
done

# Check if the repackaged AppImage was created
if [ -f "$OUTPUT_TMP" ]; then
    mv "$OUTPUT_TMP" "$OUTPUT_APPIMAGE"
    chmod +x "$OUTPUT_APPIMAGE"
    FILE_SIZE=$(stat -c%s "$OUTPUT_APPIMAGE" 2>/dev/null || stat -f%z "$OUTPUT_APPIMAGE" 2>/dev/null || echo "unknown")
    echo "[fix-appimage-webkit] ✅ Repackaged AppImage: $OUTPUT_APPIMAGE"
    echo "[fix-appimage-webkit]    Size: $FILE_SIZE bytes ($(( FILE_SIZE / 1048576 )) MB)"
    
    # Remove backup if successful
    rm -f "${OUTPUT_APPIMAGE}.bak"
else
    echo "[fix-appimage-webkit] ❌ Failed to repackage AppImage"
    # Restore backup
    if [ -f "${OUTPUT_APPIMAGE}.bak" ]; then
        mv "${OUTPUT_APPIMAGE}.bak" "$OUTPUT_APPIMAGE"
    fi
    exit 1
fi

echo ""
echo "[fix-appimage-webkit] ✅ All fixes applied successfully!"
echo "    • Symlink:  $APPDIR/lib → usr/lib"
echo "    • Hook:     apprun-hooks/61-webkit-env.sh"
echo "    • Wrapper:  usr/bin/itzambox → chdir → itzambox.bin"
echo "    Next build will also include these fixes permanently."
echo "    (via GTK plugin patch in scripts/fix-linuxdeploy.sh)"
