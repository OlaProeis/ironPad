use axum::{
    body::Bytes, extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json,
    Router,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::config;
use crate::services::filesystem;
use crate::services::frontmatter;

#[derive(Debug, Serialize)]
pub struct DailyNote {
    pub id: String,
    pub date: String,
    pub path: String,
    pub content: String,
    pub frontmatter: serde_yaml::Mapping,
}

#[derive(Debug, Serialize)]
pub struct DailyNoteSummary {
    pub id: String,
    pub date: String,
    pub path: String,
    pub title: String,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_daily_notes))
        .route("/today", get(get_or_create_today))
        .route(
            "/{date}",
            get(get_daily_note)
                .post(create_daily_note)
                .put(update_daily_note),
        )
}

/// List all daily notes
async fn list_daily_notes() -> impl IntoResponse {
    match list_daily_notes_impl() {
        Ok(notes) => Json(notes).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list daily notes: {}", err),
        )
            .into_response(),
    }
}

fn list_daily_notes_impl() -> Result<Vec<DailyNoteSummary>, String> {
    let daily_dir = config::data_dir().join("daily");

    // Create directory if it doesn't exist
    if !daily_dir.exists() {
        fs::create_dir_all(&daily_dir).map_err(|e| e.to_string())?;
        return Ok(Vec::new());
    }

    let mut notes = Vec::new();

    for entry in fs::read_dir(&daily_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        // Validate date format
        if NaiveDate::parse_from_str(filename, "%Y-%m-%d").is_err() {
            continue;
        }

        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let (fm, _, _) = frontmatter::parse_frontmatter(&content);

        let title = fm
            .get(&serde_yaml::Value::from("title"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| filename.to_string());

        notes.push(DailyNoteSummary {
            id: format!("daily-{}", filename),
            date: filename.to_string(),
            path: format!("daily/{}.md", filename),
            title,
        });
    }

    // Sort by date descending
    notes.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(notes)
}

/// Get or create today's daily note
async fn get_or_create_today() -> impl IntoResponse {
    let today = Utc::now().format("%Y-%m-%d").to_string();

    match get_daily_note_impl(&today) {
        Ok(note) => Json(note).into_response(),
        Err(_) => {
            // Note doesn't exist, create it with default template
            match create_daily_note_impl(&today, None) {
                Ok(note) => (StatusCode::CREATED, Json(note)).into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create today's note: {}", err),
                )
                    .into_response(),
            }
        }
    }
}

/// Get a daily note by date
async fn get_daily_note(Path(date): Path<String>) -> impl IntoResponse {
    // Validate date format
    if NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid date format. Use YYYY-MM-DD",
        )
            .into_response();
    }

    match get_daily_note_impl(&date) {
        Ok(note) => Json(note).into_response(),
        Err(err) if err.contains("not found") => (StatusCode::NOT_FOUND, err).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get daily note: {}", err),
        )
            .into_response(),
    }
}

fn get_daily_note_impl(date: &str) -> Result<DailyNote, String> {
    let daily_dir = config::data_dir().join("daily");
    let note_path = daily_dir.join(format!("{}.md", date));

    if !note_path.exists() {
        return Err(format!("Daily note not found: {}", date));
    }

    let content = fs::read_to_string(&note_path).map_err(|e| e.to_string())?;
    let (fm, body, _) = frontmatter::parse_frontmatter(&content);

    Ok(DailyNote {
        id: format!("daily-{}", date),
        date: date.to_string(),
        path: format!("daily/{}.md", date),
        content: body,
        frontmatter: fm,
    })
}

#[derive(Debug, Deserialize)]
pub struct CreateDailyNoteRequest {
    pub content: Option<String>,
}

