use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;

use crate::config;
use crate::routes::tasks::create_task_impl;
use crate::services::filesystem;
use crate::services::frontmatter;
use crate::services::github::auth;
use crate::services::github::lists;
use crate::services::github::snapshot::{GithubIssue, GithubPull, Snapshot};
use crate::services::github::snapshot_io::{
    self, delete_caches_for_closed_items, delete_snapshot_dir, read_snapshot, write_snapshot,
    ItemKind,
};
use crate::services::github::{GithubClient, GithubError};

pub const GITHUB_REPO_VALIDATION_MSG: &str = "Use owner/name, with no URL and no .git suffix.";

static GITHUB_REPO_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$").expect("repo regex"));

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubSnapshotResponse {
    pub repo: Option<String>,
    pub login: Option<String>,
    pub refreshed_at: Option<String>,
    pub stale: bool,
    pub issues: Vec<GithubIssue>,
    pub pulls: Vec<GithubPull>,
    pub issues_truncated: bool,
    pub pulls_truncated: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PutGithubRepoRequest {
    pub repo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PutGithubRepoResponse {
    pub repo: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GithubSummaryEntry {
    pub project_id: String,
    pub repo: String,
    pub assigned_issues: usize,
    pub review_pulls: usize,
    pub refreshed_at: Option<String>,
}

pub fn project_github_router() -> Router {
    Router::new()
        .route(
            "/{id}/github",
            get(get_project_github).put(put_project_github),
        )
        .route("/{id}/github/refresh", post(refresh_project_github))
        .route("/{id}/github/issues/{number}", get(get_issue_detail))
        .route("/{id}/github/pulls/{number}", get(get_pull_detail))
        .route("/{id}/github/issues/{number}/track", post(track_issue))
        .route("/{id}/github/pulls/{number}/track", post(track_pull))
}

fn project_index_path(id: &str) -> std::path::PathBuf {
    config::data_dir()
        .join("projects")
        .join(id)
        .join("index.md")
}

fn snapshot_is_stale(refreshed_at: &Option<String>) -> bool {
    let Some(raw) = refreshed_at else {
        return true;
    };
    let Ok(parsed) = DateTime::parse_from_rfc3339(raw) else {
        return true;
    };
    let utc = parsed.with_timezone(&Utc);
    Utc::now().signed_duration_since(utc) > Duration::minutes(15)
}

fn response_from_snapshot(snap: Option<Snapshot>) -> GithubSnapshotResponse {
    match snap {
        None => GithubSnapshotResponse {
            repo: None,
            login: None,
            refreshed_at: None,
            stale: true,
            issues: vec![],
            pulls: vec![],
            issues_truncated: false,
            pulls_truncated: false,
            error: None,
        },
        Some(s) => {
            let stale = snapshot_is_stale(&s.refreshed_at);
            GithubSnapshotResponse {
                repo: Some(s.repo),
                login: s.login,
                refreshed_at: s.refreshed_at,
                stale,
                issues: s.issues,
                pulls: s.pulls,
                issues_truncated: s.issues_truncated,
                pulls_truncated: s.pulls_truncated,
                error: None,
            }
        }
    }
}

pub fn validate_github_repo(raw: &str) -> Result<String, ()> {
    let trimmed = raw.trim();
    if trimmed.is_empty()
        || trimmed.contains("://")
        || trimmed.ends_with(".git")
        || trimmed.contains("..")
        || trimmed.matches('/').count() != 1
    {
        return Err(());
    }
    if !GITHUB_REPO_RE.is_match(trimmed) {
        return Err(());
    }
    Ok(trimmed.to_string())
}

fn login_list_contains(logins: &[String], login: &str) -> bool {
    logins
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(login))
}

fn summary_counts(snap: &Snapshot, login: &str) -> (usize, usize) {
    let assigned_issues = snap
        .issues
        .iter()
        .filter(|issue| login_list_contains(&issue.assignee_logins, login))
        .count();
    let review_pulls = snap
        .pulls
        .iter()
        .filter(|pull| login_list_contains(&pull.requested_reviewer_logins, login))
        .count();
    (assigned_issues, review_pulls)
}

fn project_ids_with_index() -> Vec<String> {
    let projects_dir = config::data_dir().join("projects");
    let Ok(entries) = fs::read_dir(&projects_dir) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if !path.join("index.md").is_file() {
            continue;
        }
        let Some(id) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        ids.push(id.to_string());
    }
    ids.sort();
    ids
}

pub fn github_summary_entries() -> Vec<GithubSummaryEntry> {
    let mut entries = Vec::new();
    for project_id in project_ids_with_index() {
        let Some(snap) = read_snapshot(&project_id) else {
            continue;
        };
        let Some(login) = snap.login.as_deref().filter(|value| !value.is_empty()) else {
            continue;
        };
        if snap.repo.trim().is_empty() {
            continue;
        }
        let (assigned_issues, review_pulls) = summary_counts(&snap, login);
        entries.push(GithubSummaryEntry {
            project_id,
            repo: snap.repo.clone(),
            assigned_issues,
            review_pulls,
            refreshed_at: snap.refreshed_at,
        });
    }
    entries
}

pub async fn github_summary() -> impl IntoResponse {
    Json(github_summary_entries())
}

async fn get_project_github(Path(id): Path<String>) -> impl IntoResponse {
    let snap = read_snapshot(&id);
    Json(response_from_snapshot(snap)).into_response()
}

async fn put_project_github(
    Path(id): Path<String>,
    Json(payload): Json<PutGithubRepoRequest>,
) -> impl IntoResponse {
    let index_path = project_index_path(&id);
    if !index_path.exists() {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    let existing = match fs::read_to_string(&index_path) {
        Ok(content) => content,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read project: {}", err),
            )
                .into_response();
        }
    };

    let (mut fm, body, _) = frontmatter::parse_frontmatter(&existing);

    match payload.repo {
        None => {
            if let Err(err) = delete_snapshot_dir(&id) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to remove snapshot: {}", err),
                )
                    .into_response();
            }
            fm.remove(serde_yaml::Value::from("github_repo"));
            let new_content = match frontmatter::serialize_frontmatter(&fm, &body) {
                Ok(c) => c,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to serialize: {}", err),
                    )
                        .into_response();
                }
            };
            if let Err(err) = filesystem::atomic_write(&index_path, new_content.as_bytes()) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write file: {}", err),
                )
                    .into_response();
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Some(repo_raw) => {
            let validated = match validate_github_repo(&repo_raw) {
                Ok(v) => v,
                Err(()) => {
                    return (StatusCode::BAD_REQUEST, GITHUB_REPO_VALIDATION_MSG).into_response();
                }
            };
            fm.insert(
                serde_yaml::Value::from("github_repo"),
                serde_yaml::Value::from(validated.as_str()),
            );
            let new_content = match frontmatter::serialize_frontmatter(&fm, &body) {
                Ok(c) => c,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to serialize: {}", err),
                    )
                        .into_response();
                }
            };
            if let Err(err) = filesystem::atomic_write(&index_path, new_content.as_bytes()) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write file: {}", err),
                )
                    .into_response();
            }
            (
                StatusCode::OK,
                Json(PutGithubRepoResponse { repo: validated }),
            )
                .into_response()
        }
    }
}

