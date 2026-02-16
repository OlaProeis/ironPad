# System Tray Implementation (v0.2.0) ✅

**Status:** Implemented

**Goal:** Replace the CMD window with a system tray icon. Users interact via tray menu: "Open in Browser" or "Quit". No console window on Windows.

---

## Overview

- **Scope:** Single codebase, cross-platform. CI/CD unchanged — same build pipeline produces one binary per OS.
- **Crate:** `tray-item = "0.10"` (cross-platform tray icon with simple API)
- **Windows icon:** `windows-sys = "0.52"` for loading the standard application icon via `LoadIconW`

---

## What Was Implemented

### 1. Dependencies (`backend/Cargo.toml`)

```toml
# System tray (production mode)
tray-item = "0.10"

# Windows system icon loading (for tray icon)
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging"] }
```

### 2. Windows: Hide Console Window (`backend/src/main.rs`)

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

- **Debug builds (`cargo run`):** Console remains for logs.
- **Release builds (`cargo build --release`):** No CMD window on Windows.

### 3. Restructured `main()` for Dual-Mode Operation

```
main()
├── Development mode (no static/index.html next to exe)
│   └── Normal tokio runtime on main thread, API-only, no tray
└── Production mode (static/index.html exists)
    └── run_with_tray()
        ├── Background thread: tokio runtime + Axum server
        ├── mpsc channel: server sends bound port back to main thread
        ├── Auto-open browser (400ms delay)
        └── Main thread: tray icon + event loop
```

### 4. Tray Icon (Platform-Specific)

| Platform | Icon Source |
|----------|------------|
| Windows  | `IDI_APPLICATION` via `LoadIconW` (standard system app icon) |
| macOS    | `IconSource::Resource("")` (default icon) |
| Linux    | `IconSource::Resource("application-x-executable")` (icon theme) |

### 5. Tray Menu

- **"Open in Browser"** — calls `webbrowser::open()` with `http://localhost:{port}`
- **"Quit"** — calls `std::process::exit(0)` to shut down server and exit

### 6. Threading Model

The tray event loop runs on the **main thread** (required on macOS, safe on all platforms). The Axum server runs on a **background thread** with its own `tokio::runtime::Runtime`. Port discovery uses `std::sync::mpsc::channel`.

---

## Behaviour Summary

| Before (v0.1.0)        | After (v0.2.0)                     |
|------------------------|------------------------------------|
| CMD window visible     | No console window (Windows release)|
| Browser opens on start | Browser opens on start + via tray  |
| Quit via Ctrl+C        | Quit via tray menu or Ctrl+C (dev) |

---

## Testing Checklist

- [x] Backend compiles with new dependencies (`cargo check`)
- [ ] Windows: No CMD window when running release binary
- [ ] Windows: Tray icon appears; "Open in Browser" opens correct URL
- [ ] Windows: "Quit" exits cleanly
- [ ] macOS: Tray icon in menu bar; menu works
- [ ] Linux: Tray icon in system tray; menu works
- [ ] Development mode (`cargo run`): Behaviour unchanged (no tray, API-only)

---

## Icon Asset

Currently using platform default icons (Windows system app icon, macOS default, Linux icon theme). To use a custom branded icon:

1. Create a 16×16 or 32×32 PNG/ICO
2. On Windows: embed via `.rc` resource file and `build.rs`, use `IconSource::Resource("icon-name")`
3. On macOS/Linux: use `IconSource::Data { width, height, data }` with raw RGBA bytes

---

## References

- [tray-item crate](https://crates.io/crates/tray-item)
- [tray-icon crate](https://crates.io/crates/tray-icon) (alternative, heavier)
- `#![windows_subsystem = "windows"]` — [Rust conditional compilation](https://doc.rust-lang.org/reference/conditional-compilation.html#windows_subsystem)
