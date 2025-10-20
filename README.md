# waybarx (starter)

A modern, lightweight Wayland bar that uses **gtk4-layer-shell** to get real panel semantics (anchors & exclusive zone) and renders the UI with **WebKitGTK** so you can write your bar in **HTML/CSS/JS** (framework-agnostic).

> GTK is used only to grant the *layer-shell* role; **your UI is a full web app** rendered by WebKitGTK. No GTK CSS is required.

## Features

- Layer-shell window: anchored to the top edge, reserves space (exclusive zone), one instance per monitor.
- Modern web UI: CSS Grid/Flex, container queries, backdrop-filter, etc. (depends on your WebKitGTK version).
- JS ↔ Rust bridge: message channel `native` for commands/events.
- Dev-friendly: point WebView at Vite dev server in debug; embed static files in release.

## Build prerequisites (Linux/Wayland)

You need system packages for GTK4 and WebKitGTK 6.0:
- Debian/Ubuntu: `sudo apt install libgtk-4-dev libwebkitgtk-6.0-dev`
- Fedora: `sudo dnf install gtk4-devel webkitgtk6.0-devel`
- Arch: `sudo pacman -S gtk4 webkitgtk` (ensure 6.x APIs available)
- Also install `wayland-protocols` if your distro splits it out.

> Confirm that `gtk4-layer-shell` works on your compositor (Sway/Hyprland/Plasma Wayland supported). GNOME Wayland does **not** implement wlr-layer-shell for panels; the window will fall back to a normal window without reservation.

## Run (dev mode)

Terminal A (web):
```sh
cd web
npm i
npm run dev
```

Terminal B (Rust app):
```sh
cargo run
```

The app will load `http://127.0.0.1:5173/` into the bar WebView.

## Build (release)

```sh
# Build web assets
(cd web && npm i && npm run build)

# Bundle into the binary
cargo build --release
```

This will embed files from `web-dist/` into the executable.

## Optional: Sway/Hypr IPC

Enable a feature to use the corresponding IPC crate and implement real modules:

```sh
# Sway/i3
cargo run --features sway

# Hyprland
cargo run --features hypr
```

Wire up JS calls in `src/ui_bridge.rs` and implement async tasks in `src/ipc.rs` to push data into the web UI (`evaluate_javascript` / `window.postMessage`).

## License

MIT
