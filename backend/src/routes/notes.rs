use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};

use crate::models::note::{Note, NoteSummary};
use crate::services::filesystem;

pub fn router() -> Router {
    Router::new().route("/{id}", get(get_note).put(update_note).delete(delete_note))
}

pub async fn list_notes() -> impl IntoResponse {
    match filesystem::list_notes() {
        Ok(notes) => Json::<Vec<NoteSummary>>(notes).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list notes: {}", err),
        )
            .into_response(),
    }
}

async fn get_note(Path(id): Path<String>) -> impl IntoResponse {
    match filesystem::read_note_by_id(&id) {
        Ok(note) => Json::<Note>(note).into_response(),
        Err(err) if err.starts_with("Note not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read note: {}", err),
        )
            .into_response(),
    }
}

pub async fn create_note() -> impl IntoResponse {
    match filesystem::create_note() {
        Ok(note) => (StatusCode::CREATED, Json::<Note>(note)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create note: {}", err),
        )
            .into_response(),
    }
}

async fn update_note(Path(id): Path<String>, body: String) -> impl IntoResponse {
    match filesystem::update_note(&id, &body) {
        Ok(note) => Json::<Note>(note).into_response(),
        Err(err) if err.starts_with("Note not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update note: {}", err),
        )
            .into_response(),
    }
}

async fn delete_note(Path(id): Path<String>) -> impl IntoResponse {
    match filesystem::archive_note(&id) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) if err.starts_with("Note not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to archive note: {}", err),
        )
            .into_response(),
    }
}
