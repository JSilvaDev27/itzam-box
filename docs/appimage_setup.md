# AppImage Setup — ItzamBox

## Status: ✅ Working

The Tauri build pipeline produces an AppImage automatically via the `tauri-bundler` crate. The `tauri.conf.json` already includes `"appimage"` in its `bundle.targets` array:

```json
"bundle": {
  "active": true,
  "targets": ["deb", "rpm", "appimage"],
  ...
}
```

## Prerequisites

The following system packages are required for AppImage generation:

```bash
# libfuse2 is required to run AppImages on most distributions
sudo apt-get install -y libfuse2

# (Optional) install appimagetool for custom AppImage workflows
wget -q "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage" \
  -O /tmp/appimagetool-x86_64.AppImage
chmod +x /tmp/appimagetool-x86_64.AppImage
sudo mv /tmp/appimagetool-x86_64.AppImage /usr/local/bin/appimagetool
```

## Building the AppImage

Run the standard Tauri build:

```bash
npx tauri build
```

The AppImage will be output at:

```
src-tauri/target/release/bundle/appimage/itzambox_1.3.0_amd64.AppImage
```

## linuxdeploy

`linuxdeploy` is available and verified working for custom AppImage packaging outside of Tauri's built-in pipeline.

Installation (if needed for CI or custom workflows):

```bash
wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage
sudo mv linuxdeploy-x86_64.AppImage /usr/local/bin/linuxdeploy
```

## Verification

After building, verify the AppImage runs:

```bash
./src-tauri/target/release/bundle/appimage/itzambox_1.3.0_amd64.AppImage --help
```

> **Note:** In sandboxed/CI environments without FUSE, AppImages may not execute.
> Use `--appimage-extract` as a fallback:
> ```bash
> ./itzambox_1.3.0_amd64.AppImage --appimage-extract
> ./squashfs-root/AppRun
> ```

## AppImage Release Workflow

1. Run `npx tauri build`
2. The AppImage is automatically generated in `bundle/appimage/`
3. Sign it with the updater private key (see signing key docs)
4. Upload to GitHub Releases alongside the `latest.json` manifest