fn read_linked_repo(id: &str) -> Option<String> {
    let index_path = project_index_path(id);
    let existing = fs::read_to_string(&index_path).ok()?;
    let (fm, _, _) = frontmatter::parse_frontmatter(&existing);
    let repo = fm
        .get(serde_yaml::Value::from("github_repo"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())?;
    Some(repo.to_string())
}

fn split_owner_repo(repo: &str) -> Option<(String, String)> {
    let (owner, name) = repo.split_once('/')?;
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return None;
    }
    Some((owner.to_string(), name.to_string()))
}

fn numbers_of_issues(issues: &[GithubIssue]) -> Vec<u64> {
    issues.iter().map(|issue| issue.number).collect()
}

fn numbers_of_pulls(pulls: &[GithubPull]) -> Vec<u64> {
    pulls.iter().map(|pull| pull.number).collect()
}

async fn refresh_project_github(Path(id): Path<String>) -> impl IntoResponse {
    let Some(repo) = read_linked_repo(&id) else {
        return (StatusCode::BAD_REQUEST, "No repository linked").into_response();
    };
    let Some((owner, name)) = split_owner_repo(&repo) else {
        return (StatusCode::BAD_REQUEST, "No repository linked").into_response();
    };

    let token = match auth::resolve_token() {
        Ok(token) => token,
        Err(message) => return (StatusCode::BAD_REQUEST, message).into_response(),
    };

    let client = GithubClient::new(&token);
    let existing = read_snapshot(&id);
    let login = lists::fetch_login(&client).await.ok();
    let issue_etag = existing.as_ref().and_then(|snap| snap.issue_etag.clone());
    let pull_etag = existing.as_ref().and_then(|snap| snap.pull_etag.clone());

    let fetched = match lists::fetch_lists(
        &client,
        &owner,
        &name,
        issue_etag.as_deref(),
        pull_etag.as_deref(),
    )
    .await
    {
        Ok(fetched) => fetched,
        Err(err) => {
            if let Some(snap) = existing {
                let mut body = response_from_snapshot(Some(snap));
                body.error = Some(err.user_message());
                return Json(body).into_response();
            }
            return (StatusCode::SERVICE_UNAVAILABLE, err.user_message()).into_response();
        }
    };

    if fetched.issues_304 && fetched.pulls_304 {
        let mut snap = existing.unwrap_or_else(|| Snapshot {
            repo: repo.clone(),
            login: login.clone(),
            refreshed_at: None,
            issue_etag: fetched.issue_etag.clone(),
            pull_etag: fetched.pull_etag.clone(),
            issues: Vec::new(),
            pulls: Vec::new(),
            issues_truncated: false,
            pulls_truncated: false,
        });
        snap.refreshed_at = Some(Utc::now().to_rfc3339());
        if let Err(err) = write_snapshot(&id, &snap) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write snapshot: {err}"),
            )
                .into_response();
        }
        return Json(response_from_snapshot(Some(snap))).into_response();
    }

    let issues = if fetched.issues_304 {
        existing
            .as_ref()
            .map(|snap| snap.issues.clone())
            .unwrap_or_default()
    } else {
        fetched.issues
    };
    let pulls = if fetched.pulls_304 {
        existing
            .as_ref()
            .map(|snap| snap.pulls.clone())
            .unwrap_or_default()
    } else {
        fetched.pulls
    };
    let issues_truncated = if fetched.issues_304 {
        existing
            .as_ref()
            .map(|snap| snap.issues_truncated)
            .unwrap_or(false)
    } else {
        fetched.issues_truncated
    };
    let pulls_truncated = if fetched.pulls_304 {
        existing
            .as_ref()
            .map(|snap| snap.pulls_truncated)
            .unwrap_or(false)
    } else {
        fetched.pulls_truncated
    };

    let snap = Snapshot {
        repo,
        login,
        refreshed_at: Some(Utc::now().to_rfc3339()),
        issue_etag: fetched.issue_etag,
        pull_etag: fetched.pull_etag,
        issues,
        pulls,
        issues_truncated,
        pulls_truncated,
    };

    let issue_numbers = numbers_of_issues(&snap.issues);
    let pull_numbers = numbers_of_pulls(&snap.pulls);
    delete_caches_for_closed_items(&id, &issue_numbers, &pull_numbers);

    if let Err(err) = write_snapshot(&id, &snap) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write snapshot: {err}"),
        )
            .into_response();
    }
    Json(response_from_snapshot(Some(snap))).into_response()
}

