use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path as StdPath;

use crate::services::filesystem;
use crate::config;
use crate::services::frontmatter;

/// Task summary for list views
#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub section: String,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub parent_id: Option<String>,
    pub recurrence: Option<String>,
    pub recurrence_interval: Option<u32>,
    pub project_id: String,
    pub path: String,
    pub created: String,
    pub updated: String,
}

/// Task with full content for detail view
#[derive(Debug, Clone, Serialize)]
pub struct TaskWithContent {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub section: String,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub is_active: bool,
    pub tags: Vec<String>,
    pub parent_id: Option<String>,
    pub recurrence: Option<String>,
    pub recurrence_interval: Option<u32>,
    pub project_id: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub section: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTaskMetaRequest {
    pub title: Option<String>,
    pub section: Option<String>,
    pub priority: Option<String>,
    pub due_date: Option<String>,
    pub is_active: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub recurrence: Option<String>,
    pub recurrence_interval: Option<u32>,
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(list_all_tasks_handler))
}

// ============ Handler Functions (called from projects.rs) ============

/// List all tasks for a project
pub async fn list_project_tasks_handler(project_id: String) -> impl IntoResponse {
    match list_project_tasks_impl(&project_id) {
        Ok(tasks) => Json(tasks).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list tasks: {}", err),
        )
            .into_response(),
    }
}

/// Create a new task
pub async fn create_task_handler(
    project_id: String,
    payload: CreateTaskRequest,
) -> impl IntoResponse {
    match create_task_impl(&project_id, &payload.title, payload.section.as_deref(), payload.parent_id.as_deref()) {
        Ok(task) => (StatusCode::CREATED, Json(task)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create task: {}", err),
        )
            .into_response(),
    }
}

/// Get a task with content
pub async fn get_task_handler(project_id: String, task_id: String) -> impl IntoResponse {
    match get_task_impl(&project_id, &task_id) {
        Ok(task) => Json(task).into_response(),
        Err(err) if err.contains("not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get task: {}", err),
        )
            .into_response(),
    }
}

/// Update task content (markdown body)
pub async fn update_task_content_handler(
    project_id: String,
    task_id: String,
    body: String,
) -> impl IntoResponse {
    match update_task_content_impl(&project_id, &task_id, &body) {
        Ok(task) => Json(task).into_response(),
        Err(err) if err.contains("not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update task: {}", err),
        )
            .into_response(),
    }
}

/// Toggle task completion
pub async fn toggle_task_handler(project_id: String, task_id: String) -> impl IntoResponse {
    match toggle_task_impl(&project_id, &task_id) {
        Ok(task) => Json(task).into_response(),
        Err(err) if err.contains("not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to toggle task: {}", err),
        )
            .into_response(),
    }
}

/// Update task metadata (title, section, priority)
pub async fn update_task_meta_handler(
    project_id: String,
    task_id: String,
    payload: UpdateTaskMetaRequest,
) -> impl IntoResponse {
    match update_task_meta_impl(&project_id, &task_id, payload) {
        Ok(task) => Json(task).into_response(),
        Err(err) if err.contains("not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to update task metadata: {}", err),
        )
            .into_response(),
    }
}

/// Delete (archive) a task
pub async fn delete_task_handler(project_id: String, task_id: String) -> impl IntoResponse {
    match delete_task_impl(&project_id, &task_id) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(err) if err.contains("not found") => {
            (StatusCode::NOT_FOUND, err).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to delete task: {}", err),
        )
            .into_response(),
    }
}

// ============ Implementation Functions ============

fn get_tasks_dir(project_id: &str) -> std::path::PathBuf {
    config::data_dir()
        .join("projects")
        .join(project_id)
        .join("tasks")
}

fn ensure_tasks_dir(project_id: &str) -> Result<std::path::PathBuf, String> {
    let tasks_dir = get_tasks_dir(project_id);
    if !tasks_dir.exists() {
        fs::create_dir_all(&tasks_dir).map_err(|e| e.to_string())?;
    }
    Ok(tasks_dir)
}

fn list_project_tasks_impl(project_id: &str) -> Result<Vec<Task>, String> {
    let tasks_dir = ensure_tasks_dir(project_id)?;

    let mut tasks = Vec::new();

    let entries = match fs::read_dir(&tasks_dir) {
        Ok(e) => e,
        Err(_) => return Ok(Vec::new()), // No tasks directory yet
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

        if let Some(task) = parse_task_file(&content, &path, project_id) {
            tasks.push(task);
        }
    }

    // Sort by updated date descending (most recent first)
    // Sort by created date (stable ordering - won't change when task is viewed/edited)
    tasks.sort_by(|a, b| b.created.cmp(&a.created));

    Ok(tasks)
}

