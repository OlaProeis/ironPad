use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::services::backlinks;

/// Response containing backlinks for a note
#[derive(Debug, Serialize)]
pub struct BacklinksResponse {
    pub note_id: String,
    pub backlinks: Vec<backlinks::Backlink>,
    pub forward_links: Vec<backlinks::ForwardLink>,
}

/// Response containing all note titles for autocompletion
#[derive(Debug, Serialize)]
pub struct NoteTitlesResponse {
    pub notes: Vec<NoteTitleEntry>,
}

#[derive(Debug, Serialize)]
pub struct NoteTitleEntry {
    pub id: String,
    pub title: String,
}

/// Query params for note search
#[derive(Debug, Deserialize)]
pub struct SearchNotesQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

/// Response for note search
#[derive(Debug, Serialize)]
pub struct NoteSearchResponse {
    pub query: String,
    pub results: Vec<NoteTitleEntry>,
}

pub fn router() -> Router {
    Router::new()
        .route("/notes/{id}/links", get(get_note_links))
        .route("/notes/{id}/backlinks", get(get_note_backlinks))
        .route("/notes/{id}/forward-links", get(get_note_forward_links))
        .route("/notes/titles", get(get_all_note_titles))
        .route("/notes/search", get(search_notes_handler))
        .route("/links/rebuild", post(rebuild_index))
        .route("/links/stats", get(get_stats))
}

/// Get both backlinks and forward links for a note
async fn get_note_links(Path(note_id): Path<String>) -> impl IntoResponse {
    let backlinks = backlinks::get_backlinks(&note_id);
    let forward_links = backlinks::get_forward_links(&note_id);

    Json(BacklinksResponse {
        note_id,
        backlinks,
        forward_links,
    })
    .into_response()
}

/// Get only backlinks for a note
async fn get_note_backlinks(Path(note_id): Path<String>) -> impl IntoResponse {
    let backlinks = backlinks::get_backlinks(&note_id);

    Json(serde_json::json!({
        "note_id": note_id,
        "backlinks": backlinks,
        "count": backlinks.len()
    }))
    .into_response()
}

/// Get only forward links for a note
async fn get_note_forward_links(Path(note_id): Path<String>) -> impl IntoResponse {
    let forward_links = backlinks::get_forward_links(&note_id);

    Json(serde_json::json!({
        "note_id": note_id,
        "forward_links": forward_links,
        "count": forward_links.len()
    }))
    .into_response()
}

/// Get all note titles for link autocompletion
async fn get_all_note_titles() -> impl IntoResponse {
    let titles = backlinks::get_all_note_titles();

    let entries: Vec<NoteTitleEntry> = titles
        .into_iter()
        .map(|(id, title)| NoteTitleEntry { id, title })
        .collect();

    Json(NoteTitlesResponse { notes: entries }).into_response()
}

/// Search notes by partial match for autocompletion
async fn search_notes_handler(Query(query): Query<SearchNotesQuery>) -> impl IntoResponse {
    let results = backlinks::search_note_titles(&query.q, query.limit);

    let entries: Vec<NoteTitleEntry> = results
        .into_iter()
        .map(|(id, title)| NoteTitleEntry { id, title })
        .collect();

    Json(NoteSearchResponse {
        query: query.q,
        results: entries,
    })
    .into_response()
}

/// Rebuild the link index manually
async fn rebuild_index() -> impl IntoResponse {
    match backlinks::rebuild_link_index() {
        Ok(count) => Json(serde_json::json!({
            "success": true,
            "indexed_notes": count,
            "message": format!("Indexed {} notes", count)
        }))
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": err
            }))
        )
            .into_response(),
    }
}

/// Get link index statistics
async fn get_stats() -> impl IntoResponse {
    let (total_links, unique_targets) = backlinks::get_link_stats();

    Json(serde_json::json!({
        "total_links": total_links,
        "unique_targets": unique_targets,
    }))
    .into_response()
}