const NOT_IN_OPEN_SNAPSHOT: &str = "That item is not in the open snapshot.";
const MAX_BODY_BYTES: usize = 1_048_576;
const MAX_COMMENTS: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubComment {
    pub author_login: String,
    pub created_at: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubItemDetail {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub body: Option<String>,
    pub body_too_large: bool,
    pub comments: Vec<GithubComment>,
    pub more_comments: bool,
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_status: Option<String>,
}

enum DetailKind {
    Issue,
    Pull,
}

struct OpenItem {
    title: String,
    html_url: String,
    head_sha: Option<String>,
    kind: ItemKind,
}

fn find_open_item(snap: &Snapshot, number: u64, kind: DetailKind) -> Option<OpenItem> {
    match kind {
        DetailKind::Issue => snap
            .issues
            .iter()
            .find(|issue| issue.number == number)
            .map(|issue| OpenItem {
                title: issue.title.clone(),
                html_url: issue.html_url.clone(),
                head_sha: None,
                kind: ItemKind::Issue,
            }),
        DetailKind::Pull => snap
            .pulls
            .iter()
            .find(|pull| pull.number == number)
            .map(|pull| OpenItem {
                title: pull.title.clone(),
                html_url: pull.html_url.clone(),
                head_sha: Some(pull.head_sha.clone()),
                kind: ItemKind::Pull,
            }),
    }
}

fn comments_from_payload(value: &serde_json::Value) -> Result<(Vec<GithubComment>, bool), String> {
    let Some(items) = value.as_array() else {
        return Err("GitHub comments response was not a list".to_string());
    };
    let mut comments = Vec::with_capacity(items.len());
    for item in items {
        comments.push(GithubComment {
            author_login: item
                .get("user")
                .and_then(|user| user.get("login"))
                .and_then(|login| login.as_str())
                .unwrap_or("")
                .to_string(),
            created_at: item
                .get("created_at")
                .and_then(|created| created.as_str())
                .unwrap_or("")
                .to_string(),
            body: item
                .get("body")
                .and_then(|body| body.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    comments.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    let more_comments = comments.len() > MAX_COMMENTS;
    comments.truncate(MAX_COMMENTS);
    Ok((comments, more_comments))
}

fn cached_to_comments(cached: Vec<snapshot_io::CachedComment>) -> Vec<GithubComment> {
    cached
        .into_iter()
        .map(|comment| GithubComment {
            author_login: comment.author_login,
            created_at: comment.created_at,
            body: comment.body,
        })
        .collect()
}

fn comments_to_cached(comments: &[GithubComment]) -> Vec<snapshot_io::CachedComment> {
    comments
        .iter()
        .map(|comment| snapshot_io::CachedComment {
            author_login: comment.author_login.clone(),
            created_at: comment.created_at.clone(),
            body: comment.body.clone(),
        })
        .collect()
}

async fn get_issue_detail(Path((id, number)): Path<(String, u64)>) -> impl IntoResponse {
    fetch_item_detail(&id, number, DetailKind::Issue).await
}

async fn get_pull_detail(Path((id, number)): Path<(String, u64)>) -> impl IntoResponse {
    fetch_item_detail(&id, number, DetailKind::Pull).await
}

async fn fetch_item_detail(id: &str, number: u64, kind: DetailKind) -> axum::response::Response {
    let Some(snap) = read_snapshot(id) else {
        return (StatusCode::NOT_FOUND, NOT_IN_OPEN_SNAPSHOT).into_response();
    };
    let Some(item) = find_open_item(&snap, number, kind) else {
        return (StatusCode::NOT_FOUND, NOT_IN_OPEN_SNAPSHOT).into_response();
    };
    let include_status = item.head_sha.is_some();
    let Some((owner, repo)) = split_owner_repo(&snap.repo) else {
        return Json(GithubItemDetail {
            number,
            title: item.title,
            html_url: item.html_url,
            body: snapshot_io::read_body(id, item.kind, number),
            body_too_large: false,
            comments: cached_to_comments(
                snapshot_io::read_comments(id, item.kind, number).unwrap_or_default(),
            ),
            more_comments: false,
            error: Some("No repository linked".to_string()),
            commit_status: None,
        })
        .into_response();
    };

    let cached_body = snapshot_io::read_body(id, item.kind, number);
    let cached_comments = snapshot_io::read_comments(id, item.kind, number).unwrap_or_default();

    let token = match auth::resolve_token() {
        Ok(token) => token,
        Err(message) => {
            return Json(detail_from_cache(
                number,
                &item,
                cached_body,
                cached_comments,
                Some(message),
                None,
            ))
            .into_response();
        }
    };
    let client = GithubClient::new(&token);

    let mut error: Option<String> = None;
    let mut body = cached_body.clone();
    let mut body_too_large = false;

    if cached_body.is_none() {
        let body_path = match item.kind {
            ItemKind::Issue => format!("repos/{owner}/{repo}/issues/{number}"),
            ItemKind::Pull => format!("repos/{owner}/{repo}/pulls/{number}"),
        };
        match client
            .get_json::<serde_json::Value>(&lists::github_api_url(&body_path), None)
            .await
        {
            Ok((payload, _)) => match payload.get("body") {
                Some(value) if value.is_null() => {
                    body = Some(String::new());
                    let _ = snapshot_io::write_body(id, item.kind, number, "");
                }
                Some(value) => match value.as_str() {
                    Some(text) if text.len() > MAX_BODY_BYTES => {
                        body = None;
                        body_too_large = true;
                        snapshot_io::delete_body(id, item.kind, number);
                    }
                    Some(text) => {
                        body = Some(text.to_string());
                        let _ = snapshot_io::write_body(id, item.kind, number, text);
                    }
                    None => {
                        error = Some(
                            GithubError::Network("issue body was not text".to_string())
                                .user_message(),
                        );
                    }
                },
                None => {
                    body = Some(String::new());
                    let _ = snapshot_io::write_body(id, item.kind, number, "");
                }
            },
            Err(err) => {
                error = Some(err.user_message());
                body = cached_body.clone();
            }
        }
    }

    let mut comments = cached_to_comments(cached_comments.clone());
    let mut more_comments = false;
    let comments_path = format!("repos/{owner}/{repo}/issues/{number}/comments?per_page=100");
    match client
        .get_json::<serde_json::Value>(&lists::github_api_url(&comments_path), None)
        .await
    {
        Ok((payload, _)) => match comments_from_payload(&payload) {
            Ok((fresh, more)) => {
                let _ =
                    snapshot_io::write_comments(id, item.kind, number, &comments_to_cached(&fresh));
                comments = fresh;
                more_comments = more;
            }
            Err(message) => {
                if error.is_none() {
                    error = Some(GithubError::Network(message).user_message());
                }
            }
        },
        Err(err) => {
            if error.is_none() {
                error = Some(err.user_message());
            }
        }
    }

    let mut commit_status = None;
    if include_status {
        if let Some(sha) = item.head_sha.as_deref().filter(|sha| !sha.is_empty()) {
            let status_path = format!("repos/{owner}/{repo}/commits/{sha}/status");
            if let Ok((payload, _)) = client
                .get_json::<serde_json::Value>(&lists::github_api_url(&status_path), None)
                .await
            {
                commit_status = payload
                    .get("state")
                    .and_then(|state| state.as_str())
                    .map(str::to_string);
            }
        }
    }

    Json(GithubItemDetail {
        number,
        title: item.title,
        html_url: item.html_url,
        body,
        body_too_large,
        comments,
        more_comments,
        error,
        commit_status,
    })
    .into_response()
}

fn detail_from_cache(
    number: u64,
    item: &OpenItem,
    body: Option<String>,
    comments: Vec<snapshot_io::CachedComment>,
    error: Option<String>,
    commit_status: Option<String>,
) -> GithubItemDetail {
    GithubItemDetail {
        number,
        title: item.title.clone(),
        html_url: item.html_url.clone(),
        body,
        body_too_large: false,
        comments: cached_to_comments(comments),
        more_comments: false,
        error,
        commit_status,
    }
}

#[derive(Debug, Clone)]
struct TrackSource {
    title: String,
    html_url: String,
    labels: Vec<String>,
    milestone_due: Option<String>,
    kind: ItemKind,
    number: u64,
}

fn find_track_source(snap: &Snapshot, number: u64, detail_kind: DetailKind) -> Option<TrackSource> {
    match detail_kind {
        DetailKind::Issue => snap
            .issues
            .iter()
            .find(|issue| issue.number == number)
            .map(|issue| TrackSource {
                title: issue.title.clone(),
                html_url: issue.html_url.clone(),
                labels: issue
                    .labels
                    .iter()
                    .map(|label| label.name.clone())
                    .collect(),
                milestone_due: issue.milestone_due.clone(),
                kind: ItemKind::Issue,
                number,
            }),
        DetailKind::Pull => snap
            .pulls
            .iter()
            .find(|pull| pull.number == number)
            .map(|pull| TrackSource {
                title: pull.title.clone(),
                html_url: pull.html_url.clone(),
                labels: pull.labels.iter().map(|label| label.name.clone()).collect(),
                milestone_due: pull.milestone_due.clone(),
                kind: ItemKind::Pull,
                number,
            }),
    }
}

fn find_task_id_by_github_url(project_id: &str, html_url: &str) -> Option<String> {
    let tasks_dir = config::data_dir()
        .join("projects")
        .join(project_id)
        .join("tasks");
    let entries = fs::read_dir(&tasks_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let content = fs::read_to_string(&path).ok()?;
        let (fm, _, _) = frontmatter::parse_frontmatter(&content);
        let Some(url) = frontmatter::get_str(&fm, "github_url") else {
            continue;
        };
        if url == html_url {
            return frontmatter::get_str(&fm, "id");
        }
    }
    None
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackItemResponse {
    pub task_id: String,
    pub created: bool,
}

fn track_item(
    project_id: &str,
    number: u64,
    detail_kind: DetailKind,
) -> Result<(TrackItemResponse, StatusCode), (StatusCode, String)> {
    let Some(snap) = read_snapshot(project_id) else {
        return Err((StatusCode::NOT_FOUND, NOT_IN_OPEN_SNAPSHOT.to_string()));
    };
    let Some(source) = find_track_source(&snap, number, detail_kind) else {
        return Err((StatusCode::NOT_FOUND, NOT_IN_OPEN_SNAPSHOT.to_string()));
    };

    if let Some(task_id) = find_task_id_by_github_url(project_id, &source.html_url) {
        return Ok((
            TrackItemResponse {
                task_id,
                created: false,
            },
            StatusCode::OK,
        ));
    }

    let cached_body = snapshot_io::read_body(project_id, source.kind, number);
    let task_body = match cached_body.as_deref() {
        Some(text) if !text.is_empty() => format!("{text}\n\n{}", source.html_url),
        _ => source.html_url.clone(),
    };

    let created = create_task_impl(project_id, &source.title, Some("Active"), None)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err))?;

    let task_path = config::data_dir().join(&created.path);
    let existing_raw = fs::read_to_string(&task_path).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read task: {err}"),
        )
    })?;
    let (mut fm, _, _) = frontmatter::parse_frontmatter(&existing_raw);

    if !source.labels.is_empty() {
        let tag_values: Vec<serde_yaml::Value> = source
            .labels
            .iter()
            .map(|name| serde_yaml::Value::from(name.as_str()))
            .collect();
        fm.insert(
            serde_yaml::Value::from("tags"),
            serde_yaml::Value::Sequence(tag_values),
        );
    }
    if let Some(due) = &source.milestone_due {
        fm.insert(
            serde_yaml::Value::from("due_date"),
            serde_yaml::Value::from(due.as_str()),
        );
    }
    fm.insert(
        serde_yaml::Value::from("github_url"),
        serde_yaml::Value::from(source.html_url.as_str()),
    );
    fm.insert(
        serde_yaml::Value::from("github_number"),
        serde_yaml::Value::Number(source.number.into()),
    );

    let new_content = frontmatter::serialize_frontmatter(&fm, &task_body).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize task: {err}"),
        )
    })?;
    filesystem::atomic_write(&task_path, new_content.as_bytes()).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write task: {err}"),
        )
    })?;

    Ok((
        TrackItemResponse {
            task_id: created.id,
            created: true,
        },
        StatusCode::CREATED,
    ))
}

