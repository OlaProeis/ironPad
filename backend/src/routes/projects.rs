use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::config;
use crate::routes::tasks::{
    create_task_handler, delete_task_handler, get_task_handler, list_project_tasks_handler,
    toggle_task_handler, update_task_content_handler, update_task_meta_handler, CreateTaskRequest,
    UpdateTaskMetaRequest,
};
use crate::services::filesystem;
use crate::services::frontmatter;

#[derive(Debug, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectWithContent {
    pub id: String,
    pub name: String,
    pub path: String,
    pub created: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectContentRequest {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectNote {
    pub id: String,
    pub title: String,
    pub path: String,
    pub project_id: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectNoteWithContent {
    pub id: String,
    pub title: String,
    pub path: String,
    pub project_id: String,
    pub created: String,
    pub updated: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub title: Option<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_projects).post(create_project))
        .route("/{id}", get(get_project))
        .route(
            "/{id}/content",
            get(get_project_content).put(update_project_content),
        )
        // Task routes (file-based)
        .route(
            "/{id}/tasks",
            get(get_project_tasks).post(create_project_task),
        )
        .route(
            "/{id}/tasks/{task_id}",
            get(get_project_task)
                .put(update_project_task)
                .delete(delete_project_task),
        )
        .route("/{id}/tasks/{task_id}/toggle", put(toggle_project_task))
        .route("/{id}/tasks/{task_id}/meta", put(update_project_task_meta))
        // Note routes
        .route(
            "/{id}/notes",
            get(list_project_notes).post(create_project_note),
        )
        .route(
            "/{id}/notes/{note_id}",
            get(get_project_note)
                .put(update_project_note)
                .delete(delete_project_note),
        )
}

// ============ Task Handlers ============

async fn get_project_tasks(Path(id): Path<String>) -> impl IntoResponse {
    list_project_tasks_handler(id).await
}

async fn create_project_task(
    Path(id): Path<String>,
    Json(payload): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    create_task_handler(id, payload).await
}

async fn get_project_task(Path((id, task_id)): Path<(String, String)>) -> impl IntoResponse {
    get_task_handler(id, task_id).await
}

async fn update_project_task(
    Path((id, task_id)): Path<(String, String)>,
    body: String,
) -> impl IntoResponse {
    update_task_content_handler(id, task_id, body).await
}

async fn toggle_project_task(Path((id, task_id)): Path<(String, String)>) -> impl IntoResponse {
    toggle_task_handler(id, task_id).await
}

async fn update_project_task_meta(
    Path((id, task_id)): Path<(String, String)>,
    Json(payload): Json<UpdateTaskMetaRequest>,
) -> impl IntoResponse {
    update_task_meta_handler(id, task_id, payload).await
}

async fn delete_project_task(Path((id, task_id)): Path<(String, String)>) -> impl IntoResponse {
    delete_task_handler(id, task_id).await
}

async fn list_projects() -> impl IntoResponse {
    match list_projects_impl() {
        Ok(projects) => Json(projects).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list projects: {}", err),
        )
            .into_response(),
    }
}