/// Shared helper: extract common task fields from frontmatter.
/// Eliminates duplication between parse_task_file and parse_task_with_content.
fn extract_task_fields(fm: &serde_yaml::Mapping, path: &StdPath, project_id: &str) -> Task {
    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    Task {
        id: frontmatter::get_str_or(fm, "id", &filename),
        title: frontmatter::get_str_or(fm, "title", "Untitled"),
        completed: frontmatter::get_bool_or(fm, "completed", false),
        section: frontmatter::get_str_or(fm, "section", "Active"),
        priority: frontmatter::get_str(fm, "priority"),
        due_date: frontmatter::get_str(fm, "due_date"),
        is_active: frontmatter::get_bool_or(fm, "is_active", true),
        tags: frontmatter::get_string_seq(fm, "tags"),
        parent_id: frontmatter::get_str(fm, "parent_id"),
        recurrence: frontmatter::get_str(fm, "recurrence"),
        recurrence_interval: frontmatter::get_u64(fm, "recurrence_interval").map(|v| v as u32),
        project_id: project_id.to_string(),
        path: format!("projects/{}/tasks/{}.md", project_id, filename),
        created: frontmatter::get_str_or(fm, "created", ""),
        updated: frontmatter::get_str_or(fm, "updated", ""),
    }
}

fn parse_task_file(content: &str, path: &StdPath, project_id: &str) -> Option<Task> {
    let (fm, _, _) = frontmatter::parse_frontmatter(content);
    Some(extract_task_fields(&fm, path, project_id))
}

fn create_task_impl(
    project_id: &str,
    title: &str,
    section: Option<&str>,
    parent_id: Option<&str>,
) -> Result<TaskWithContent, String> {
    use chrono::Utc;

    let tasks_dir = ensure_tasks_dir(project_id)?;

    // Generate filename from timestamp
    let now = Utc::now();
    let filename = format!("task-{}", now.format("%Y%m%d-%H%M%S"));
    let task_path = tasks_dir.join(format!("{}.md", filename));

    let section = section.unwrap_or("Active").to_string();
    let now_str = now.to_rfc3339();
    let id = format!("{}-{}", project_id, filename);

    let mut fm = serde_yaml::Mapping::new();
    fm.insert(
        serde_yaml::Value::from("id"),
        serde_yaml::Value::from(id.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("type"),
        serde_yaml::Value::from("task"),
    );
    fm.insert(
        serde_yaml::Value::from("title"),
        serde_yaml::Value::from(title),
    );
    fm.insert(
        serde_yaml::Value::from("completed"),
        serde_yaml::Value::from(false),
    );
    fm.insert(
        serde_yaml::Value::from("section"),
        serde_yaml::Value::from(section.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("priority"),
        serde_yaml::Value::from("normal"),
    );
    fm.insert(
        serde_yaml::Value::from("is_active"),
        serde_yaml::Value::from(true),
    );
    fm.insert(
        serde_yaml::Value::from("project_id"),
        serde_yaml::Value::from(project_id),
    );
    if let Some(pid) = parent_id {
        fm.insert(
            serde_yaml::Value::from("parent_id"),
            serde_yaml::Value::from(pid),
        );
    }
    fm.insert(
        serde_yaml::Value::from("created"),
        serde_yaml::Value::from(now_str.clone()),
    );
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now_str.clone()),
    );

    let body = format!("# {}\n\n", title);
    let content = frontmatter::serialize_frontmatter(&fm, &body)?;

    filesystem::atomic_write(&task_path, content.as_bytes())?;

    Ok(TaskWithContent {
        id,
        title: title.to_string(),
        completed: false,
        section,
        priority: Some("normal".to_string()),
        due_date: None,
        is_active: true,
        tags: Vec::new(),
        parent_id: parent_id.map(String::from),
        recurrence: None,
        recurrence_interval: None,
        project_id: project_id.to_string(),
        path: format!("projects/{}/tasks/{}.md", project_id, filename),
        created: now_str.clone(),
        updated: now_str,
        content: body,
    })
}