async fn track_issue(Path((id, number)): Path<(String, u64)>) -> impl IntoResponse {
    match track_item(&id, number, DetailKind::Issue) {
        Ok((body, status)) => (status, Json(body)).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

async fn track_pull(Path((id, number)): Path<(String, u64)>) -> impl IntoResponse {
    match track_item(&id, number, DetailKind::Pull) {
        Ok((body, status)) => (status, Json(body)).into_response(),
        Err((status, message)) => (status, message).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::sync::{Mutex, Once};
    use tempfile::TempDir;
    use tower::ServiceExt;

    use crate::services::github::snapshot::{GithubIssue, GithubLabel, GithubPull};
    use crate::services::github::snapshot_io::{set_test_cache_root, write_snapshot};

    static INIT: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn ensure_test_data_dir() -> std::path::PathBuf {
        INIT.call_once(|| {
            let base = std::env::temp_dir().join(format!(
                "ironpad-github-routes-tests-{}",
                std::process::id()
            ));
            fs::create_dir_all(&base).expect("test data base");
            std::env::set_var("IRONPAD_DATA_DIR", &base);
            config::init_data_dir();
        });
        config::data_dir().to_path_buf()
    }

    fn write_project(slug: &str, extra_fm: &str) {
        let dir = config::data_dir().join("projects").join(slug);
        fs::create_dir_all(&dir).expect("project dir");
        let content = format!(
            "---\nid: {slug}-index\ntype: project\ntitle: Test Project\ncreated: 2020-01-01T00:00:00Z\n{extra_fm}---\n\n# Test\n"
        );
        fs::write(dir.join("index.md"), content).expect("index");
    }

    fn read_index_frontmatter(slug: &str) -> serde_yaml::Mapping {
        let path = project_index_path(slug);
        let raw = fs::read_to_string(path).expect("read index");
        let (fm, _, _) = frontmatter::parse_frontmatter(&raw);
        fm
    }

    #[tokio::test]
    async fn get_no_snapshot_returns_stale_empty_lists() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-a", "github_repo: owner/repo\n");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/proj-a/github")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: GithubSnapshotResponse = serde_json::from_slice(&bytes).expect("json");
        assert!(body.stale);
        assert!(body.issues.is_empty());
        assert!(body.pulls.is_empty());
        assert_eq!(body.repo, None);
    }

    #[tokio::test]
    async fn put_valid_repo_writes_frontmatter_and_returns_200() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-put", "");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/proj-put/github")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"repo":"acme/widget"}"#))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: PutGithubRepoResponse = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(body.repo, "acme/widget");

        let fm = read_index_frontmatter("proj-put");
        let repo = fm
            .get(&serde_yaml::Value::from("github_repo"))
            .and_then(|v| v.as_str());
        assert_eq!(repo, Some("acme/widget"));
    }

    #[tokio::test]
    async fn put_full_url_returns_400_with_validation_message() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-bad-url", "");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/proj-bad-url/github")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"repo":"https://github.com/owner/repo"}"#))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(bytes.to_vec()).expect("utf8");
        assert_eq!(text, GITHUB_REPO_VALIDATION_MSG);
    }

    #[tokio::test]
    async fn put_git_suffix_returns_400() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-bad-git", "");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/proj-bad-git/github")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"repo":"owner/repo.git"}"#))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(bytes.to_vec()).expect("utf8");
        assert_eq!(text, GITHUB_REPO_VALIDATION_MSG);
    }

    #[tokio::test]
    async fn put_null_repo_clears_frontmatter_and_returns_204() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-clear", "github_repo: owner/repo\n");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/proj-clear/github")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"repo":null}"#))
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let fm = read_index_frontmatter("proj-clear");
        assert!(!fm.contains_key(serde_yaml::Value::from("github_repo")));
    }

    #[tokio::test]
    async fn get_after_recent_snapshot_returns_lists_and_not_stale() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-snap", "github_repo: owner/repo\n");

        let snap = Snapshot {
            repo: "owner/repo".to_string(),
            login: Some("octocat".to_string()),
            refreshed_at: Some(Utc::now().to_rfc3339()),
            issue_etag: None,
            pull_etag: None,
            issues: vec![GithubIssue {
                number: 1,
                title: "Issue one".to_string(),
                html_url: "https://github.com/owner/repo/issues/1".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
                author_login: "alice".to_string(),
                labels: vec![GithubLabel {
                    name: "bug".to_string(),
                    color: None,
                }],
                comments_count: 0,
                milestone_due: None,
                assignee_logins: vec![],
            }],
            pulls: vec![GithubPull {
                number: 2,
                title: "PR two".to_string(),
                html_url: "https://github.com/owner/repo/pull/2".to_string(),
                updated_at: "2026-01-02T00:00:00Z".to_string(),
                author_login: "bob".to_string(),
                labels: vec![],
                comments_count: 0,
                milestone_due: None,
                draft: false,
                requested_reviewer_logins: vec![],
                head_sha: "deadbeef".to_string(),
            }],
            issues_truncated: false,
            pulls_truncated: true,
        };
        write_snapshot("proj-snap", &snap).expect("write snapshot");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/proj-snap/github")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: GithubSnapshotResponse = serde_json::from_slice(&bytes).expect("json");
        assert!(!body.stale);
        assert_eq!(body.issues.len(), 1);
        assert_eq!(body.pulls.len(), 1);
        assert_eq!(body.issues[0].number, 1);
        assert_eq!(body.pulls[0].number, 2);
        assert!(body.pulls_truncated);
    }
}

