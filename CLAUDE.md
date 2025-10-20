# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

waybarx is a modern Wayland bar that uses GTK4 layer-shell for panel semantics and WebKitGTK to render the UI. The key architectural insight is that GTK is only used for layer-shell window management—the entire UI is a web application (HTML/CSS/JS) rendered in a WebKitGTK WebView.

## Architecture

### Rust Backend (src/)

- **main.rs**: GTK application setup, layer-shell window configuration, monitor management
  - Spawns one bar per monitor using `spawn_bar_on_monitor()`
  - Handles monitor hotplug events
  - Builds WebView with `build_webview()` which conditionally loads dev server (debug) or embedded assets (release)
  - Layer-shell configuration: anchored to top edge, 36px exclusive zone

- **ui_bridge.rs**: JS ↔ Rust message channel
  - Registers "native" script message handler for JS → Rust communication
  - Rust → JS uses `web.evaluate_javascript()` calling `window.__nativeReceive`
  - Example echo handler—extend this for real commands

- **ipc.rs**: Optional compositor IPC modules (enabled via features)
  - `sway` feature: swayipc-async for workspace data
  - `hypr` feature: hyprland crate for workspace data
  - Both provide async `workspaces()` functions

### Web Frontend (web/)

- **main.ts**: Demo UI logic with TypeScript
  - `postNative(payload)` sends messages to Rust via `window.webkit.messageHandlers.native.postMessage`
  - `window.__nativeReceive` callback receives messages from Rust
  - Framework-agnostic—can be replaced with React/Vue/Svelte

- **vite.config.ts**: Dev server on 127.0.0.1:5173 (must match main.rs debug URL)

### Asset Embedding

In release builds, `rust-embed` bundles `web-dist/` into the binary. Vite is configured to output directly to `web-dist/` via `build.outDir` in `vite.config.ts`. The `vite-plugin-singlefile` plugin creates a self-contained HTML file with all CSS/JS inlined, which is loaded via `web.load_html()` in main.rs:43-45.

## Development Commands

### Dev Mode (two terminals required)

Terminal 1 (web dev server):
```sh
cd web
npm install
npm run dev
```

Terminal 2 (Rust app):
```sh
cargo run
# Or with IPC features:
cargo run --features sway
cargo run --features hypr
```

The Rust app loads `http://127.0.0.1:5173/` in debug mode.

### Release Build

**Easy method** (using build script):
```sh
./build.sh release hypr  # or 'sway' for Sway WM
```

**Manual method**:
```sh
# Build web assets (outputs to web-dist/ automatically)
cd web && npm install && npm run build && cd ..

# Build Rust binary with embedded assets
cargo build --release --features hypr
```

**Note**: Vite is configured to output to `web-dist/` (via `vite.config.ts`). The `vite-plugin-singlefile` plugin inlines all CSS/JS into a single `index.html` for embedding via `rust-embed`.

### Build Script

The `build.sh` helper script automates the full build process:

```sh
./build.sh [debug|release] [sway|hypr]
```

Examples:
- `./build.sh` - Debug build with hypr feature (default)
- `./build.sh release hypr` - Release build for Hyprland
- `./build.sh debug sway` - Debug build for Sway

The script:
1. Installs npm dependencies
2. Builds web assets to `web-dist/`
3. Verifies output files exist
4. Builds Rust binary with specified feature
5. Shows the path to the resulting binary

### Testing

No test suite is currently configured in the starter.

## System Prerequisites

- Wayland compositor with wlr-layer-shell support (Sway, Hyprland, Plasma Wayland)
- GTK4 and WebKitGTK 6.0 development libraries:
  - Debian/Ubuntu: `libgtk-4-dev libwebkitgtk-6.0-dev`
  - Fedora: `gtk4-devel webkitgtk6.0-devel`
  - Arch: `gtk4 webkitgtk-6.0`

GNOME Wayland does not support wlr-layer-shell; windows will fall back to regular windows.

## Adding Features

### New JS ↔ Rust Commands

The bridge in `src/ui_bridge.rs` now supports a command-based protocol:

1. **From JS**: Send `postNative({ cmd: 'command_name', ...args })`
2. **In Rust**: Add a new match arm in `ui_bridge.rs` for the command
3. **Call IPC**: Use `glib::spawn_future_local()` for async operations
4. **Response**: Send JSON back via `web.evaluate_javascript()` calling `window.__nativeReceive({ ok: true, cmd: 'command_name', data: ... })`

**Example**: The `get_workspaces` command (ui_bridge.rs:24-46) calls `ipc::hypr::workspaces()` and returns the workspace list.

### New Web Components

Add HTML/CSS/TS in `web/`. The web app is framework-agnostic—use vanilla JS, React, Vue, or Svelte as preferred. Just ensure the build outputs to the correct location for embedding.

### Monitor-Specific Bars

The current code spawns identical bars on all monitors. To customize per-monitor, store window references keyed by monitor in a global registry and pass monitor info to each `spawn_bar_on_monitor()` call.

## Configuration

- Bar height: `gls::set_exclusive_zone(&win, 36)` in main.rs:60
- Dev server URL: `web.load_uri("http://127.0.0.1:5173/")` in main.rs:37
- Application ID: `"dev.example.waybarx"` in main.rs:75
- Web Inspector: Enabled in debug builds only via `#[cfg(debug_assertions)]` (right-click bar → Inspect)
- Workspace update interval: `setInterval(fetchWorkspaces, 2000)` in web/main.ts:43 (2 seconds)
