use axum::{
    body::Body,
    extract::{Multipart, Path, Query},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path as StdPath;
use tokio_util::io::ReaderStream;

use crate::config;
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    pub project: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
    pub size: usize,
}

pub fn router() -> Router {
    Router::new()
        .route("/upload", post(upload_asset))
        .route("/{project}/{filename}", get(get_asset))
}

async fn upload_asset(
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Determine target directory
    let assets_dir = if let Some(project_id) = &query.project {
        config::data_dir()
            .join("projects")
            .join(project_id)
            .join("assets")
    } else {
        config::data_dir().join("notes").join("assets")
    };

    // Create assets directory if it doesn't exist
    if !assets_dir.exists() {
        if let Err(e) = fs::create_dir_all(&assets_dir) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create assets directory: {}", e),
            )
                .into_response();
        }
    }

    // Process uploaded file
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("file").to_string();
        if name != "file" {
            continue;
        }

        let original_filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("upload_{}", chrono::Utc::now().timestamp()));

        // Validate file type (images only for now)
        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_default();

        if !is_allowed_content_type(&content_type) {
            return (
                StatusCode::BAD_REQUEST,
                format!(
                    "Unsupported file type: {}. Only images are allowed.",
                    content_type
                ),
            )
                .into_response();
        }

        // Read file data
        let data = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read file data: {}", e),
                )
                    .into_response();
            }
        };

        // Check file size
        if data.len() > MAX_FILE_SIZE {
            return (
                StatusCode::BAD_REQUEST,
                format!(
                    "File too large. Maximum size is {} MB.",
                    MAX_FILE_SIZE / 1024 / 1024
                ),
            )
                .into_response();
        }

        // Generate unique filename if needed
        let filename = generate_unique_filename(&assets_dir, &original_filename);
        let file_path = assets_dir.join(&filename);

        // Write file
        let mut file = match fs::File::create(&file_path) {
            Ok(f) => f,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create file: {}", e),
                )
                    .into_response();
            }
        };

        if let Err(e) = file.write_all(&data) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write file: {}", e),
            )
                .into_response();
        }

        // Build response URL
        let project_part = query.project.as_deref().unwrap_or("notes");
        let url = format!("/api/assets/{}/{}", project_part, filename);

        return (
            StatusCode::CREATED,
            Json(UploadResponse {
                url,
                filename,
                size: data.len(),
            }),
        )
            .into_response();
    }

    (StatusCode::BAD_REQUEST, "No file provided").into_response()
}

/// Validate that a path component doesn't contain directory traversal
fn validate_path_component(component: &str) -> Result<(), String> {
    if component.contains("..")
        || component.contains('/')
        || component.contains('\\')
        || component.is_empty()
    {
        return Err("Invalid path component".to_string());
    }
    Ok(())
}

async fn get_asset(Path((project, filename)): Path<(String, String)>) -> impl IntoResponse {
    // Validate path components to prevent directory traversal
    if validate_path_component(&project).is_err() || validate_path_component(&filename).is_err() {
        return (StatusCode::BAD_REQUEST, "Invalid path").into_response();
    }

    // Determine file path
    let file_path = if project == "notes" {
        config::data_dir()
            .join("notes")
            .join("assets")
            .join(&filename)
    } else {
        config::data_dir()
            .join("projects")
            .join(&project)
            .join("assets")
            .join(&filename)
    };

    // Check if file exists
    if !file_path.exists() {
        return (StatusCode::NOT_FOUND, "Asset not found").into_response();
    }

    // Read file
    let file = match tokio::fs::File::open(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to open file: {}", e),
            )
                .into_response();
        }
    };

    // Determine content type
    let content_type = get_content_type(&filename);

    // Stream file response
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (StatusCode::OK, [(header::CONTENT_TYPE, content_type)], body).into_response()
}

fn is_allowed_content_type(content_type: &str) -> bool {
    matches!(
        content_type,
        "image/jpeg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "image/svg+xml"
            | "application/pdf"
    )
}

fn get_content_type(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

fn generate_unique_filename(dir: &StdPath, original: &str) -> String {
    // Extract name and extension
    let (name, ext) = if let Some(dot_idx) = original.rfind('.') {
        (&original[..dot_idx], &original[dot_idx..])
    } else {
        (original, "")
    };

    // Sanitize filename
    let sanitized_name: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let base_filename = format!("{}{}", sanitized_name, ext);
    let target_path = dir.join(&base_filename);

    // If file doesn't exist, use original name
    if !target_path.exists() {
        return base_filename;
    }

    // Otherwise, add timestamp
    let timestamp = chrono::Utc::now().timestamp_millis();
    format!("{}_{}{}", sanitized_name, timestamp, ext)
}
