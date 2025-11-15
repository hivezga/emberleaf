# REL-002C — AppImage linuxdeploy crash: verbose capture + version pin + containerized build

**Goal:** Reproduce and capture actionable logs from linuxdeploy failure, isolate root cause by pinning versions and/or building in Ubuntu 22.04 container where linuxdeploy is battle-tested.

## Why

AppImage build has failed on 7 consecutive v1.0.0-rc1 attempts despite multiple fixes:
- ✅ Added FUSE environment variable (APPIMAGE_EXTRACT_AND_RUN=1)
- ✅ Added libfuse2t64 for Ubuntu 24.04
- ✅ Disabled binary stripping (NO_STRIP=true)
- ❌ linuxdeploy still fails silently

**Current behavior:** Build logs show only `failed to run linuxdeploy` with no diagnostic output.

**Impact:** Blocks AppImage artifact generation for v1.0.0 release.

**Workaround:** Made AppImage step non-blocking with `continue-on-error: true` to ship .deb while debugging.

## Deliverables

- CI job `appimage-debug` with:
  - **Verbose logging** from Tauri CLI, bundler, linuxdeploy, and appimagetool
  - **Version pinning** for linuxdeploy/appimagetool to known-good versions
  - **Log artifacts** uploaded to GitHub Actions for offline analysis
  - **Ubuntu 22.04 containerized build** (alternate approach)
- One of the above produces either:
  - ✅ Working AppImage artifact, or
  - ✅ Clear failure reason with actionable error message

## Acceptance Criteria

- [ ] A CI run with **verbose linuxdeploy/appimagetool logs** saved as artifacts
- [ ] A run that **pins linuxdeploy/appimagetool** to known-good versions
- [ ] A run that builds AppImage **inside Ubuntu 22.04 container** and attaches the artifact
- [ ] One of the above produces a working AppImage **OR** a clear failure reason

## Implementation

### Approach 1: Verbose Logging Job

Add to `.github/workflows/release.yml`:

```yaml
  appimage-debug:
    name: AppImage Debug (Verbose Logs)
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable

      - name: Install system deps
        run: |
          sudo apt-get update
          sudo apt-get install -y \
            libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
            libssl-dev libglib2.0-dev build-essential pkg-config \
            libasound2-dev libpulse-dev pipewire-audio \
            libfuse2t64

      - name: Install JS deps
        run: npm ci

      - name: Build frontend
        run: npm run build

      - name: Build AppImage with verbose logging
        env:
          APPIMAGE_EXTRACT_AND_RUN: 1
          NO_STRIP: true
          TAURI_CLI_LOG: debug
          TAURI_BUNDLER_VERBOSE: true
          RUST_BACKTRACE: 1
          RUST_LOG: tauri=debug,tauri_bundler=debug
        run: |
          npm run tauri build -- --bundles appimage --verbose 2>&1 | tee appimage-build.log

      - name: Upload build logs
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: appimage-debug-logs
          path: |
            appimage-build.log
            src-tauri/target/release/bundle/**/*.log
          if-no-files-found: ignore
          retention-days: 7

      - name: Upload AppImage if successful
        if: success()
        uses: actions/upload-artifact@v4
        with:
          name: appimage-debug-artifact
          path: src-tauri/target/release/bundle/appimage/*.AppImage
          if-no-files-found: ignore
```

### Approach 2: Version Pinning

Pin linuxdeploy and appimagetool to last known-good versions:

```yaml
      - name: Download pinned linuxdeploy
        run: |
          mkdir -p ~/.cache/tauri
          cd ~/.cache/tauri
          # Pin to specific release
          wget https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
          wget https://github.com/AppImage/AppImageKit/releases/download/13/appimagetool-x86_64.AppImage
          chmod +x *.AppImage
          # Verify
          APPIMAGE_EXTRACT_AND_RUN=1 ./linuxdeploy-x86_64.AppImage --version || true
          APPIMAGE_EXTRACT_AND_RUN=1 ./appimagetool-x86_64.AppImage --version || true
```

### Approach 3: Ubuntu 22.04 Container

Build inside Docker container with Ubuntu 22.04 where AppImage tooling is battle-tested:

```yaml
      - name: Build AppImage in Ubuntu 22.04 container
        run: |
          docker run --rm -v $PWD:/workspace -w /workspace ubuntu:22.04 bash -c "
            apt-get update && \
            apt-get install -y curl wget git build-essential pkg-config \
              libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev \
              libssl-dev libglib2.0-dev libasound2-dev libpulse-dev libfuse2 && \
            curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && \
            apt-get install -y nodejs && \
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
            . ~/.cargo/env && \
            npm ci && \
            npm run build && \
            npm run tauri build -- --bundles appimage
          "
```

## Likely Culprits

Based on 7 failed attempts, we're targeting:

1. **AppImage tool version mismatch** - Ubuntu 24.04 has glibc/FUSE incompatibilities with linuxdeploy
2. **Binary compatibility** - Strip/patchelf failing on specific binary despite NO_STRIP workaround
3. **Metadata validation** - Desktop entry or icon misconfig that linuxdeploy treats as fatal
4. **Silent dependency** - Missing runtime library that linuxdeploy needs but doesn't declare

## Local Repro Checklist

```bash
# Enable verbose logging
export TAURI_CLI_LOG=debug
export TAURI_BUNDLER_VERBOSE=true
export RUST_LOG=tauri=debug,tauri_bundler=debug

# Run bundler
npm run tauri build -- --bundles appimage --verbose

# Check logs
cat src-tauri/target/release/bundle/**/*.log

# Validate metadata
cat src-tauri/tauri.conf.json | jq '.bundle'

# Check icon sizes
ls -lh src-tauri/icons/
```

## Related

- Blocks: v1.0.0-rc1 AppImage artifact
- Related: REL-002 (packaging bootstrap), REL-002B (Tauri bundler)
- Workaround: PR #18 (continue-on-error for AppImage)
- Unblocks: RC testing with both .deb and AppImage packages

## Timeline

- **Priority:** Medium (not blocking .deb release)
- **Target:** Debug before v1.0.0 final
- **Fallback:** Ship v1.0.0 with .deb only, AppImage in v1.0.1

## References

- [Tauri AppImage bundling docs](https://v2.tauri.app/reference/config/#appimageconfig)
- [linuxdeploy GitHub](https://github.com/linuxdeploy/linuxdeploy)
- [Ubuntu 24.04 FUSE changes](https://discourse.ubuntu.com/t/spec-packaging-libfuse2-for-24-04-lts/42290)
- [Tauri issue #13113](https://github.com/tauri-apps/tauri/issues/13113) - NO_STRIP workaround
