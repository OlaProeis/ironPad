use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent};
use tokio::sync::mpsc;

use crate::config;
use crate::services::backlinks;
use crate::websocket::{WsMessage, WsState};

/// Start the file watcher in a background task
pub async fn start_watcher(ws_state: Arc<WsState>) -> Result<(), String> {
    let (tx, mut rx) = mpsc::channel::<Vec<DebouncedEvent>>(100);

    // Create debouncer with 500ms debounce time
    let debouncer = new_debouncer(
        Duration::from_millis(500),
        None,
        move |result: Result<Vec<DebouncedEvent>, Vec<notify::Error>>| {
            if let Ok(events) = result {
                let _ = tx.blocking_send(events);
            }
        },
    )
    .map_err(|e| format!("Failed to create file watcher: {}", e))?;

    // Watch the data directory
    let data_path = config::data_dir();
    if !data_path.exists() {
        return Err(format!(
            "Data directory does not exist: {}",
            data_path.display()
        ));
    }

    // We need to keep the debouncer alive, so we'll store it
    let debouncer = Arc::new(tokio::sync::Mutex::new(debouncer));

    {
        let mut d = debouncer.lock().await;
        d.watcher()
            .watch(data_path, RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch directory: {}", e))?;
    }

    tracing::info!("File watcher started for: {}", data_path.display());

    // Spawn task to process file events
    let ws_state_clone = ws_state.clone();
    tokio::spawn(async move {
        // Keep debouncer alive
        let _debouncer = debouncer;

        while let Some(events) = rx.recv().await {
            for event in events {
                process_event(&event, &ws_state_clone);
            }
        }
    });

    Ok(())
}

use std::collections::HashMap;
/// Track recent saves to avoid notifying about our own changes
use std::sync::Mutex;
use std::time::Instant;

lazy_static::lazy_static! {
    static ref RECENT_SAVES: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

/// Mark a file as recently saved by us (call this before saving)
pub fn mark_file_saved(path: &str) {
    if let Ok(mut saves) = RECENT_SAVES.lock() {
        saves.insert(path.to_string(), Instant::now());
    }
}

/// Process a single debounced file event
fn process_event(event: &DebouncedEvent, ws_state: &WsState) {
    use notify::EventKind;

    // Only process markdown files
    let paths: Vec<_> = event
        .paths
        .iter()
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e == "md")
                .unwrap_or(false)
        })
        .collect();

    if paths.is_empty() {
        return;
    }

    // Skip temporary files (used for atomic writes)
    if paths.iter().any(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('.') && n.ends_with(".tmp"))
            .unwrap_or(false)
    }) {
        return;
    }

    // Skip archive and .git directories
    if paths.iter().any(|p| {
        let s = p.to_string_lossy();
        s.contains("archive") || s.contains(".git")
    }) {
        return;
    }

    let path_str = normalize_path(&paths[0]);

    // Check if this was a recent save by us (within last 2 seconds)
    if let Ok(mut saves) = RECENT_SAVES.lock() {
        // Clean up old entries
        saves.retain(|_, t| t.elapsed().as_secs() < 5);

        if let Some(saved_at) = saves.get(&path_str) {
            if saved_at.elapsed().as_secs() < 2 {
                return; // Skip - this was our own save
            }
        }
    }

    let msg = match &event.kind {
        EventKind::Create(_) => {
            tracing::info!("External file created: {}", path_str);
            Some(WsMessage::FileCreated { path: path_str })
        }
        EventKind::Modify(_) => {
            tracing::info!("External file modified: {}", path_str);
            
            // Trigger backlink index rebuild for note files
            if is_note_file(&paths[0]) {
                tokio::task::spawn_blocking(|| {
                    if let Err(e) = backlinks::rebuild_link_index() {
                        tracing::warn!("Failed to rebuild backlink index: {}", e);
                    }
                });
            }
            
            Some(WsMessage::FileModified { path: path_str })
        }
        EventKind::Remove(_) => {
            tracing::info!("External file deleted: {}", path_str);
            
            // Trigger backlink index rebuild when notes are deleted
            if is_note_file(&paths[0]) {
                tokio::task::spawn_blocking(|| {
                    if let Err(e) = backlinks::rebuild_link_index() {
                        tracing::warn!("Failed to rebuild backlink index: {}", e);
                    }
                });
            }
            
            Some(WsMessage::FileDeleted { path: path_str })
        }
        _ => None,
    };

    if let Some(msg) = msg {
        ws_state.broadcast(msg);
    }
}

/// Check if a path is a note file (not tasks, prompts, etc.)
fn is_note_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Must be a markdown file
    if path.extension().and_then(|s| s.to_str()) != Some("md") {
        return false;
    }

    // Skip task files (contain "tasks/" in path)
    if path_str.contains("tasks") && !path_str.contains("notes") {
        return false;
    }

    // Skip prompt files (contain "prompts/" in path)
    if path_str.contains("prompts") {
        return false;
    }

    // Include notes folder files
    if path_str.contains("notes") && !path_str.contains("archive") {
        return true;
    }

    // Include project index.md files
    if path_str.contains("projects")
        && path.file_name().and_then(|s| s.to_str()) == Some("index.md")
    {
        return true;
    }

    // Include root-level files (index.md, inbox.md)
    if let Some(parent) = path.parent() {
        if parent == config::data_dir() {
            return true;
        }
    }

    // Include daily notes
    if path_str.contains("daily") {
        return true;
    }

    false
}

/// Normalize path for client consumption
fn normalize_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    // Find "data" in the path and strip everything before and including it
    if let Some(idx) = path_str.find("data") {
        let stripped = &path_str[idx + 5..]; // Skip "data" + separator
        return stripped
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
    }

    path_str.replace('\\', "/")
}
