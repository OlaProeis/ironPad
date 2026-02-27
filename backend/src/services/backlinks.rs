use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

use crate::config;
use crate::services::frontmatter;

/// Represents a single link from one note to another
#[derive(Debug, Clone)]
pub struct NoteLink {
    pub source_id: String,
    pub source_title: String,
    pub source_path: String,
    pub target_id: String,
    pub target_title: Option<String>,
    pub context: String,
    pub line_number: usize,
}

/// Represents a link target for forward links
#[derive(Debug, Clone, serde::Serialize)]
pub struct ForwardLink {
    pub target_id: String,
    pub target_title: Option<String>,
    pub context: String,
    pub line_number: usize,
}

/// Represents a backlink (incoming link) with source info
#[derive(Debug, Clone, serde::Serialize)]
pub struct Backlink {
    pub source_id: String,
    pub source_title: String,
    pub source_path: String,
    pub context: String,
    pub line_number: usize,
}

/// Global link index: target_id -> list of links pointing to it
static LINK_INDEX: Lazy<Arc<Mutex<HashMap<String, Vec<NoteLink>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Note ID to title mapping for resolving titles
static NOTE_TITLES: Lazy<Arc<Mutex<HashMap<String, String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Regex to match wiki-style links: [[note-id]] or [[note-id|display text]]
/// Also handles escaped versions like \[\[note-id]] that Milkdown may produce
static WIKI_LINK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\?\[\\?\[([^\[\]]+?)(?:\|([^\[\]]*?))?\\?\]\\?\]").unwrap());

/// Regex to match markdown links that reference other notes by ID: [text](note-id)
static MARKDOWN_LINK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[([^\]]*)\]\(([^\)]+)\)").unwrap());

/// Check if a path is a note file (not tasks, prompts, etc.)
fn is_note_file(path: &Path) -> bool {
    let path_str = path.to_string_lossy();

    // Must be a markdown file
    if path.extension().and_then(|s| s.to_str()) != Some("md") {
        return false;
    }

    // Skip task files (contain "tasks/" in path)
    if path_str.contains("tasks") && !path_str.contains("notes") {
        return false;
    }

    // Skip prompt files (contain "prompts/" in path)
    if path_str.contains("prompts") {
        return false;
    }

    // Include notes folder files
    if path_str.contains("notes") && !path_str.contains("archive") {
        return true;
    }

    // Include project index.md files
    if path_str.contains("projects")
        && path.file_name().and_then(|s| s.to_str()) == Some("index.md")
    {
        return true;
    }

    // Include root-level files (index.md, inbox.md)
    if let Some(parent) = path.parent() {
        if parent == config::data_dir() {
            return true;
        }
    }

    // Include daily notes
    if path_str.contains("daily") {
        return true;
    }

    false
}

/// Normalize path for consistent storage
fn normalize_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    let stripped = if let Some(idx) = path_str.find("data") {
        &path_str[idx + 5..]
    } else {
        &path_str
    };
    stripped
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string()
}