#[cfg(test)]
mod refresh {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, HeaderMap, Request};
    use http_body_util::BodyExt;
    use std::sync::{Mutex, Once};
    use tempfile::TempDir;
    use tower::ServiceExt;

    use crate::services::github::lists::set_test_api_base;
    use crate::services::github::snapshot::{GithubIssue, GithubLabel, GithubPull};
    use crate::services::github::snapshot_io::{
        set_test_cache_root, write_body, write_comments, write_snapshot, CachedComment, ItemKind,
    };

    static INIT: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn ensure_test_data_dir() {
        INIT.call_once(|| {
            let base = std::env::temp_dir().join(format!(
                "ironpad-github-refresh-tests-{}",
                std::process::id()
            ));
            fs::create_dir_all(&base).expect("test data base");
            std::env::set_var("IRONPAD_DATA_DIR", &base);
            config::init_data_dir();
        });
    }

    fn write_project(slug: &str, extra_fm: &str) {
        let dir = config::data_dir().join("projects").join(slug);
        fs::create_dir_all(&dir).expect("project dir");
        let content = format!(
            "---\nid: {slug}-index\ntype: project\ntitle: Test Project\ncreated: 2020-01-01T00:00:00Z\n{extra_fm}---\n\n# Test\n"
        );
        fs::write(dir.join("index.md"), content).expect("index");
    }

    fn path_without_gh() -> &'static str {
        #[cfg(windows)]
        {
            "C:\\Windows\\System32"
        }
        #[cfg(not(windows))]
        {
            "/usr/bin"
        }
    }

    async fn start_mock() -> (String, tokio::task::JoinHandle<()>) {
        let app = Router::new()
            .route("/user", get(mock_user))
            .route("/repos/{owner}/{repo}/issues", get(mock_issues))
            .route("/repos/{owner}/{repo}/pulls", get(mock_pulls));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        (format!("http://{addr}"), handle)
    }

    async fn mock_user() -> impl IntoResponse {
        Json(serde_json::json!({"login": "octocat"}))
    }

    async fn mock_issues(
        Path((_owner, _repo)): Path<(String, String)>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        if headers.get(header::IF_NONE_MATCH).is_some() {
            return StatusCode::NOT_MODIFIED.into_response();
        }
        (
            [(header::ETAG, "W/\"issues-new\"")],
            Json(serde_json::json!([{
                "number": 7,
                "title": "Open issue",
                "html_url": "https://github.com/acme/widget/issues/7",
                "updated_at": "2026-05-01T00:00:00Z",
                "user": {"login": "alice"},
                "labels": [],
                "comments": 1
            }])),
        )
            .into_response()
    }

    async fn mock_pulls(
        Path((_owner, _repo)): Path<(String, String)>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        if headers.get(header::IF_NONE_MATCH).is_some() {
            return StatusCode::NOT_MODIFIED.into_response();
        }
        (
            [(header::ETAG, "W/\"pulls-new\"")],
            Json(serde_json::json!([{
                "number": 8,
                "title": "Open pull",
                "html_url": "https://github.com/acme/widget/pull/8",
                "updated_at": "2026-05-02T00:00:00Z",
                "user": {"login": "bob"},
                "labels": [],
                "comments": 0,
                "draft": false,
                "head": {"sha": "abc123"}
            }])),
        )
            .into_response()
    }

    fn sample_issue(number: u64, title: &str) -> GithubIssue {
        GithubIssue {
            number,
            title: title.to_string(),
            html_url: format!("https://github.com/acme/widget/issues/{number}"),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            author_login: "alice".to_string(),
            labels: vec![GithubLabel {
                name: "bug".to_string(),
                color: None,
            }],
            comments_count: 0,
            milestone_due: None,
            assignee_logins: vec![],
        }
    }

    fn sample_pull(number: u64, title: &str) -> GithubPull {
        GithubPull {
            number,
            title: title.to_string(),
            html_url: format!("https://github.com/acme/widget/pull/{number}"),
            updated_at: "2026-01-02T00:00:00Z".to_string(),
            author_login: "bob".to_string(),
            labels: vec![],
            comments_count: 0,
            milestone_due: None,
            draft: false,
            requested_reviewer_logins: vec![],
            head_sha: "deadbeef".to_string(),
        }
    }

    #[tokio::test]
    async fn refresh_writes_snapshot_with_issue_and_pull() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        let _path = EnvVarGuard::set("PATH", path_without_gh());
        let _github = EnvVarGuard::remove("GITHUB_TOKEN");
        let _token = EnvVarGuard::set("GH_TOKEN", "test-token");
        write_project("proj-refresh", "github_repo: acme/widget\n");
        write_body("proj-refresh", ItemKind::Issue, 99, "stale body").expect("body");
        write_comments(
            "proj-refresh",
            ItemKind::Pull,
            3,
            &[CachedComment {
                author_login: "carol".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                body: "old".to_string(),
            }],
        )
        .expect("comments");

        let (base, server) = start_mock().await;
        set_test_api_base(Some(base));

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proj-refresh/github/refresh")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: GithubSnapshotResponse = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(body.repo.as_deref(), Some("acme/widget"));
        assert_eq!(body.login.as_deref(), Some("octocat"));
        assert_eq!(body.issues.len(), 1);
        assert_eq!(body.issues[0].number, 7);
        assert_eq!(body.pulls.len(), 1);
        assert_eq!(body.pulls[0].number, 8);
        assert!(body.error.is_none());

        let stored = read_snapshot("proj-refresh").expect("snapshot");
        assert_eq!(stored.issues.len(), 1);
        assert_eq!(stored.pulls.len(), 1);
        assert!(stored.refreshed_at.is_some());

        let bodies = _cache.path().join("proj-refresh").join("bodies");
        assert!(!bodies.join("issue-99.md").exists());
        assert!(!bodies.join("comments-pull-3.json").exists());

        set_test_api_base(None);
        set_test_cache_root(None);
        server.abort();
    }

    #[tokio::test]
    async fn refresh_both_304_updates_refreshed_at_and_keeps_lists() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        let _path = EnvVarGuard::set("PATH", path_without_gh());
        let _github = EnvVarGuard::remove("GITHUB_TOKEN");
        let _token = EnvVarGuard::set("GH_TOKEN", "test-token");
        write_project("proj-304", "github_repo: acme/widget\n");

        let previous = "2020-01-01T00:00:00Z".to_string();
        let snap = Snapshot {
            repo: "acme/widget".to_string(),
            login: Some("octocat".to_string()),
            refreshed_at: Some(previous.clone()),
            issue_etag: Some("W/\"issues-old\"".to_string()),
            pull_etag: Some("W/\"pulls-old\"".to_string()),
            issues: vec![sample_issue(4, "Cached issue")],
            pulls: vec![sample_pull(5, "Cached pull")],
            issues_truncated: false,
            pulls_truncated: true,
        };
        write_snapshot("proj-304", &snap).expect("seed");

        let (base, server) = start_mock().await;
        set_test_api_base(Some(base));

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proj-304/github/refresh")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: GithubSnapshotResponse = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(body.issues.len(), 1);
        assert_eq!(body.issues[0].number, 4);
        assert_eq!(body.issues[0].title, "Cached issue");
        assert_eq!(body.pulls.len(), 1);
        assert_eq!(body.pulls[0].number, 5);
        assert!(body.pulls_truncated);
        assert_ne!(body.refreshed_at.as_deref(), Some(previous.as_str()));

        let stored = read_snapshot("proj-304").expect("snapshot");
        assert_eq!(stored.issues[0].title, "Cached issue");
        assert_eq!(stored.pulls[0].title, "Cached pull");
        assert_ne!(stored.refreshed_at.as_deref(), Some(previous.as_str()));

        set_test_api_base(None);
        set_test_cache_root(None);
        server.abort();
    }

    #[tokio::test]
    async fn refresh_without_token_returns_gh_auth_login_message() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        let _path = EnvVarGuard::set("PATH", path_without_gh());
        let _gh = EnvVarGuard::remove("GH_TOKEN");
        let _github = EnvVarGuard::remove("GITHUB_TOKEN");
        write_project("proj-no-auth", "github_repo: acme/widget\n");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proj-no-auth/github/refresh")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(bytes.to_vec()).expect("utf8");
        assert!(text.contains("gh auth login"));
        set_test_cache_root(None);
    }

    #[tokio::test]
    async fn refresh_missing_github_repo_returns_400() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-no-repo", "");

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proj-no-repo/github/refresh")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(bytes.to_vec()).expect("utf8");
        assert_eq!(text, "No repository linked");
        set_test_cache_root(None);
    }
}

