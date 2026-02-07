# System Tray Implementation (v0.2.0)

**Goal:** Replace the CMD window with a system tray icon. Users interact via tray menu: "Open in Browser" or "Quit". No console window on Windows.

---

## Overview

- **Scope:** Single codebase, cross-platform. CI/CD unchanged—same build pipeline produces one binary per OS.
- **Complexity:** Low–medium. Uses a cross-platform Rust crate; platform-specific code is minimal.

---

## Implementation Steps

### 1. Add Tray Crate Dependency

Add to `backend/Cargo.toml`:

```toml
# System tray (production mode)
tray-item = "0.10"
```

Alternative: `tray-icon` (more features, heavier; requires event loop integration).

### 2. Windows: Hide Console Window

Add near the top of `backend/src/main.rs` (after `mod` declarations if any, before `fn main`):

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

- **Debug builds:** Console remains (for logs).
- **Release builds:** No CMD window on Windows.

### 3. Remove Auto-Open Browser on Startup

In `main.rs`, remove or conditionally disable the auto-open logic (the `tokio::spawn` block that calls `webbrowser::open()`). The user will open the browser from the tray menu instead.

### 4. Add Tray Icon and Menu (Production Mode Only)

When `has_frontend` is true (production mode):

1. Create tray icon with an appropriate icon (or placeholder).
2. Add menu items:
   - **"Open in Browser"** — calls `webbrowser::open()` with `http://localhost:{port}`.
   - **"Quit"** — shuts down the server and exits the process.

### 5. Threading Considerations

- **macOS:** Some tray crates expect event handling on the main thread. May need to run tray logic on main thread and spawn the Axum server on a background thread, or use crate-specific patterns.
- **Windows/Linux:** Usually more flexible; verify with the chosen crate’s docs.

### 6. CI/CD Changes (If Needed)

Current `release.yml` builds for Windows, macOS, and Linux. Likely no changes required.

If using a crate that needs GTK on Linux (e.g. `tray-icon`), add to the "Install system dependencies (Linux)" step:

```yaml
sudo apt-get install -y cmake libgtk-3-dev libappindicator3-dev
```

Note: Linux users would then need GTK installed at runtime. For `tray-item`, check whether it has different Linux deps.

---

## Behaviour Summary

| Before (v0.1.0)        | After (v0.2.0)                 |
|------------------------|--------------------------------|
| CMD window visible     | No console window (Windows)    |
| Browser opens on start | Browser opens via tray menu    |
| Quit via Ctrl+C        | Quit via tray menu             |

---

## Testing Checklist

- [ ] Windows: No CMD window when running release binary.
- [ ] Windows: Tray icon appears; "Open in Browser" opens correct URL.
- [ ] Windows: "Quit" exits cleanly.
- [ ] macOS: Tray icon in menu bar; menu works.
- [ ] Linux: Tray icon in system tray; menu works.
- [ ] Development mode (`cargo run`): Behaviour unchanged (no tray, API-only).

---

## Icon Asset

You’ll need a tray icon (e.g. 16×16 or 32×32 PNG). Options:

- Extract from existing branding/logo.
- Use a simple placeholder (e.g. filled circle) for initial implementation.
- Store in `backend/` or `backend/static/` and load at runtime.

---

## References

- [tray-item crate](https://crates.io/crates/tray-item)
- [tray-icon crate](https://crates.io/crates/tray-icon) (alternative)
- `#![windows_subsystem = "windows"]` — [Rust embed documentation](https://doc.rust-lang.org/reference/conditional-compilation.html#windows_subsystem)
