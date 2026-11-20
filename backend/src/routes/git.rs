use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::services::git;
use crate::services::git_errors;

pub fn router() -> Router {
    Router::new()
        .route("/status", get(get_status))
        .route("/commit", post(commit))
        .route("/init", post(init_repo))
        .route("/conflicts", get(get_conflicts))
        .route("/push", post(push))
        .route("/log", get(get_log))
        .route("/log/file", get(get_log_for_file))
        .route("/diff", get(get_working_diff))
        .route("/diff/{commit_id}", get(get_commit_diff))
        .route("/remote", get(get_remote).post(set_remote))
        .route("/fetch", post(fetch))
        .route("/pull", post(pull))
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
                message: git_errors::MSG_NO_REMOTE.to_string(),
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

#[derive(Debug, Deserialize)]
pub struct FileLogQuery {
    path: String,
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

async fn get_log_for_file(Query(query): Query<FileLogQuery>) -> impl IntoResponse {
    match git::get_log_for_path(&query.path, query.limit) {
        Ok(commits) => Json(commits).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get file git log: {}", err),
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

#[derive(Debug, Deserialize)]
struct SetRemoteRequest {
    url: String,
}

#[derive(Debug, Serialize)]
struct SetRemoteResponse {
    success: bool,
    message: String,
    url: String,
    replaced: bool,
    fetch_error: Option<String>,
}

async fn set_remote(Json(payload): Json<SetRemoteRequest>) -> impl IntoResponse {
    match git::set_remote_origin(&payload.url) {
        Ok(outcome) => {
            let message = if outcome.replaced {
                "Remote origin URL was replaced.".to_string()
            } else {
                "Remote origin was added.".to_string()
            };
            (
                StatusCode::OK,
                Json(SetRemoteResponse {
                    success: true,
                    message,
                    url: outcome.url,
                    replaced: outcome.replaced,
                    fetch_error: outcome.fetch_error,
                }),
            )
                .into_response()
        }
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(SetRemoteResponse {
                success: false,
                message: err,
                url: String::new(),
                replaced: false,
                fetch_error: None,
            }),
        )
            .into_response(),
    }
}

#[derive(Debug, Serialize)]
struct FetchResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct PullResponse {
    success: bool,
    message: String,
    commits: usize,
}

async fn pull() -> impl IntoResponse {
    match git::pull_from_remote() {
        Ok(outcome) => (
            StatusCode::OK,
            Json(PullResponse {
                success: true,
                message: outcome.message,
                commits: outcome.commits,
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(PullResponse {
                success: false,
                message: err,
                commits: 0,
            }),
        )
            .into_response(),
    }
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