fn get_task_impl(project_id: &str, task_id: &str) -> Result<TaskWithContent, String> {
    let tasks_dir = get_tasks_dir(project_id);

    // Try direct filename match first
    let task_path = tasks_dir.join(format!("{}.md", task_id));

    if task_path.exists() {
        return read_task_with_content(&task_path, project_id);
    }

    // Search by ID in frontmatter
    if let Ok(entries) = fs::read_dir(&tasks_dir) {
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

                if file_id.as_deref() == Some(task_id) {
                    return parse_task_with_content(&fm, &body, &path, project_id);
                }
            }
        }
    }

    Err("Task not found".to_string())
}

fn read_task_with_content(path: &StdPath, project_id: &str) -> Result<TaskWithContent, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (fm, body, _) = frontmatter::parse_frontmatter(&content);
    parse_task_with_content(&fm, &body, path, project_id)
}

fn parse_task_with_content(
    fm: &serde_yaml::Mapping,
    body: &str,
    path: &StdPath,
    project_id: &str,
) -> Result<TaskWithContent, String> {
    let task = extract_task_fields(fm, path, project_id);
    Ok(TaskWithContent {
        id: task.id,
        title: task.title,
        completed: task.completed,
        section: task.section,
        priority: task.priority,
        due_date: task.due_date,
        is_active: task.is_active,
        tags: task.tags,
        parent_id: task.parent_id,
        recurrence: task.recurrence,
        recurrence_interval: task.recurrence_interval,
        project_id: task.project_id,
        path: task.path,
        created: task.created,
        updated: task.updated,
        content: body.to_string(),
    })
}

fn update_task_content_impl(
    project_id: &str,
    task_id: &str,
    new_body: &str,
) -> Result<TaskWithContent, String> {
    let task_path = find_task_path(project_id, task_id)?;

    // Read existing content
    let existing = fs::read_to_string(&task_path).map_err(|e| e.to_string())?;
    let (mut fm, _, _) = frontmatter::parse_frontmatter(&existing);

    // Update timestamp
    let now = chrono::Utc::now().to_rfc3339();
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now),
    );

    // Serialize with new content (atomic write to prevent corruption)
    let new_content = frontmatter::serialize_frontmatter(&fm, new_body)?;
    filesystem::atomic_write(&task_path, new_content.as_bytes())?;

    parse_task_with_content(&fm, new_body, &task_path, project_id)
}