/// Extract note ID from frontmatter or derive from path
fn extract_note_id(path: &Path, frontmatter: &serde_yaml::Mapping) -> String {
    frontmatter
        .get(&serde_yaml::Value::from("id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| frontmatter::derive_id_from_path(path))
}

/// Extract note title from frontmatter or filename
fn extract_note_title(path: &Path, frontmatter: &serde_yaml::Mapping) -> String {
    frontmatter
        .get(&serde_yaml::Value::from("title"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        })
}

/// Extract all links from note content
fn extract_links(
    content: &str,
    source_id: &str,
    source_title: &str,
    source_path: &str,
) -> Vec<NoteLink> {
    let mut links = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    tracing::debug!(
        "Extracting links from content with {} lines for note: {}",
        lines.len(),
        source_id
    );

    for (line_idx, line) in lines.iter().enumerate() {
        // Find wiki-style links [[target-id]] or [[target-id|display]]
        for cap in WIKI_LINK_REGEX.captures_iter(line) {
            let target_id = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            let display_text = cap.get(2).map(|m| m.as_str());

            if !target_id.is_empty() {
                let context = extract_context(line, &cap[0]);
                tracing::debug!("Found wiki-link: {} -> {}", target_id, source_id);
                links.push(NoteLink {
                    source_id: source_id.to_string(),
                    source_title: source_title.to_string(),
                    source_path: source_path.to_string(),
                    target_id: target_id.to_string(),
                    target_title: display_text.map(|s| s.to_string()),
                    context,
                    line_number: line_idx + 1,
                });
            }
        }

        // Find markdown links [text](target) where target looks like a note ID
        for cap in MARKDOWN_LINK_REGEX.captures_iter(line) {
            let link_text = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let target = cap.get(2).map(|m| m.as_str()).unwrap_or("");

            tracing::debug!(
                "Found markdown link candidate: [{}]({}) on line {}",
                link_text,
                target,
                line_idx + 1
            );

            // Only consider targets that look like note IDs (alphanumeric with dashes/underscores)
            // Exclude URLs (http, https, etc.)
            if !target.is_empty()
                && !target.starts_with("http")
                && !target.starts_with("//")
                && !target.starts_with("#")
                && !target.starts_with("/")
                && !target.contains('/')
                && target.len() >= 3
                && target
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                let context = extract_context(line, &cap[0]);
                tracing::info!(
                    "Extracted markdown link: {} -> {} (text: {})",
                    source_id,
                    target,
                    link_text
                );
                links.push(NoteLink {
                    source_id: source_id.to_string(),
                    source_title: source_title.to_string(),
                    source_path: source_path.to_string(),
                    target_id: target.to_string(),
                    target_title: Some(link_text.to_string()).filter(|s| !s.is_empty()),
                    context,
                    line_number: line_idx + 1,
                });
            } else {
                tracing::debug!(
                    "Skipped markdown link: [{}]({}) - doesn't match note ID pattern",
                    link_text,
                    target
                );
            }
        }
    }

    tracing::info!("Extracted {} links from note: {}", links.len(), source_id);

    links
}

/// Extract surrounding context for a link (the whole sentence or line)
fn extract_context(line: &str, link_match: &str) -> String {
    let line = line.trim();

    // Find the sentence containing the link
    let sentences: Vec<&str> = line.split(". ").collect();
    for sentence in &sentences {
        if sentence.contains(link_match) {
            return sentence.trim().to_string();
        }
    }

    // Fallback: return the whole line (truncated if too long)
    if line.len() > 200 {
        format!("{}...", &line[..200])
    } else {
        line.to_string()
    }
}

/// Rebuild the entire link index by scanning all notes
pub fn rebuild_link_index() -> Result<usize, String> {
    let root = config::data_dir();
    let mut index: HashMap<String, Vec<NoteLink>> = HashMap::new();
    let mut titles: HashMap<String, String> = HashMap::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            // Skip common ignored directories
            let path_str = e.path().to_string_lossy();
            !path_str.contains(".git")
                && !path_str.contains("assets")
                && !path_str.contains("archive")
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if !is_note_file(path) {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, body, _) = frontmatter::parse_frontmatter(&content);
        let note_id = extract_note_id(path, &fm);
        let note_title = extract_note_title(path, &fm);
        let note_path = normalize_path(path);

        // Store title mapping
        titles.insert(note_id.clone(), note_title.clone());

        // Extract links from this note
        let links = extract_links(&body, &note_id, &note_title, &note_path);

        // Add to index (grouped by target)
        for link in links {
            index.entry(link.target_id.clone()).or_default().push(link);
        }
    }

    // Update global index
    let count = {
        let mut guard = LINK_INDEX
            .lock()
            .map_err(|_| "Failed to lock link index".to_string())?;
        *guard = index;
        titles.len()
    };

    // Update titles
    {
        let mut guard = NOTE_TITLES
            .lock()
            .map_err(|_| "Failed to lock note titles".to_string())?;
        *guard = titles;
    }

    Ok(count)
}

