use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use serde::Serialize;
use walkdir::WalkDir;

use crate::config;

/// Search result item
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub title: String,
    pub matches: Vec<SearchMatch>,
}

/// Individual match within a file
#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub line_number: u32,
    pub line_content: String,
}

/// Search notes using simple string matching
/// Falls back to manual search if ripgrep is not available
pub fn search_notes(query: &str) -> Result<Vec<SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    // Try ripgrep first (faster)
    match search_with_ripgrep(query) {
        Ok(results) => return Ok(results),
        Err(e) => {
            tracing::debug!(
                "ripgrep not available, falling back to manual search: {}",
                e
            );
        }
    }

    // Fallback to manual search
    search_manual(query)
}

/// Search using ripgrep (rg)
fn search_with_ripgrep(query: &str) -> Result<Vec<SearchResult>, String> {
    let data_dir_str = config::data_dir().to_string_lossy();
    let output = Command::new("rg")
        .args([
            "--json",        // JSON output for parsing
            "--ignore-case", // Case insensitive
            "--type",
            "md", // Only markdown files
            "--max-count",
            "5", // Max 5 matches per file
            query,
            &data_dir_str,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("Failed to run ripgrep: {}", e))?;

    if !output.status.success() && output.stdout.is_empty() {
        // No matches found or error
        return Ok(Vec::new());
    }

    parse_ripgrep_output(&output.stdout)
}

/// Parse ripgrep JSON output
fn parse_ripgrep_output(output: &[u8]) -> Result<Vec<SearchResult>, String> {
    use std::collections::HashMap;

    let output_str = String::from_utf8_lossy(output);
    let mut results_map: HashMap<String, SearchResult> = HashMap::new();

    for line in output_str.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if json["type"] == "match" {
                let data = &json["data"];
                let path_str = data["path"]["text"].as_str().unwrap_or("");
                let line_number = data["line_number"].as_u64().unwrap_or(0) as u32;
                let line_content = data["lines"]["text"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string();

                let normalized_path = normalize_path(path_str);
                let title = extract_title_from_path(&normalized_path);

                let result = results_map
                    .entry(normalized_path.clone())
                    .or_insert_with(|| SearchResult {
                        path: normalized_path,
                        title,
                        matches: Vec::new(),
                    });

                result.matches.push(SearchMatch {
                    line_number,
                    line_content,
                });
            }
        }
    }

    Ok(results_map.into_values().collect())
}

/// Manual search fallback (no external dependencies)
fn search_manual(query: &str) -> Result<Vec<SearchResult>, String> {
    let query_lower = query.to_lowercase();
    let root = config::data_dir();
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path()))
        .filter_map(Result::ok)
    {
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut matches = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&query_lower) {
                matches.push(SearchMatch {
                    line_number: (i + 1) as u32,
                    line_content: line.trim().to_string(),
                });

                // Limit matches per file
                if matches.len() >= 5 {
                    break;
                }
            }
        }

        if !matches.is_empty() {
            let normalized_path = normalize_path(&path.to_string_lossy());
            let title = extract_title_from_path(&normalized_path);

            results.push(SearchResult {
                path: normalized_path,
                title,
                matches,
            });
        }
    }

    Ok(results)
}

fn is_ignored(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some(".git") | Some("assets") | Some("archive")
        )
    })
}

fn normalize_path(path: &str) -> String {
    if let Some(idx) = path.find("data") {
        let stripped = &path[idx + 5..];
        return stripped
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
    }
    path.replace('\\', "/")
}

fn extract_title_from_path(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
}
