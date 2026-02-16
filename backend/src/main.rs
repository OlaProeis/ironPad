// Hide console window on Windows in release builds (production mode)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{info, warn};

pub mod config;
mod models;
mod routes;
mod services;
mod watcher;
mod websocket;

/// Find an available port and return the bound listener.
/// Avoids TOCTOU race by keeping the listener alive.
async fn find_available_port() -> (TcpListener, u16) {
    for port in 3000..=3010 {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        if let Ok(listener) = TcpListener::bind(addr).await {
            return (listener, port);
        }
    }
    panic!("No available ports in range 3000–3010");
}

fn main() {
    tracing_subscriber::fmt().init();
    config::init_data_dir();

    if config::is_production() {
        run_with_tray();
    } else {
        // Development mode: normal tokio runtime, no tray
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(run_server(None));
    }
}

/// Start the Axum server. In tray mode, sends the bound port through `port_tx`
/// before entering the serve loop.
async fn run_server(port_tx: Option<std::sync::mpsc::Sender<u16>>) {
    let (listener, port) = find_available_port().await;

    // Notify tray thread of the bound port
    if let Some(tx) = port_tx {
        let _ = tx.send(port);
    }

    // WebSocket state (shared across handlers)
    let ws_state = Arc::new(websocket::WsState::new());

    // Start file watcher
    let ws_state_clone = ws_state.clone();
    if let Err(e) = watcher::start_watcher(ws_state_clone).await {
        warn!("File watcher failed to start: {}", e);
    }

    // Initialize git repo if needed
    if let Err(e) = services::git::init_repo() {
        warn!("Git init skipped: {}", e);
    }

    // Start auto-commit background task (tries to commit every 60s)
    services::git::start_auto_commit();

    // CORS layer (permissive for local-only app)
    let cors = CorsLayer::permissive();

    // API router
    let api_router = Router::new()
        // Notes CRUD
        .route(
            "/notes",
            get(routes::notes::list_notes).post(routes::notes::create_note),
        )
        .nest("/notes", routes::notes::router())
        // Tasks
        .nest("/tasks", routes::tasks::router())
        // Search
        .nest("/search", routes::search::router())
        // Git
        .nest("/git", routes::git::router())
        // Projects
        .nest("/projects", routes::projects::router())
        // Daily notes
        .nest("/daily", routes::daily::router())
        // Assets
        .nest("/assets", routes::assets::router());

    // App router with WebSocket state
    let mut app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route(
            "/ws",
            get({
                let ws = ws_state.clone();
                move |upgrade: axum::extract::WebSocketUpgrade| {
                    websocket::ws_handler(upgrade, axum::extract::State(ws))
                }
            }),
        )
        .nest("/api", api_router)
        .layer(cors);

    // Check for embedded frontend (production mode)
    let has_frontend = config::is_production();

    if has_frontend {
        // Production mode: serve frontend from static/ next to the exe
        let static_path = config::exe_dir().join("static");
        let index_path = static_path.join("index.html");
        info!(
            "Production mode: serving frontend from {}",
            static_path.display()
        );
        let serve_dir =
            ServeDir::new(&static_path).fallback(tower_http::services::ServeFile::new(index_path));
        app = app.fallback_service(serve_dir);
    } else {
        // Development mode: API-only
        app = app.fallback(|| async {
            "Ironpad API server running. Use 'npm run dev' in frontend/ for the GUI."
        });
    }

    // Start server
    info!("Ironpad running on http://localhost:{port}");

    axum::serve(listener, app).await.expect("Server failed");
}

// ---------------------------------------------------------------------------
// System tray (production mode)
// ---------------------------------------------------------------------------

/// Build a platform-appropriate tray icon.
///
/// On Windows the Ironpad icon is embedded in the .exe via winresource (build.rs).
/// We load it with LoadIconW using the resource ID assigned by winresource.
#[cfg(target_os = "windows")]
fn tray_icon() -> tray_item::IconSource {
    let hicon = unsafe {
        // winresource embeds the icon at resource ID 1.
        // GetModuleHandleW(null) = current exe, MAKEINTRESOURCE(1) = 1 as PCWSTR.
        let hinstance =
            windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(std::ptr::null());
        windows_sys::Win32::UI::WindowsAndMessaging::LoadIconW(hinstance, 1 as *const u16)
    };
    tray_item::IconSource::RawIcon(hicon)
}

#[cfg(target_os = "macos")]
fn tray_icon() -> tray_item::IconSource {
    tray_item::IconSource::Resource("")
}

#[cfg(target_os = "linux")]
fn tray_icon() -> tray_item::IconSource {
    tray_item::IconSource::Resource("application-x-executable")
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn tray_icon() -> tray_item::IconSource {
    tray_item::IconSource::Resource("")
}

/// Production mode: run the server on a background thread, tray on main thread.
/// The main thread drives the tray event loop (required on macOS; safe everywhere).
fn run_with_tray() {
    use std::sync::mpsc;

    enum TrayMessage {
        OpenBrowser,
        Quit,
    }

    // Channel to receive the dynamically-bound port from the server thread
    let (port_tx, port_rx) = mpsc::channel::<u16>();

    // Start the Axum server on a background thread with its own tokio runtime
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(run_server(Some(port_tx)));
    });

    // Wait for the server to report its port
    let port = port_rx.recv().expect("Server failed to start");
    let url = format!("http://localhost:{}", port);

    // Auto-open browser after a short delay (non-blocking)
    let url_for_open = url.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        let _ = webbrowser::open(&url_for_open);
    });

    // Set up system tray icon and menu
    let (tx, rx) = mpsc::sync_channel::<TrayMessage>(2);

    let mut tray = match tray_item::TrayItem::new("Ironpad", tray_icon()) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create system tray: {}. Running headless.", e);
            // Keep the process alive so the server thread continues
            loop {
                std::thread::park();
            }
        }
    };

    let tx_open = tx.clone();
    let _ = tray.add_menu_item("Open in Browser", move || {
        let _ = tx_open.send(TrayMessage::OpenBrowser);
    });

    let tx_quit = tx;
    let _ = tray.add_menu_item("Quit", move || {
        let _ = tx_quit.send(TrayMessage::Quit);
    });

    // Main-thread event loop — processes tray menu actions
    for msg in rx {
        match msg {
            TrayMessage::OpenBrowser => {
                let _ = webbrowser::open(&url);
            }
            TrayMessage::Quit => {
                info!("Quit requested from system tray");
                std::process::exit(0);
            }
        }
    }
}