#[cfg(test)]
mod detail {
    use super::*;
    use axum::body::Body;
    use axum::extract::State;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, Once};
    use tempfile::TempDir;
    use tower::ServiceExt;

    use crate::services::github::lists::set_test_api_base;
    use crate::services::github::snapshot::{GithubIssue, GithubPull};
    use crate::services::github::snapshot_io::{
        read_body, read_comments, set_test_cache_root, write_body, write_snapshot, ItemKind,
    };

    static INIT: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    #[derive(Clone)]
    struct MockHits {
        issue_body: Arc<AtomicUsize>,
        fail: Arc<AtomicBool>,
    }

    fn ensure_test_data_dir() {
        INIT.call_once(|| {
            let base = std::env::temp_dir().join(format!(
                "ironpad-github-detail-tests-{}",
                std::process::id()
            ));
            fs::create_dir_all(&base).expect("test data base");
            std::env::set_var("IRONPAD_DATA_DIR", &base);
            config::init_data_dir();
        });
    }

    fn path_without_gh() -> &'static str {
        #[cfg(windows)]
        {
            "C:\\Windows\\System32"
        }
        #[cfg(not(windows))]
        {
            "/usr/bin"
        }
    }

    fn sample_issue(number: u64) -> GithubIssue {
        GithubIssue {
            number,
            title: format!("Issue {number}"),
            html_url: format!("https://github.com/acme/widget/issues/{number}"),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            author_login: "alice".to_string(),
            labels: vec![],
            comments_count: 2,
            milestone_due: None,
            assignee_logins: vec![],
        }
    }

    fn sample_pull(number: u64, sha: &str) -> GithubPull {
        GithubPull {
            number,
            title: format!("Pull {number}"),
            html_url: format!("https://github.com/acme/widget/pull/{number}"),
            updated_at: "2026-01-02T00:00:00Z".to_string(),
            author_login: "bob".to_string(),
            labels: vec![],
            comments_count: 0,
            milestone_due: None,
            draft: false,
            requested_reviewer_logins: vec![],
            head_sha: sha.to_string(),
        }
    }

    fn seed_snapshot(project: &str, issues: Vec<GithubIssue>, pulls: Vec<GithubPull>) {
        write_snapshot(
            project,
            &Snapshot {
                repo: "acme/widget".to_string(),
                login: Some("octocat".to_string()),
                refreshed_at: Some("2026-05-01T00:00:00Z".to_string()),
                issue_etag: None,
                pull_etag: None,
                issues,
                pulls,
                issues_truncated: false,
                pulls_truncated: false,
            },
        )
        .expect("snapshot");
    }

    async fn start_mock(hits: MockHits) -> (String, tokio::task::JoinHandle<()>) {
        let app = Router::new()
            .route(
                "/repos/{owner}/{repo}/issues/{number}/comments",
                get(mock_comments),
            )
            .route(
                "/repos/{owner}/{repo}/issues/{number}",
                get(mock_issue_body),
            )
            .route("/repos/{owner}/{repo}/pulls/{number}", get(mock_pull_body))
            .route(
                "/repos/{owner}/{repo}/commits/{sha}/status",
                get(mock_status),
            )
            .with_state(hits);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        (format!("http://{addr}"), handle)
    }

    async fn mock_issue_body(
        Path((_owner, _repo, number)): Path<(String, String, u64)>,
        State(hits): State<MockHits>,
    ) -> impl IntoResponse {
        hits.issue_body.fetch_add(1, Ordering::SeqCst);
        if hits.fail.load(Ordering::SeqCst) {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        if number == 77 {
            let huge = "a".repeat(MAX_BODY_BYTES + 1);
            return Json(serde_json::json!({ "body": huge })).into_response();
        }
        Json(serde_json::json!({ "body": "Hello from GitHub" })).into_response()
    }

    async fn mock_pull_body(
        Path((_owner, _repo, _number)): Path<(String, String, u64)>,
        State(hits): State<MockHits>,
    ) -> impl IntoResponse {
        if hits.fail.load(Ordering::SeqCst) {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Json(serde_json::json!({ "body": "Pull body" })).into_response()
    }

    async fn mock_comments(State(hits): State<MockHits>) -> impl IntoResponse {
        if hits.fail.load(Ordering::SeqCst) {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Json(serde_json::json!([
            {"user": {"login": "alice"}, "created_at": "2026-01-01T00:00:00Z", "body": "older"},
            {"user": {"login": "bob"}, "created_at": "2026-02-01T00:00:00Z", "body": "newer"}
        ]))
        .into_response()
    }

    async fn mock_status(
        Path((_owner, _repo, _sha)): Path<(String, String, String)>,
        State(hits): State<MockHits>,
    ) -> impl IntoResponse {
        if hits.fail.load(Ordering::SeqCst) {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
        Json(serde_json::json!({ "state": "success" })).into_response()
    }

    async fn get_detail(uri: &str) -> (StatusCode, serde_json::Value) {
        let app = project_github_router();
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .expect("response");
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        if status == StatusCode::NOT_FOUND {
            let text = String::from_utf8(bytes.to_vec()).unwrap_or_default();
            return (status, serde_json::Value::String(text));
        }
        let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    fn token_guards() -> (EnvVarGuard, EnvVarGuard, EnvVarGuard) {
        (
            EnvVarGuard::set("PATH", path_without_gh()),
            EnvVarGuard::remove("GITHUB_TOKEN"),
            EnvVarGuard::set("GH_TOKEN", "test-token"),
        )
    }

    #[tokio::test]
    async fn issue_detail_fetches_then_serves_cached_body_without_another_body_call() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        let _tokens = token_guards();
        seed_snapshot("proj-detail", vec![sample_issue(7)], vec![]);

        let hits = MockHits {
            issue_body: Arc::new(AtomicUsize::new(0)),
            fail: Arc::new(AtomicBool::new(false)),
        };
        let (base, server) = start_mock(hits.clone()).await;
        set_test_api_base(Some(base));

        let (status, body) = get_detail("/proj-detail/github/issues/7").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["body"], "Hello from GitHub");
        assert_eq!(body["body_too_large"], false);
        assert_eq!(body["html_url"], "https://github.com/acme/widget/issues/7");
        assert_eq!(body["comments"][0]["author_login"], "bob");
        assert_eq!(body["comments"][0]["body"], "newer");
        assert_eq!(body["more_comments"], false);
        assert!(body["error"].is_null());
        assert_eq!(hits.issue_body.load(Ordering::SeqCst), 1);
        assert_eq!(
            read_body("proj-detail", ItemKind::Issue, 7).as_deref(),
            Some("Hello from GitHub")
        );
        let cached_comments = read_comments("proj-detail", ItemKind::Issue, 7).expect("comments");
        assert_eq!(cached_comments[0].author_login, "bob");

        let (status, again) = get_detail("/proj-detail/github/issues/7").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(again["body"], "Hello from GitHub");
        assert_eq!(hits.issue_body.load(Ordering::SeqCst), 1);

        set_test_api_base(None);
        set_test_cache_root(None);
        server.abort();
    }

    #[tokio::test]
    async fn issue_body_over_one_mib_sets_body_too_large_and_skips_cache() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        let _tokens = token_guards();
        seed_snapshot("proj-large", vec![sample_issue(77)], vec![]);

        let hits = MockHits {
            issue_body: Arc::new(AtomicUsize::new(0)),
            fail: Arc::new(AtomicBool::new(false)),
        };
        let (base, server) = start_mock(hits).await;
        set_test_api_base(Some(base));

        let (status, body) = get_detail("/proj-large/github/issues/77").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["body_too_large"], true);
        assert!(body["body"].is_null());
        assert!(read_body("proj-large", ItemKind::Issue, 77).is_none());

        set_test_api_base(None);
        set_test_cache_root(None);
        server.abort();
    }

    #[tokio::test]
    async fn pull_detail_includes_commit_status() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        let _tokens = token_guards();
        seed_snapshot("proj-pr", vec![], vec![sample_pull(8, "abc123")]);

        let hits = MockHits {
            issue_body: Arc::new(AtomicUsize::new(0)),
            fail: Arc::new(AtomicBool::new(false)),
        };
        let (base, server) = start_mock(hits).await;
        set_test_api_base(Some(base));

        let (status, body) = get_detail("/proj-pr/github/pulls/8").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["body"], "Pull body");
        assert_eq!(body["commit_status"], "success");
        assert_eq!(body["html_url"], "https://github.com/acme/widget/pull/8");

        set_test_api_base(None);
        set_test_cache_root(None);
        server.abort();
    }

    #[tokio::test]
    async fn missing_number_returns_404_prd_sentence() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        seed_snapshot("proj-missing", vec![sample_issue(7)], vec![]);

        let (status, body) = get_detail("/proj-missing/github/issues/99").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(
            body,
            serde_json::Value::String(NOT_IN_OPEN_SNAPSHOT.to_string())
        );

        set_test_cache_root(None);
    }

    #[tokio::test]
    async fn github_500_returns_cached_body_and_error() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        let _tokens = token_guards();
        seed_snapshot("proj-fail", vec![sample_issue(7)], vec![]);
        write_body("proj-fail", ItemKind::Issue, 7, "cached markdown").expect("cache body");

        let hits = MockHits {
            issue_body: Arc::new(AtomicUsize::new(0)),
            fail: Arc::new(AtomicBool::new(true)),
        };
        let (base, server) = start_mock(hits.clone()).await;
        set_test_api_base(Some(base));

        let (status, body) = get_detail("/proj-fail/github/issues/7").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["body"], "cached markdown");
        assert!(body["error"].as_str().is_some());
        assert_eq!(hits.issue_body.load(Ordering::SeqCst), 0);

        set_test_api_base(None);
        set_test_cache_root(None);
        server.abort();
    }
}

