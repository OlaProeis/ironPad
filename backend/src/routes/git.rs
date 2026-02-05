use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::services::git;

pub fn router() -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/commit", post(commit))
        .route("/init", post(init_repo))
        .route("/conflicts", get(get_conflicts))
        .route("/push", post(push))
        .route("/log", get(get_log))
        .route("/diff", get(get_working_diff))
        .route("/diff/{commit_id}", get(get_commit_diff))
        .route("/remote", get(get_remote))
        .route("/fetch", post(fetch))
}

async fn get_status() -> impl IntoResponse {
    match git::get_status() {
        Ok(status) => Json(status).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get git status: {}", err),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CommitRequest {
    message: Option<String>,
}

async fn commit(Json(payload): Json<CommitRequest>) -> impl IntoResponse {
    match git::commit_all(payload.message.as_deref()) {
        Ok(info) => (StatusCode::CREATED, Json(info)).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err).into_response(),
    }
}

async fn init_repo() -> impl IntoResponse {
    match git::init_repo() {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to init repo: {}", err),
        )
            .into_response(),
    }
}

async fn get_conflicts() -> impl IntoResponse {
    match git::check_conflicts() {
        Ok(conflicts) => Json(conflicts).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to check conflicts: {}", err),
        )
            .into_response(),
    }
}

#[derive(Debug, Serialize)]
struct PushResponse {
    success: bool,
    message: String,
}

async fn push() -> impl IntoResponse {
    // Check if remote is configured
    if !git::has_remote() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PushResponse {
                success: false,
                message: "No remote repository configured. Add a remote with: git remote add origin <url>".to_string(),
            }),
        )
            .into_response();
    }

    match git::push_to_remote() {
        Ok(()) => (
            StatusCode::OK,
            Json(PushResponse {
                success: true,
                message: "Successfully pushed to remote".to_string(),
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PushResponse {
                success: false,
                message: err,
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    limit: Option<usize>,
}

async fn get_log(Query(query): Query<LogQuery>) -> impl IntoResponse {
    match git::get_log(query.limit) {
        Ok(commits) => Json(commits).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get git log: {}", err),
        )
            .into_response(),
    }
}

async fn get_working_diff() -> impl IntoResponse {
    match git::get_working_diff() {
        Ok(diff) => Json(diff).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get diff: {}", err),
        )
            .into_response(),
    }
}

async fn get_commit_diff(Path(commit_id): Path<String>) -> impl IntoResponse {
    match git::get_commit_diff(&commit_id) {
        Ok(diff) => Json(diff).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get commit diff: {}", err),
        )
            .into_response(),
    }
}

async fn get_remote() -> impl IntoResponse {
    match git::get_remote_info() {
        Ok(info) => Json(info).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get remote info: {}", err),
        )
            .into_response(),
    }
}

#[derive(Debug, Serialize)]
struct FetchResponse {
    success: bool,
    message: String,
}

async fn fetch() -> impl IntoResponse {
    match git::fetch_from_remote() {
        Ok(()) => (
            StatusCode::OK,
            Json(FetchResponse {
                success: true,
                message: "Successfully fetched from remote".to_string(),
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(FetchResponse {
                success: false,
                message: err,
            }),
        )
            .into_response(),
    }
}
