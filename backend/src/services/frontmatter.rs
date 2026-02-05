use std::path::Path;

use chrono::Utc;
use serde_yaml::{Mapping, Value};

/// Derive deterministic ID from file path.
/// Matches filesystem ID logic: strips data directory prefix and folder name.
pub fn derive_id_from_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();

    // Find "data" in the path and strip everything before and including it
    let rel_str = if let Some(idx) = path_str.find("data") {
        &path_str[idx + 5..] // Skip "data" + separator
    } else {
        &path_str
    };

    // Split by both forward and back slashes, filter empty parts
    let mut parts: Vec<String> = rel_str
        .split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .map(|s| s.replace(".md", ""))
        .collect();

    // Drop top-level folder name (notes, projects, etc.) if we have multiple parts
    if parts.len() > 1 {
        parts.remove(0);
    }

    parts.join("-")
}

/// Parse frontmatter from file content.
/// Returns (frontmatter mapping, body content, has_frontmatter flag).
pub fn parse_frontmatter(content: &str) -> (Mapping, String, bool) {
    if !content.starts_with("---") {
        return (Mapping::new(), content.to_string(), false);
    }

    let mut parts = content.splitn(3, "---");
    parts.next(); // empty before first ---
    let yaml = parts.next().unwrap_or("");
    let body = parts.next().unwrap_or("");

    let fm: Value = serde_yaml::from_str(yaml).unwrap_or(Value::Null);
    let map = fm.as_mapping().cloned().unwrap_or_default();

    (map, body.to_string(), true)
}

/// Serialize frontmatter and body back to markdown string.
pub fn serialize_frontmatter(frontmatter: &Mapping, body: &str) -> Result<String, String> {
    let yaml = serde_yaml::to_string(frontmatter).map_err(|e| e.to_string())?;

    let mut content = String::new();
    content.push_str("---\n");
    content.push_str(&yaml);
    content.push_str("---\n\n");
    content.push_str(body.trim_start());

    Ok(content)
}

/// Generate initial frontmatter for a newly created file.
/// Sets backend-owned fields only.
pub fn generate_frontmatter(path: &Path, note_type: &str) -> Mapping {
    let mut map = Mapping::new();

    let id = derive_id_from_path(path);
    let now = Utc::now().to_rfc3339();

    map.insert(Value::from("id"), Value::from(id));
    map.insert(Value::from("type"), Value::from(note_type));
    map.insert(Value::from("created"), Value::from(now.clone()));
    map.insert(Value::from("updated"), Value::from(now));

    map
}

/// Ensure frontmatter has all required backend-owned fields.
/// - If `id` is missing, derive from path
/// - If `created` is missing, set to now
/// - Always updates `updated` timestamp
/// - Preserves all user-defined fields (title, tags, status, etc.)
pub fn ensure_frontmatter(existing: &mut Mapping, path: &Path) {
    let now = Utc::now().to_rfc3339();

    // Ensure ID exists (derive from path if missing)
    if !existing.contains_key(&Value::from("id")) {
        let id = derive_id_from_path(path);
        existing.insert(Value::from("id"), Value::from(id));
    }

    // Ensure created timestamp exists (set once, never overwritten)
    if !existing.contains_key(&Value::from("created")) {
        existing.insert(Value::from("created"), Value::from(now.clone()));
    }

    // Always update the updated timestamp
    existing.insert(Value::from("updated"), Value::from(now));
}

/// Update frontmatter on save.
/// Only updates `updated` timestamp, preserves all other fields.
pub fn update_frontmatter(existing: &mut Mapping) {
    let now = Utc::now().to_rfc3339();
    existing.insert(Value::from("updated"), Value::from(now));
}

/// Check if frontmatter has all required backend-owned fields.
pub fn is_frontmatter_complete(frontmatter: &Mapping) -> bool {
    frontmatter.contains_key(&Value::from("id"))
        && frontmatter.contains_key(&Value::from("created"))
        && frontmatter.contains_key(&Value::from("updated"))
}

// ============ Helper functions for cleaner frontmatter field access ============

/// Get a string value from frontmatter by key.
pub fn get_str(fm: &Mapping, key: &str) -> Option<String> {
    fm.get(&Value::from(key))
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Get a string value from frontmatter, with a default fallback.
pub fn get_str_or(fm: &Mapping, key: &str, default: &str) -> String {
    get_str(fm, key).unwrap_or_else(|| default.to_string())
}

/// Get a bool value from frontmatter by key.
pub fn get_bool(fm: &Mapping, key: &str) -> Option<bool> {
    fm.get(&Value::from(key)).and_then(|v| v.as_bool())
}

/// Get a bool value from frontmatter, with a default fallback.
pub fn get_bool_or(fm: &Mapping, key: &str, default: bool) -> bool {
    get_bool(fm, key).unwrap_or(default)
}

/// Get a u64 value from frontmatter by key.
pub fn get_u64(fm: &Mapping, key: &str) -> Option<u64> {
    fm.get(&Value::from(key)).and_then(|v| v.as_u64())
}

/// Get a string sequence (tags, etc.) from frontmatter by key.
pub fn get_string_seq(fm: &Mapping, key: &str) -> Vec<String> {
    fm.get(&Value::from(key))
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_with_frontmatter() {
        let content = "---\nid: test\ntitle: Test Note\n---\n\nBody content";
        let (fm, body, has_fm) = parse_frontmatter(content);

        assert!(has_fm);
        assert_eq!(fm.get(&Value::from("id")).unwrap().as_str().unwrap(), "test");
        assert_eq!(fm.get(&Value::from("title")).unwrap().as_str().unwrap(), "Test Note");
        assert!(body.contains("Body content"));
    }

    #[test]
    fn test_parse_frontmatter_without_frontmatter() {
        let content = "Just some content without frontmatter";
        let (fm, body, has_fm) = parse_frontmatter(content);

        assert!(!has_fm);
        assert!(fm.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn test_derive_id_from_path() {
        let path = Path::new("data/notes/my-note.md");
        assert_eq!(derive_id_from_path(path), "my-note");

        let path = Path::new("data/projects/myproject/index.md");
        assert_eq!(derive_id_from_path(path), "myproject-index");
    }
}