#[cfg(test)]
mod track {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::sync::{Mutex, Once};
    use tempfile::TempDir;
    use tower::ServiceExt;

    use crate::services::github::snapshot::{GithubIssue, GithubLabel, GithubPull};
    use crate::services::github::snapshot_io::{
        set_test_cache_root, write_body, write_snapshot, ItemKind,
    };

    static INIT: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn ensure_test_data_dir() {
        INIT.call_once(|| {
            let base = std::env::temp_dir()
                .join(format!("ironpad-github-track-tests-{}", std::process::id()));
            fs::create_dir_all(&base).expect("test data base");
            std::env::set_var("IRONPAD_DATA_DIR", &base);
            config::init_data_dir();
        });
    }

    fn write_project(slug: &str) {
        let dir = config::data_dir().join("projects").join(slug);
        fs::create_dir_all(&dir).expect("project dir");
        let content = format!(
            "---\nid: {slug}-index\ntype: project\ntitle: Test Project\ncreated: 2020-01-01T00:00:00Z\ngithub_repo: acme/widget\n---\n\n# Test\n"
        );
        fs::write(dir.join("index.md"), content).expect("index");
    }

    fn issue_42_snapshot() -> GithubIssue {
        GithubIssue {
            number: 42,
            title: "Track me".to_string(),
            html_url: "https://github.com/acme/widget/issues/42".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            author_login: "alice".to_string(),
            labels: vec![
                GithubLabel {
                    name: "bug".to_string(),
                    color: None,
                },
                GithubLabel {
                    name: "help wanted".to_string(),
                    color: None,
                },
            ],
            comments_count: 0,
            milestone_due: Some("2026-12-01".to_string()),
            assignee_logins: vec![],
        }
    }

    fn seed_snapshot(project: &str, issues: Vec<GithubIssue>, pulls: Vec<GithubPull>) {
        write_snapshot(
            project,
            &Snapshot {
                repo: "acme/widget".to_string(),
                login: Some("octocat".to_string()),
                refreshed_at: Some("2026-05-01T00:00:00Z".to_string()),
                issue_etag: None,
                pull_etag: None,
                issues,
                pulls,
                issues_truncated: false,
                pulls_truncated: false,
            },
        )
        .expect("snapshot");
    }

    async fn post_track(uri: &str) -> (StatusCode, TrackItemResponse) {
        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");
        let status = response.status();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        if status == StatusCode::NOT_FOUND {
            panic!("unexpected 404: {}", String::from_utf8_lossy(&bytes));
        }
        let body: TrackItemResponse = serde_json::from_slice(&bytes).expect("json");
        (status, body)
    }

    fn read_task_frontmatter(project: &str, task_id: &str) -> serde_yaml::Mapping {
        let tasks_dir = config::data_dir()
            .join("projects")
            .join(project)
            .join("tasks");
        for entry in fs::read_dir(&tasks_dir).expect("tasks dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            let raw = fs::read_to_string(&path).expect("read task");
            let (fm, _, _) = frontmatter::parse_frontmatter(&raw);
            if frontmatter::get_str(&fm, "id").as_deref() == Some(task_id) {
                return fm;
            }
        }
        panic!("task file not found for {task_id}");
    }