fn list_projects_impl() -> Result<Vec<Project>, String> {
    let projects_dir = config::data_dir().join("projects");

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut projects = Vec::new();

    for entry in fs::read_dir(&projects_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let index_path = path.join("index.md");
        if !index_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
        let (fm, _, _) = frontmatter::parse_frontmatter(&content);

        let id = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let name = fm
            .get(&serde_yaml::Value::from("title"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| id.clone());

        let created = fm
            .get(&serde_yaml::Value::from("created"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        projects.push(Project {
            id: id.clone(),
            name,
            path: format!("projects/{}", id),
            created,
        });
    }

    Ok(projects)
}

async fn get_project(Path(id): Path<String>) -> impl IntoResponse {
    let projects_dir = config::data_dir().join("projects").join(&id);
    let index_path = projects_dir.join("index.md");

    if !index_path.exists() {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    match fs::read_to_string(&index_path) {
        Ok(content) => {
            let (fm, _, _) = frontmatter::parse_frontmatter(&content);

            let name = fm
                .get(&serde_yaml::Value::from("title"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| id.clone());

            let created = fm
                .get(&serde_yaml::Value::from("created"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default();

            Json(Project {
                id: id.clone(),
                name,
                path: format!("projects/{}", id),
                created,
            })
            .into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read project: {}", err),
        )
            .into_response(),
    }
}

async fn create_project(Json(payload): Json<CreateProjectRequest>) -> impl IntoResponse {
    match create_project_impl(&payload.name) {
        Ok(project) => (StatusCode::CREATED, Json(project)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create project: {}", err),
        )
            .into_response(),
    }
}

fn create_project_impl(name: &str) -> Result<Project, String> {
    use chrono::Utc;

    // Create slug from name
    let slug = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if slug.is_empty() {
        return Err("Invalid project name".to_string());
    }

    let projects_dir = config::data_dir().join("projects");
    let project_dir = projects_dir.join(&slug);

    if project_dir.exists() {
        return Err("Project already exists".to_string());
    }

    // Create directories
    fs::create_dir_all(&project_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(project_dir.join("assets")).map_err(|e| e.to_string())?;

    // Create index.md
    let index_path = project_dir.join("index.md");
    let now = Utc::now().to_rfc3339();

    let mut fm = serde_yaml::Mapping::new();
    fm.insert(
        serde_yaml::Value::from("id"),
        serde_yaml::Value::from(format!("{}-index", slug)),
    );
    fm.insert(
        serde_yaml::Value::from("type"),
        serde_yaml::Value::from("project"),
    );
    fm.insert(
        serde_yaml::Value::from("title"),
        serde_yaml::Value::from(name),
    );
    fm.insert(
        serde_yaml::Value::from("created"),
        serde_yaml::Value::from(now.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now.clone()),
    );

    let content = frontmatter::serialize_frontmatter(&fm, &format!("# {}\n\n", name))?;

    filesystem::atomic_write(&index_path, content.as_bytes())?;

    // Also create notes directory for project-scoped notes
    fs::create_dir_all(project_dir.join("notes")).map_err(|e| e.to_string())?;

    // Create tasks directory for file-based tasks
    fs::create_dir_all(project_dir.join("tasks")).map_err(|e| e.to_string())?;

    Ok(Project {
        id: slug.clone(),
        name: name.to_string(),
        path: format!("projects/{}", slug),
        created: now,
    })
}

async fn get_project_content(Path(id): Path<String>) -> impl IntoResponse {
    let index_path = config::data_dir()
        .join("projects")
        .join(&id)
        .join("index.md");

    if !index_path.exists() {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    match fs::read_to_string(&index_path) {
        Ok(content) => {
            let (fm, body, _) = frontmatter::parse_frontmatter(&content);

            let name = fm
                .get(&serde_yaml::Value::from("title"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| id.clone());

            let created = fm
                .get(&serde_yaml::Value::from("created"))
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_default();

            Json(ProjectWithContent {
                id: id.clone(),
                name,
                path: format!("projects/{}", id),
                created,
                content: body,
            })
            .into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to read project: {}", err),
        )
            .into_response(),
    }
}

async fn update_project_content(Path(id): Path<String>, body: String) -> impl IntoResponse {
    let index_path = config::data_dir()
        .join("projects")
        .join(&id)
        .join("index.md");

    if !index_path.exists() {
        return (StatusCode::NOT_FOUND, "Project not found").into_response();
    }

    // Read existing file to get frontmatter
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

    let (mut fm, _, _) = frontmatter::parse_frontmatter(&existing);

    // Update the timestamp
    let now = chrono::Utc::now().to_rfc3339();
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now),
    );

    // Serialize with new content
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

    // Write back (atomic to prevent corruption)
    if let Err(err) = filesystem::atomic_write(&index_path, new_content.as_bytes()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write file: {}", err),
        )
            .into_response();
    }

    let name = fm
        .get(&serde_yaml::Value::from("title"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| id.clone());

    let created = fm
        .get(&serde_yaml::Value::from("created"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    Json(ProjectWithContent {
        id: id.clone(),
        name,
        path: format!("projects/{}", id),
        created,
        content: body,
    })
    .into_response()
}

// ============ Project Notes Handlers ============

async fn list_project_notes(Path(project_id): Path<String>) -> impl IntoResponse {
    let notes_dir = config::data_dir()
        .join("projects")
        .join(&project_id)
        .join("notes");

    // Create notes directory if it doesn't exist
    if !notes_dir.exists() {
        if let Err(e) = fs::create_dir_all(&notes_dir) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create notes directory: {}", e),
            )
                .into_response();
        }
    }

    let mut notes = Vec::new();

    let entries = match fs::read_dir(&notes_dir) {
        Ok(e) => e,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read notes directory: {}", err),
            )
                .into_response();
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, _, _) = frontmatter::parse_frontmatter(&content);

        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let id = fm
            .get(&serde_yaml::Value::from("id"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| filename.clone());

        let title = fm
            .get(&serde_yaml::Value::from("title"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| filename.clone());

        let created = fm
            .get(&serde_yaml::Value::from("created"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        let updated = fm
            .get(&serde_yaml::Value::from("updated"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        notes.push(ProjectNote {
            id,
            title,
            path: format!("projects/{}/notes/{}.md", project_id, filename),
            project_id: project_id.clone(),
            created,
            updated,
        });
    }

    // Sort by updated date descending
    // Sort by created date (stable ordering - won't change when note is viewed/edited)
    notes.sort_by(|a, b| b.created.cmp(&a.created));

    Json(notes).into_response()
}

async fn create_project_note(
    Path(project_id): Path<String>,
    Json(payload): Json<CreateNoteRequest>,
) -> impl IntoResponse {
    use chrono::Utc;

    let notes_dir = config::data_dir()
        .join("projects")
        .join(&project_id)
        .join("notes");

    // Create notes directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&notes_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create notes directory: {}", e),
        )
            .into_response();
    }

    // Generate filename from timestamp
    let now = Utc::now();
    let filename = now.format("%Y%m%d-%H%M%S").to_string();
    let note_path = notes_dir.join(format!("{}.md", filename));

    let title = payload.title.unwrap_or_else(|| "Untitled".to_string());
    let now_str = now.to_rfc3339();

    let mut fm = serde_yaml::Mapping::new();
    fm.insert(
        serde_yaml::Value::from("id"),
        serde_yaml::Value::from(format!("{}-{}", project_id, filename)),
    );
    fm.insert(
        serde_yaml::Value::from("type"),
        serde_yaml::Value::from("note"),
    );
    fm.insert(
        serde_yaml::Value::from("title"),
        serde_yaml::Value::from(title.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("project_id"),
        serde_yaml::Value::from(project_id.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("created"),
        serde_yaml::Value::from(now_str.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now_str.clone()),
    );

    let body = format!("# {}\n\n", title);
    let content = match frontmatter::serialize_frontmatter(&fm, &body) {
        Ok(c) => c,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to serialize frontmatter: {}", err),
            )
                .into_response();
        }
    };

    if let Err(err) = filesystem::atomic_write(&note_path, content.as_bytes()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write note file: {}", err),
        )
            .into_response();
    }

    (
        StatusCode::CREATED,
        Json(ProjectNoteWithContent {
            id: format!("{}-{}", project_id, filename),
            title,
            path: format!("projects/{}/notes/{}.md", project_id, filename),
            project_id,
            created: now_str.clone(),
            updated: now_str,
            content: body,
        }),
    )
        .into_response()
}

async fn get_project_note(
    Path((project_id, note_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let notes_dir = config::data_dir()
        .join("projects")
        .join(&project_id)
        .join("notes");

    // Try to find the note by ID (which might be the filename)
    let note_path = notes_dir.join(format!("{}.md", note_id));

    if !note_path.exists() {
        // Try to find by searching all notes for matching ID
        if let Ok(entries) = fs::read_dir(&notes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    let (fm, body, _) = frontmatter::parse_frontmatter(&content);

                    let file_id = fm
                        .get(&serde_yaml::Value::from("id"))
                        .and_then(|v| v.as_str())
                        .map(String::from);

                    if file_id.as_deref() == Some(&note_id) {
                        let title = fm
                            .get(&serde_yaml::Value::from("title"))
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_default();

                        let created = fm
                            .get(&serde_yaml::Value::from("created"))
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_default();

                        let updated = fm
                            .get(&serde_yaml::Value::from("updated"))
                            .and_then(|v| v.as_str())
                            .map(String::from)
                            .unwrap_or_default();

                        let filename = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();

                        return Json(ProjectNoteWithContent {
                            id: note_id,
                            title,
                            path: format!("projects/{}/notes/{}.md", project_id, filename),
                            project_id,
                            created,
                            updated,
                            content: body,
                        })
                        .into_response();
                    }
                }
            }
        }

        return (StatusCode::NOT_FOUND, "Note not found").into_response();
    }

    let content = match fs::read_to_string(&note_path) {
        Ok(c) => c,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read note: {}", err),
            )
                .into_response();
        }
    };

    let (fm, body, _) = frontmatter::parse_frontmatter(&content);

    let id = fm
        .get(&serde_yaml::Value::from("id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| note_id.clone());

    let title = fm
        .get(&serde_yaml::Value::from("title"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    let created = fm
        .get(&serde_yaml::Value::from("created"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    let updated = fm
        .get(&serde_yaml::Value::from("updated"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    Json(ProjectNoteWithContent {
        id,
        title,
        path: format!("projects/{}/notes/{}.md", project_id, note_id),
        project_id,
        created,
        updated,
        content: body,
    })
    .into_response()
}

async fn update_project_note(
    Path((project_id, note_id)): Path<(String, String)>,
    body: String,
) -> impl IntoResponse {
    let notes_dir = config::data_dir()
        .join("projects")
        .join(&project_id)
        .join("notes");

    let note_path = notes_dir.join(format!("{}.md", note_id));

    if !note_path.exists() {
        return (StatusCode::NOT_FOUND, "Note not found").into_response();
    }

    // Read existing content for frontmatter
    let existing = match fs::read_to_string(&note_path) {
        Ok(c) => c,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read note: {}", err),
            )
                .into_response();
        }
    };

    let (mut fm, _, _) = frontmatter::parse_frontmatter(&existing);

    // Update timestamp
    let now = chrono::Utc::now().to_rfc3339();
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now.clone()),
    );

    // Serialize with new content
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

    if let Err(err) = filesystem::atomic_write(&note_path, new_content.as_bytes()) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write file: {}", err),
        )
            .into_response();
    }

    let id = fm
        .get(&serde_yaml::Value::from("id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| note_id.clone());

    let title = fm
        .get(&serde_yaml::Value::from("title"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    let created = fm
        .get(&serde_yaml::Value::from("created"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_default();

    Json(ProjectNoteWithContent {
        id,
        title,
        path: format!("projects/{}/notes/{}.md", project_id, note_id),
        project_id,
        created,
        updated: now,
        content: body,
    })
    .into_response()
}

async fn delete_project_note(
    Path((project_id, note_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let notes_dir = config::data_dir()
        .join("projects")
        .join(&project_id)
        .join("notes");

    let note_path = notes_dir.join(format!("{}.md", note_id));

    if !note_path.exists() {
        return (StatusCode::NOT_FOUND, "Note not found").into_response();
    }

    // Move to archive instead of deleting
    let archive_dir = config::data_dir().join("archive");
    if let Err(e) = fs::create_dir_all(&archive_dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create archive directory: {}", e),
        )
            .into_response();
    }

    let archive_path = archive_dir.join(format!("{}-{}.md", project_id, note_id));

    if let Err(err) = fs::rename(&note_path, &archive_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to archive note: {}", err),
        )
            .into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
