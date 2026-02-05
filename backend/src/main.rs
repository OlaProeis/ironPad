use std::net::SocketAddr;
use std::path::Path;
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

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::fmt().init();

    // Resolve data directory (production vs development mode)
    config::init_data_dir();

    // Find port and bind (listener kept alive to avoid race condition)
    let (listener, port) = find_available_port().await;

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
    let static_dir = Path::new("static");
    let has_frontend = static_dir.join("index.html").exists();

    if has_frontend {
        // Production mode: serve frontend from static/ and use SPA fallback
        info!("Production mode: serving frontend from static/");
        let serve_dir = ServeDir::new("static")
            .fallback(tower_http::services::ServeFile::new("static/index.html"));
        app = app.fallback_service(serve_dir);
    } else {
        // Development mode: API-only
        app = app.fallback(|| async {
            "Ironpad API server running. Use 'npm run dev' in frontend/ for the GUI."
        });
    }

    // Start server
    info!("🚀 Ironpad running on http://localhost:{port}");

    // Auto-open browser in production mode
    if has_frontend {
        let url = format!("http://localhost:{}", port);
        tokio::spawn(async move {
            // Small delay to ensure server is ready
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            if let Err(e) = webbrowser::open(&url) {
                tracing::warn!(
                    "Failed to open browser: {}. Open http://localhost:{} manually.",
                    e,
                    port
                );
            }
        });
    }

    axum::serve(listener, app).await.expect("Server failed");
}