/// Create a daily note (optionally with content)
async fn create_daily_note(
    Path(date): Path<String>,
    body: Option<Json<CreateDailyNoteRequest>>,
) -> impl IntoResponse {
    // Validate date format
    if NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid date format. Use YYYY-MM-DD",
        )
            .into_response();
    }

    let content = body.and_then(|b| b.content.clone());

    match create_daily_note_impl(&date, content.as_deref()) {
        Ok(note) => (StatusCode::CREATED, Json(note)).into_response(),
        Err(err) if err.contains("already exists") => (StatusCode::CONFLICT, err).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create daily note: {}", err),
        )
            .into_response(),
    }
}

fn create_daily_note_impl(date: &str, initial_content: Option<&str>) -> Result<DailyNote, String> {
    let daily_dir = config::data_dir().join("daily");

    // Create directory if it doesn't exist
    if !daily_dir.exists() {
        fs::create_dir_all(&daily_dir).map_err(|e| e.to_string())?;
    }

    let note_path = daily_dir.join(format!("{}.md", date));

    if note_path.exists() {
        return Err(format!("Daily note already exists: {}", date));
    }

    let now = Utc::now().to_rfc3339();

    // Parse date for display
    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|e| e.to_string())?;
    let display_date = parsed_date.format("%A, %B %d, %Y").to_string();

    // Create frontmatter
    let mut fm = serde_yaml::Mapping::new();
    fm.insert(
        serde_yaml::Value::from("id"),
        serde_yaml::Value::from(format!("daily-{}", date)),
    );
    fm.insert(
        serde_yaml::Value::from("type"),
        serde_yaml::Value::from("daily"),
    );
    fm.insert(
        serde_yaml::Value::from("title"),
        serde_yaml::Value::from(display_date.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("date"),
        serde_yaml::Value::from(date),
    );
    fm.insert(
        serde_yaml::Value::from("created"),
        serde_yaml::Value::from(now.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now),
    );

    // Use provided content or default template
    let body = initial_content.map(|c| c.to_string()).unwrap_or_else(|| {
        format!(
            "# {}\n\n## Today's Focus\n\n- \n\n## Notes\n\n\n\n## Tasks\n\n- [ ] \n",
            display_date
        )
    });

    let content = frontmatter::serialize_frontmatter(&fm, &body)?;

    filesystem::atomic_write(&note_path, content.as_bytes())?;

    Ok(DailyNote {
        id: format!("daily-{}", date),
        date: date.to_string(),
        path: format!("daily/{}.md", date),
        content: body,
        frontmatter: fm,
    })
}

/// Update a daily note's content
async fn update_daily_note(Path(date): Path<String>, body: Bytes) -> impl IntoResponse {
    // Validate date format
    if NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_err() {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid date format. Use YYYY-MM-DD",
        )
            .into_response();
    }

    let content = String::from_utf8_lossy(&body).to_string();

    match update_daily_note_impl(&date, &content) {
        Ok(note) => Json(note).into_response(),
        Err(err) if err.contains("not found") => (StatusCode::NOT_FOUND, err).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update daily note: {}", err),
        )
            .into_response(),
    }
}

fn update_daily_note_impl(date: &str, new_content: &str) -> Result<DailyNote, String> {
    let daily_dir = config::data_dir().join("daily");
    let note_path = daily_dir.join(format!("{}.md", date));

    if !note_path.exists() {
        return Err(format!("Daily note not found: {}", date));
    }

    // Read existing file to preserve frontmatter
    let existing_content = fs::read_to_string(&note_path).map_err(|e| e.to_string())?;
    let (mut fm, _, _) = frontmatter::parse_frontmatter(&existing_content);

    // Update the 'updated' timestamp
    let now = Utc::now().to_rfc3339();
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now),
    );

    // Serialize with updated frontmatter and new content (atomic write)
    let file_content = frontmatter::serialize_frontmatter(&fm, new_content)?;

    filesystem::atomic_write(&note_path, file_content.as_bytes())?;

    Ok(DailyNote {
        id: format!("daily-{}", date),
        date: date.to_string(),
        path: format!("daily/{}.md", date),
        content: new_content.to_string(),
        frontmatter: fm,
    })
}