    #[tokio::test]
    async fn track_issue_creates_task_with_github_fields_and_body() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-track");
        seed_snapshot("proj-track", vec![issue_42_snapshot()], vec![]);
        write_body("proj-track", ItemKind::Issue, 42, "Issue body text").expect("body");

        let (status, first) = post_track("/proj-track/github/issues/42/track").await;
        assert_eq!(status, StatusCode::CREATED);
        assert!(first.created);
        assert!(!first.task_id.is_empty());

        let fm = read_task_frontmatter("proj-track", &first.task_id);
        assert_eq!(
            frontmatter::get_str(&fm, "github_url").as_deref(),
            Some("https://github.com/acme/widget/issues/42")
        );
        assert_eq!(frontmatter::get_u64(&fm, "github_number"), Some(42));
        assert_eq!(
            frontmatter::get_string_seq(&fm, "tags"),
            vec!["bug".to_string(), "help wanted".to_string()]
        );
        assert_eq!(
            frontmatter::get_str(&fm, "due_date").as_deref(),
            Some("2026-12-01")
        );

        let (status, second) = post_track("/proj-track/github/issues/42/track").await;
        assert_eq!(status, StatusCode::OK);
        assert!(!second.created);
        assert_eq!(second.task_id, first.task_id);

        set_test_cache_root(None);
    }

    #[tokio::test]
    async fn second_project_tracks_same_url_independently() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-a-track");
        write_project("proj-b-track");
        seed_snapshot("proj-a-track", vec![issue_42_snapshot()], vec![]);
        seed_snapshot("proj-b-track", vec![issue_42_snapshot()], vec![]);

        let (_, a) = post_track("/proj-a-track/github/issues/42/track").await;
        let (_, b) = post_track("/proj-b-track/github/issues/42/track").await;
        assert_ne!(a.task_id, b.task_id);
        assert!(a.created);
        assert!(b.created);

        set_test_cache_root(None);
    }

    #[tokio::test]
    async fn track_missing_number_returns_404() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));
        write_project("proj-missing-track");
        seed_snapshot("proj-missing-track", vec![issue_42_snapshot()], vec![]);

        let app = project_github_router();
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proj-missing-track/github/issues/99/track")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            String::from_utf8(bytes.to_vec()).expect("utf8"),
            NOT_IN_OPEN_SNAPSHOT
        );

        set_test_cache_root(None);
    }
}

#[cfg(test)]
mod summary {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::sync::{Mutex, Once};
    use tempfile::TempDir;
    use tower::ServiceExt;

    use crate::services::github::snapshot::{GithubIssue, GithubPull};
    use crate::services::github::snapshot_io::{set_test_cache_root, write_snapshot};

    static INIT: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn ensure_test_data_dir() {
        INIT.call_once(|| {
            let base = std::env::temp_dir().join(format!(
                "ironpad-github-summary-tests-{}",
                std::process::id()
            ));
            fs::create_dir_all(&base).expect("test data base");
            std::env::set_var("IRONPAD_DATA_DIR", &base);
            config::init_data_dir();
        });
    }

    fn write_project(slug: &str) {
        let dir = config::data_dir().join("projects").join(slug);
        fs::create_dir_all(&dir).expect("project dir");
        let content = format!(
            "---\nid: {slug}-index\ntype: project\ntitle: Test Project\ncreated: 2020-01-01T00:00:00Z\n---\n\n# Test\n"
        );
        fs::write(dir.join("index.md"), content).expect("index");
    }

    #[tokio::test]
    async fn summary_counts_assigned_and_review_per_login() {
        let _guard = TEST_MUTEX.lock().unwrap();
        ensure_test_data_dir();
        let _cache = TempDir::new().expect("cache");
        set_test_cache_root(Some(_cache.path().to_path_buf()));

        write_project("proj-sum-a");
        write_project("proj-sum-b");

        let snap_a = Snapshot {
            repo: "acme/alpha".to_string(),
            login: Some("alice".to_string()),
            refreshed_at: Some("2026-05-01T12:00:00Z".to_string()),
            issue_etag: None,
            pull_etag: None,
            issues: vec![
                GithubIssue {
                    number: 1,
                    title: "Assigned".to_string(),
                    html_url: "https://github.com/acme/alpha/issues/1".to_string(),
                    updated_at: "2026-01-01T00:00:00Z".to_string(),
                    author_login: "bob".to_string(),
                    labels: vec![],
                    comments_count: 0,
                    milestone_due: None,
                    assignee_logins: vec!["Alice".to_string()],
                },
                GithubIssue {
                    number: 2,
                    title: "Other".to_string(),
                    html_url: "https://github.com/acme/alpha/issues/2".to_string(),
                    updated_at: "2026-01-02T00:00:00Z".to_string(),
                    author_login: "bob".to_string(),
                    labels: vec![],
                    comments_count: 0,
                    milestone_due: None,
                    assignee_logins: vec!["bob".to_string()],
                },
            ],
            pulls: vec![
                GithubPull {
                    number: 10,
                    title: "Review".to_string(),
                    html_url: "https://github.com/acme/alpha/pull/10".to_string(),
                    updated_at: "2026-01-03T00:00:00Z".to_string(),
                    author_login: "bob".to_string(),
                    labels: vec![],
                    comments_count: 0,
                    milestone_due: None,
                    draft: false,
                    requested_reviewer_logins: vec!["alice".to_string()],
                    head_sha: "abc".to_string(),
                },
                GithubPull {
                    number: 11,
                    title: "No review".to_string(),
                    html_url: "https://github.com/acme/alpha/pull/11".to_string(),
                    updated_at: "2026-01-04T00:00:00Z".to_string(),
                    author_login: "bob".to_string(),
                    labels: vec![],
                    comments_count: 0,
                    milestone_due: None,
                    draft: false,
                    requested_reviewer_logins: vec!["carol".to_string()],
                    head_sha: "def".to_string(),
                },
            ],
            issues_truncated: false,
            pulls_truncated: false,
        };
        write_snapshot("proj-sum-a", &snap_a).expect("snapshot a");

        let snap_b = Snapshot {
            repo: "acme/beta".to_string(),
            login: None,
            refreshed_at: Some("2026-05-01T12:00:00Z".to_string()),
            issue_etag: None,
            pull_etag: None,
            issues: vec![],
            pulls: vec![],
            issues_truncated: false,
            pulls_truncated: false,
        };
        write_snapshot("proj-sum-b", &snap_b).expect("snapshot b");

        let app = Router::new().route("/github/summary", get(github_summary));
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/github/summary")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body: Vec<GithubSummaryEntry> = serde_json::from_slice(&bytes).expect("json");

        assert_eq!(body.len(), 1);
        assert_eq!(body[0].project_id, "proj-sum-a");
        assert_eq!(body[0].repo, "acme/alpha");
        assert_eq!(body[0].assigned_issues, 1);
        assert_eq!(body[0].review_pulls, 1);
        assert_eq!(
            body[0].refreshed_at.as_deref(),
            Some("2026-05-01T12:00:00Z")
        );

        set_test_cache_root(None);
    }
}