fn toggle_task_impl(project_id: &str, task_id: &str) -> Result<Task, String> {
    let task_path = find_task_path(project_id, task_id)?;

    // Read existing content
    let existing = fs::read_to_string(&task_path).map_err(|e| e.to_string())?;
    let (mut fm, body, _) = frontmatter::parse_frontmatter(&existing);

    // Toggle completed
    let current_completed = fm
        .get(&serde_yaml::Value::from("completed"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let new_completed = !current_completed;
    fm.insert(
        serde_yaml::Value::from("completed"),
        serde_yaml::Value::from(new_completed),
    );

    // Update section based on completion status
    let new_section = if new_completed {
        "Completed"
    } else {
        "Active"
    };
    fm.insert(
        serde_yaml::Value::from("section"),
        serde_yaml::Value::from(new_section),
    );

    // Update timestamp
    let now = chrono::Utc::now().to_rfc3339();
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now),
    );

    // Serialize and write (atomic to prevent corruption)
    let new_content = frontmatter::serialize_frontmatter(&fm, &body)?;
    filesystem::atomic_write(&task_path, new_content.as_bytes())?;

    // If completing a recurring task, create the next instance
    if new_completed {
        let recurrence = fm
            .get(&serde_yaml::Value::from("recurrence"))
            .and_then(|v| v.as_str())
            .map(String::from);

        if let Some(rec) = recurrence {
            let interval = fm
                .get(&serde_yaml::Value::from("recurrence_interval"))
                .and_then(|v| v.as_u64())
                .unwrap_or(1) as i64;

            let title = fm
                .get(&serde_yaml::Value::from("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Untitled")
                .to_string();

            let due_date = fm
                .get(&serde_yaml::Value::from("due_date"))
                .and_then(|v| v.as_str())
                .map(String::from);

            let tags = fm
                .get(&serde_yaml::Value::from("tags"))
                .and_then(|v| v.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            // Calculate next due date
            let next_due = calculate_next_due_date(due_date.as_deref(), &rec, interval);

            // Create the next recurring task
            let _ = create_recurring_task_impl(
                project_id,
                &title,
                next_due.as_deref(),
                &rec,
                interval as u32,
                &tags,
            );
        }
    }

    // Return updated task
    let task = parse_task_file(&fs::read_to_string(&task_path).unwrap(), &task_path, project_id)
        .ok_or_else(|| "Failed to parse updated task".to_string())?;

    Ok(task)
}

fn calculate_next_due_date(current_due: Option<&str>, recurrence: &str, interval: i64) -> Option<String> {
    use chrono::{NaiveDate, Duration, Utc, Months};

    let base_date = if let Some(due_str) = current_due {
        NaiveDate::parse_from_str(due_str, "%Y-%m-%d").unwrap_or_else(|_| Utc::now().date_naive())
    } else {
        Utc::now().date_naive()
    };

    let next = match recurrence {
        "daily" => Some(base_date + Duration::days(interval)),
        "weekly" => Some(base_date + Duration::weeks(interval)),
        "monthly" => base_date.checked_add_months(Months::new(interval as u32)),
        "yearly" => base_date.checked_add_months(Months::new((interval * 12) as u32)),
        _ => None,
    };

    next.map(|d| d.format("%Y-%m-%d").to_string())
}

fn create_recurring_task_impl(
    project_id: &str,
    title: &str,
    due_date: Option<&str>,
    recurrence: &str,
    interval: u32,
    tags: &[String],
) -> Result<TaskWithContent, String> {
    use chrono::Utc;

    let tasks_dir = ensure_tasks_dir(project_id)?;
    let now = Utc::now();
    // Add a small suffix to avoid filename collision with completed task
    let filename = format!("task-{}-r", now.format("%Y%m%d-%H%M%S"));
    let task_path = tasks_dir.join(format!("{}.md", filename));

    let now_str = now.to_rfc3339();
    let id = format!("{}-{}", project_id, filename);

    let mut fm = serde_yaml::Mapping::new();
    fm.insert(serde_yaml::Value::from("id"), serde_yaml::Value::from(id.clone()));
    fm.insert(serde_yaml::Value::from("type"), serde_yaml::Value::from("task"));
    fm.insert(serde_yaml::Value::from("title"), serde_yaml::Value::from(title));
    fm.insert(serde_yaml::Value::from("completed"), serde_yaml::Value::from(false));
    fm.insert(serde_yaml::Value::from("section"), serde_yaml::Value::from("Active"));
    fm.insert(serde_yaml::Value::from("priority"), serde_yaml::Value::from("normal"));
    fm.insert(serde_yaml::Value::from("is_active"), serde_yaml::Value::from(true));
    fm.insert(serde_yaml::Value::from("project_id"), serde_yaml::Value::from(project_id));
    fm.insert(serde_yaml::Value::from("recurrence"), serde_yaml::Value::from(recurrence));
    fm.insert(serde_yaml::Value::from("recurrence_interval"), serde_yaml::Value::from(interval as u64));

    if let Some(due) = due_date {
        fm.insert(serde_yaml::Value::from("due_date"), serde_yaml::Value::from(due));
    }

    if !tags.is_empty() {
        let yaml_tags: Vec<serde_yaml::Value> = tags.iter().map(|t| serde_yaml::Value::from(t.as_str())).collect();
        fm.insert(serde_yaml::Value::from("tags"), serde_yaml::Value::Sequence(yaml_tags));
    }

    fm.insert(serde_yaml::Value::from("created"), serde_yaml::Value::from(now_str.clone()));
    fm.insert(serde_yaml::Value::from("updated"), serde_yaml::Value::from(now_str.clone()));

    let body = format!("# {}\n\n", title);
    let content = frontmatter::serialize_frontmatter(&fm, &body)?;

    filesystem::atomic_write(&task_path, content.as_bytes())?;

    Ok(TaskWithContent {
        id,
        title: title.to_string(),
        completed: false,
        section: "Active".to_string(),
        priority: Some("normal".to_string()),
        due_date: due_date.map(String::from),
        is_active: true,
        tags: tags.to_vec(),
        parent_id: None,
        recurrence: Some(recurrence.to_string()),
        recurrence_interval: Some(interval),
        project_id: project_id.to_string(),
        path: format!("projects/{}/tasks/{}.md", project_id, filename),
        created: now_str.clone(),
        updated: now_str,
        content: body,
    })
}

fn update_task_meta_impl(
    project_id: &str,
    task_id: &str,
    meta: UpdateTaskMetaRequest,
) -> Result<Task, String> {
    let task_path = find_task_path(project_id, task_id)?;

    // Read existing content
    let existing = fs::read_to_string(&task_path).map_err(|e| e.to_string())?;
    let (mut fm, body, _) = frontmatter::parse_frontmatter(&existing);

    // Update fields if provided
    if let Some(title) = meta.title {
        fm.insert(
            serde_yaml::Value::from("title"),
            serde_yaml::Value::from(title),
        );
    }
    if let Some(section) = meta.section {
        fm.insert(
            serde_yaml::Value::from("section"),
            serde_yaml::Value::from(section),
        );
    }
    if let Some(priority) = meta.priority {
        fm.insert(
            serde_yaml::Value::from("priority"),
            serde_yaml::Value::from(priority),
        );
    }
    if let Some(due_date) = meta.due_date {
        fm.insert(
            serde_yaml::Value::from("due_date"),
            serde_yaml::Value::from(due_date),
        );
    }
    if let Some(is_active) = meta.is_active {
        fm.insert(
            serde_yaml::Value::from("is_active"),
            serde_yaml::Value::from(is_active),
        );
    }
    if let Some(tags) = meta.tags {
        let yaml_tags: Vec<serde_yaml::Value> =
            tags.into_iter().map(serde_yaml::Value::from).collect();
        fm.insert(
            serde_yaml::Value::from("tags"),
            serde_yaml::Value::Sequence(yaml_tags),
        );
    }
    if let Some(recurrence) = meta.recurrence {
        if recurrence.is_empty() {
            fm.remove(&serde_yaml::Value::from("recurrence"));
            fm.remove(&serde_yaml::Value::from("recurrence_interval"));
        } else {
            fm.insert(
                serde_yaml::Value::from("recurrence"),
                serde_yaml::Value::from(recurrence),
            );
        }
    }
    if let Some(interval) = meta.recurrence_interval {
        fm.insert(
            serde_yaml::Value::from("recurrence_interval"),
            serde_yaml::Value::from(interval as u64),
        );
    }

    // Update timestamp
    let now = chrono::Utc::now().to_rfc3339();
    fm.insert(
        serde_yaml::Value::from("updated"),
        serde_yaml::Value::from(now),
    );

    // Serialize and write (atomic to prevent corruption)
    let new_content = frontmatter::serialize_frontmatter(&fm, &body)?;
    filesystem::atomic_write(&task_path, new_content.as_bytes())?;

    // Return updated task
    let task = parse_task_file(&fs::read_to_string(&task_path).unwrap(), &task_path, project_id)
        .ok_or_else(|| "Failed to parse updated task".to_string())?;

    Ok(task)
}

fn delete_task_impl(project_id: &str, task_id: &str) -> Result<(), String> {
    let task_path = find_task_path(project_id, task_id)?;

    // Move to archive
    let archive_dir = config::data_dir().join("archive");
    fs::create_dir_all(&archive_dir).map_err(|e| e.to_string())?;

    let filename = task_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("task.md");

    let archive_path = archive_dir.join(format!("{}-{}", project_id, filename));
    fs::rename(&task_path, &archive_path).map_err(|e| e.to_string())?;

    Ok(())
}

fn find_task_path(project_id: &str, task_id: &str) -> Result<std::path::PathBuf, String> {
    let tasks_dir = get_tasks_dir(project_id);

    // Try direct filename match
    let direct_path = tasks_dir.join(format!("{}.md", task_id));
    if direct_path.exists() {
        return Ok(direct_path);
    }

    // Search by ID in frontmatter
    if let Ok(entries) = fs::read_dir(&tasks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(&path) {
                let (fm, _, _) = frontmatter::parse_frontmatter(&content);

                let file_id = fm
                    .get(&serde_yaml::Value::from("id"))
                    .and_then(|v| v.as_str());

                if file_id == Some(task_id) {
                    return Ok(path);
                }
            }
        }
    }

    Err("Task not found".to_string())
}

// ============ Legacy/Global Task Listing ============

async fn list_all_tasks_handler() -> impl IntoResponse {
    match list_all_tasks_impl() {
        Ok(tasks) => Json(tasks).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to list tasks: {}", err),
        )
            .into_response(),
    }
}

fn list_all_tasks_impl() -> Result<Vec<Task>, String> {
    let projects_dir = config::data_dir().join("projects");

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut all_tasks = Vec::new();

    for entry in fs::read_dir(&projects_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let project_path = entry.path();

        if !project_path.is_dir() {
            continue;
        }

        let project_id = project_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if let Ok(tasks) = list_project_tasks_impl(&project_id) {
            all_tasks.extend(tasks);
        }
    }

    // Sort all tasks by updated date descending
    // Sort by created date (stable ordering)
    all_tasks.sort_by(|a, b| b.created.cmp(&a.created));

    Ok(all_tasks)
}
