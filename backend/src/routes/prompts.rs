use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

use crate::services::prompts::{
    self, PromptCreateRequest, PromptListQuery, PromptSearchQuery, PromptUpdateRequest,
};

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_prompts).post(create_prompt))
        .route("/folders", get(list_folders))
        .route("/search/semantic", get(semantic_search))
        .route("/{id}", get(get_prompt).put(update_prompt).delete(delete_prompt))
}

#[derive(Debug, Deserialize)]
pub struct FolderQuery {
    scope: Option<String>,
    project_id: Option<String>,
}

async fn list_prompts(Query(query): Query<PromptListQuery>) -> impl IntoResponse {
    match prompts::list_prompts(&query) {
        Ok(items) => Json(items).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list prompts: {}", err),
        )
            .into_response(),
    }
}

async fn get_prompt(Path(id): Path<String>) -> impl IntoResponse {
    match prompts::get_prompt(&id) {
        Ok(prompt) => Json(prompt).into_response(),
        Err(err) if err.starts_with("Prompt not found") => (StatusCode::NOT_FOUND, err).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read prompt: {}", err),
        )
            .into_response(),
    }
}

async fn create_prompt(Json(payload): Json<PromptCreateRequest>) -> impl IntoResponse {
    match prompts::create_prompt(payload) {
        Ok(prompt) => (StatusCode::CREATED, Json(prompt)).into_response(),
        Err(err) if err.contains("required") || err.contains("not found") => {
            (StatusCode::BAD_REQUEST, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create prompt: {}", err),
        )
            .into_response(),
    }
}

async fn update_prompt(Path(id): Path<String>, Json(payload): Json<PromptUpdateRequest>) -> impl IntoResponse {
    match prompts::update_prompt(&id, payload) {
        Ok(prompt) => Json(prompt).into_response(),
        Err(err) if err.starts_with("Prompt not found") => (StatusCode::NOT_FOUND, err).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update prompt: {}", err),
        )
            .into_response(),
    }
}

async fn delete_prompt(Path(id): Path<String>) -> impl IntoResponse {
    match prompts::delete_prompt(&id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) if err.starts_with("Prompt not found") => (StatusCode::NOT_FOUND, err).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete prompt: {}", err),
        )
            .into_response(),
    }
}

async fn list_folders(Query(query): Query<FolderQuery>) -> impl IntoResponse {
    match prompts::list_folders(query.scope.as_deref(), query.project_id.as_deref()) {
        Ok(folders) => Json(folders).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list folders: {}", err),
        )
            .into_response(),
    }
}

async fn semantic_search(Query(query): Query<PromptSearchQuery>) -> impl IntoResponse {
    if query.q.trim().is_empty() {
        return Json(Vec::<crate::models::prompt::PromptSearchResult>::new()).into_response();
    }
    match prompts::semantic_search(&query) {
        Ok(results) => Json(results).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Prompt semantic search failed: {}", err),
        )
            .into_response(),
    }
}
