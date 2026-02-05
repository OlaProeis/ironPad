use std::fs;
use std::io::Write;
use std::path::Path;

use serde_yaml::Value;
use walkdir::WalkDir;

use crate::models::note::{Note, NoteSummary};
use crate::services::frontmatter;

use crate::config;

/// List all notes in the filesystem (read-only).
pub fn list_notes() -> Result<Vec<NoteSummary>, String> {
    let mut notes = Vec::new();
    let root = config::data_dir();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        // Only include notes and project index files
        if !is_note_file(path) {
            continue;
        }

        match parse_note_summary(path) {
            Ok(note) => notes.push(note),
            Err(err) => {
                tracing::warn!("Skipping file {:?}: {}", path, err);
            }
        }
    }

    Ok(notes)
}

fn is_ignored(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some(".git") | Some("assets") | Some("archive")
        )
    })
}

fn is_note_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // data/notes/**/*.md (handles both forward and back slashes)
    if path_str.contains("notes") && !path_str.contains("archive") {
        return true;
    }

    // data/projects/*/index.md
    if path_str.contains("projects") && path.file_name().and_then(|s| s.to_str()) == Some("index.md") {
        return true;
    }

    // Root-level files (index.md, inbox.md) - parent is the data dir
    if let Some(parent) = path.parent() {
        if parent == config::data_dir() {
            return true;
        }
    }

    false
}

fn parse_note_summary(path: &Path) -> Result<NoteSummary, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (fm, _body, _has_fm) = frontmatter::parse_frontmatter(&content);

    let id = fm
        .get(&Value::from("id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| frontmatter::derive_id_from_path(path));

    let title = fm
        .get(&Value::from("title"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

    let note_type = fm
        .get(&Value::from("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("note")
        .to_string();

    let updated = fm
        .get(&Value::from("updated"))
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(NoteSummary {
        id,
        title,
        path: normalize_path(path),
        note_type,
        updated,
    })
}

pub fn normalize_path(path: &Path) -> String {
    // Strip the data directory prefix and normalize separators
    let path_str = path.to_string_lossy();
    let stripped = if let Some(idx) = path_str.find("data") {
        &path_str[idx + 5..] // Skip "data" + separator
    } else {
        &path_str
    };
    stripped.replace('\\', "/").trim_start_matches('/').to_string()
}

/// Read a full note by deterministic ID.
pub fn read_note_by_id(note_id: &str) -> Result<Note, String> {
    let root = config::data_dir();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        if !is_note_file(path) {
            continue;
        }

        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let (fm, body, _has_fm) = frontmatter::parse_frontmatter(&content);

        let derived_id = fm
            .get(&Value::from("id"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| frontmatter::derive_id_from_path(path));

        if derived_id != note_id {
            continue;
        }

        let note_type = fm
            .get(&Value::from("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("note")
            .to_string();

        return Ok(Note {
            id: derived_id,
            path: normalize_path(path),
            note_type,
            frontmatter: fm,
            content: body.trim_start().to_string(),
        });
    }

    Err(format!("Note not found: {}", note_id))
}

/// Create a new empty note in data/notes/.
pub fn create_note() -> Result<Note, String> {
    use chrono::Utc;

    let dir = config::data_dir().join("notes");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let filename = format!("{}.md", Utc::now().format("%Y%m%d-%H%M%S"));
    let path = dir.join(&filename);

    let fm = frontmatter::generate_frontmatter(&path, "note");
    let content = frontmatter::serialize_frontmatter(&fm, "")?;

    // Atomic write: write to temp file, then rename
    atomic_write(&path, content.as_bytes())?;

    let id = fm
        .get(&Value::from("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(Note {
        id,
        path: normalize_path(&path),
        note_type: "note".to_string(),
        frontmatter: fm,
        content: String::new(),
    })
}

/// Update an existing note by ID with full markdown payload.
/// Handles notes with or without existing frontmatter.
/// Preserves user-defined fields, updates backend-owned fields.
pub fn update_note(note_id: &str, new_content: &str) -> Result<Note, String> {
    let root = config::data_dir();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        if !is_note_file(path) {
            continue;
        }

        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let (mut fm, _old_body, has_fm) = frontmatter::parse_frontmatter(&content);

        let derived_id = fm
            .get(&Value::from("id"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| frontmatter::derive_id_from_path(path));

        if derived_id != note_id {
            continue;
        }

        // Ensure frontmatter has all required fields
        // This handles files without frontmatter or with incomplete frontmatter
        if !has_fm || !frontmatter::is_frontmatter_complete(&fm) {
            frontmatter::ensure_frontmatter(&mut fm, path);
        } else {
            // Just update the timestamp
            frontmatter::update_frontmatter(&mut fm);
        }

        // Rebuild file content
        let rebuilt = frontmatter::serialize_frontmatter(&fm, new_content.trim_start())?;

        // Atomic write
        atomic_write(path, rebuilt.as_bytes())?;

        let note_type = fm
            .get(&Value::from("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("note")
            .to_string();

        return Ok(Note {
            id: derived_id,
            path: normalize_path(path),
            note_type,
            frontmatter: fm,
            content: new_content.to_string(),
        });
    }

    Err(format!("Note not found: {}", note_id))
}

/// Archive a note by ID (move to data/archive/).
pub fn archive_note(note_id: &str) -> Result<(), String> {
    let root = config::data_dir();
    let archive_dir = config::data_dir().join("archive");

    fs::create_dir_all(&archive_dir).map_err(|e| e.to_string())?;

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        if !is_note_file(path) {
            continue;
        }

        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let (fm, _, _) = frontmatter::parse_frontmatter(&content);

        let derived_id = fm
            .get(&Value::from("id"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| frontmatter::derive_id_from_path(path));

        if derived_id != note_id {
            continue;
        }

        let filename = path.file_name().ok_or("Invalid filename")?;
        let target = archive_dir.join(filename);

        fs::rename(path, target).map_err(|e| e.to_string())?;
        return Ok(());
    }

    Err(format!("Note not found: {}", note_id))
}

/// Atomic write: write to temp file, then rename.
/// This prevents data loss on crash or power failure.
/// Also marks the file as recently saved to avoid triggering external edit notifications.
pub fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("Invalid path")?;
    let temp_name = format!(
        ".{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
    );
    let temp_path = parent.join(temp_name);

    // Mark this file as being saved by us (to avoid triggering external edit notification)
    let normalized = normalize_path(path);
    crate::watcher::mark_file_saved(&normalized);

    // Write to temp file
    let mut file = fs::File::create(&temp_path).map_err(|e| e.to_string())?;
    file.write_all(contents).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);

    // Rename temp file to target (atomic on most filesystems)
    fs::rename(&temp_path, path).map_err(|e| e.to_string())?;

    Ok(())
}