/// Update links for a single note (called when a note is saved)
pub fn update_note_links(note_id: &str, _content: &str) {
    // Rebuild entire index for simplicity
    // In a production system, you'd incrementally update
    let _ = rebuild_link_index();
}

/// Get all backlinks for a note (notes that link TO this note)
pub fn get_backlinks(note_id: &str) -> Vec<Backlink> {
    let index = match LINK_INDEX.lock() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };

    let links = match index.get(note_id) {
        Some(links) => links,
        None => return Vec::new(),
    };

    links
        .iter()
        .map(|link| Backlink {
            source_id: link.source_id.clone(),
            source_title: link.source_title.clone(),
            source_path: link.source_path.clone(),
            context: link.context.clone(),
            line_number: link.line_number,
        })
        .collect()
}

/// Get all forward links from a note (notes this note links TO)
pub fn get_forward_links(note_id: &str) -> Vec<ForwardLink> {
    let index = match LINK_INDEX.lock() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };

    let titles = match NOTE_TITLES.lock() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };

    let mut forward_links = Vec::new();

    for (target_id, links) in index.iter() {
        for link in links {
            if link.source_id == note_id {
                let target_title = link
                    .target_title
                    .clone()
                    .or_else(|| titles.get(target_id).cloned());
                forward_links.push(ForwardLink {
                    target_id: target_id.clone(),
                    target_title,
                    context: link.context.clone(),
                    line_number: link.line_number,
                });
            }
        }
    }

    forward_links
}

/// Get all note IDs and titles for link autocompletion
pub fn get_all_note_titles() -> Vec<(String, String)> {
    let titles = match NOTE_TITLES.lock() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };

    titles
        .iter()
        .map(|(id, title)| (id.clone(), title.clone()))
        .collect()
}

/// Resolve a note ID or title to an actual note ID
pub fn resolve_note_id(query: &str) -> Option<String> {
    let titles = match NOTE_TITLES.lock() {
        Ok(guard) => guard,
        Err(_) => return None,
    };

    // Direct ID match
    if titles.contains_key(query) {
        return Some(query.to_string());
    }

    // Search by title (case-insensitive)
    for (id, title) in titles.iter() {
        if title.eq_ignore_ascii_case(query) {
            return Some(id.clone());
        }
    }

    None
}

/// Get note info by ID - searches the filesystem
pub fn get_note_info(note_id: &str) -> Option<(String, String, String)> {
    let root = config::data_dir();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let path_str = e.path().to_string_lossy();
            !path_str.contains(".git")
                && !path_str.contains("assets")
                && !path_str.contains("archive")
        })
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if !is_note_file(path) {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let (fm, _, _) = frontmatter::parse_frontmatter(&content);
        let id = extract_note_id(path, &fm);

        if id == note_id {
            let title = extract_note_title(path, &fm);
            let path_str = normalize_path(path);
            return Some((id, title, path_str));
        }
    }

    None
}

/// Search notes by partial title match for autocompletion
pub fn search_note_titles(query: &str, limit: usize) -> Vec<(String, String)> {
    let titles = match NOTE_TITLES.lock() {
        Ok(guard) => guard,
        Err(_) => return Vec::new(),
    };

    let query_lower = query.to_lowercase();

    titles
        .iter()
        .filter(|(id, title)| {
            id.to_lowercase().contains(&query_lower) || title.to_lowercase().contains(&query_lower)
        })
        .take(limit)
        .map(|(id, title)| (id.clone(), title.clone()))
        .collect()
}

/// Get link stats for debugging/monitoring
pub fn get_link_stats() -> (usize, usize) {
    let index = match LINK_INDEX.lock() {
        Ok(guard) => guard,
        Err(_) => return (0, 0),
    };

    let total_links: usize = index.values().map(|v| v.len()).sum();
    let unique_targets = index.len();

    (total_links, unique_targets)
}
