use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use walkdir::{DirEntry, WalkDir};

use crate::config;
use crate::models::prompt::{Prompt, PromptFolder, PromptSearchResult, PromptSummary};
use crate::services::{filesystem, frontmatter};

#[derive(Debug, Deserialize, Clone)]
pub struct PromptListQuery {
    pub scope: Option<String>,
    pub project_id: Option<String>,
    pub folder: Option<String>,
    pub tag: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PromptCreateRequest {
    pub scope: Option<String>,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub folder: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PromptUpdateRequest {
    pub title: Option<String>,
    pub folder: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PromptSearchQuery {
    pub q: String,
    pub scope: Option<String>,
    pub project_id: Option<String>,
    pub folder: Option<String>,
    pub tag: Option<String>,
    pub limit: Option<usize>,
}

fn normalize_folder(folder: Option<&str>) -> String {
    let normalized = folder
        .unwrap_or("")
        .replace('\\', "/")
        .trim()
        .trim_matches('/')
        .to_string();
    if normalized.is_empty() {
        "root".to_string()
    } else {
        normalized
    }
}

fn normalize_scope(scope: Option<&str>) -> String {
    match scope.unwrap_or("global").trim().to_lowercase().as_str() {
        "project" => "project".to_string(),
        _ => "global".to_string(),
    }
}

fn normalize_tags(tags: Option<Vec<String>>) -> Vec<String> {
    let mut unique = BTreeSet::new();
    for tag in tags.unwrap_or_default() {
        let t = tag.trim().to_lowercase();
        if !t.is_empty() {
            unique.insert(t);
        }
    }
    unique.into_iter().collect()
}

fn slugify(input: &str) -> String {
    let slug = input
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if slug.is_empty() {
        "prompt".to_string()
    } else {
        slug
    }
}

fn prompt_root(scope: &str, project_id: Option<&str>) -> Result<PathBuf, String> {
    if scope == "project" {
        let pid = project_id
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| "project_id is required for project scope".to_string())?;
        let project_dir = config::data_dir().join("projects").join(pid);
        if !project_dir.exists() {
            return Err(format!("Project not found: {}", pid));
        }
        Ok(project_dir.join("prompts"))
    } else {
        Ok(config::data_dir().join("prompts"))
    }
}

fn list_prompt_roots() -> Vec<(String, Option<String>, PathBuf)> {
    let mut roots = Vec::new();
    roots.push((
        "global".to_string(),
        None,
        config::data_dir().join("prompts"),
    ));

    let projects_dir = config::data_dir().join("projects");
    if let Ok(entries) = fs::read_dir(projects_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Some(pid) = path.file_name().and_then(|s| s.to_str()) {
                roots.push((
                    "project".to_string(),
                    Some(pid.to_string()),
                    path.join("prompts"),
                ));
            }
        }
    }

    roots
}

fn is_ignored(entry: &DirEntry) -> bool {
    entry.path().components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some(".git") | Some("assets") | Some("archive")
        )
    })
}

fn build_summary(
    path: &Path,
    default_scope: &str,
    default_project_id: Option<String>,
) -> Result<PromptSummary, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (fm, _body, _has_fm) = frontmatter::parse_frontmatter(&raw);

    let id =
        frontmatter::get_str(&fm, "id").unwrap_or_else(|| frontmatter::derive_id_from_path(path));
    let title = frontmatter::get_str(&fm, "title").unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled Prompt")
            .to_string()
    });
    let scope = frontmatter::get_str(&fm, "scope").unwrap_or_else(|| default_scope.to_string());
    let project_id = frontmatter::get_str(&fm, "project_id").or(default_project_id);
    let folder = normalize_folder(frontmatter::get_str(&fm, "folder").as_deref());
    let tags = frontmatter::get_string_seq(&fm, "tags");
    let description = frontmatter::get_str(&fm, "description");
    let updated = frontmatter::get_str(&fm, "updated");

    Ok(PromptSummary {
        id,
        title,
        path: filesystem::normalize_path(path),
        scope,
        project_id,
        folder,
        tags,
        description,
        updated,
    })
}

fn build_prompt(
    path: &Path,
    default_scope: &str,
    default_project_id: Option<String>,
) -> Result<Prompt, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (fm, body, _has_fm) = frontmatter::parse_frontmatter(&raw);

    let summary = build_summary(path, default_scope, default_project_id)?;

    Ok(Prompt {
        id: summary.id,
        title: summary.title,
        path: summary.path,
        scope: summary.scope,
        project_id: summary.project_id,
        folder: summary.folder,
        tags: summary.tags,
        description: summary.description,
        frontmatter: fm,
        content: body.trim_start().to_string(),
    })
}

fn prompt_matches_filters(prompt: &PromptSummary, query: &PromptListQuery) -> bool {
    if let Some(scope) = &query.scope {
        if prompt.scope != normalize_scope(Some(scope.as_str())) {
            return false;
        }
    }
    if let Some(project_id) = &query.project_id {
        if prompt.project_id.as_deref() != Some(project_id.as_str()) {
            return false;
        }
    }
    if let Some(folder) = &query.folder {
        if prompt.folder != normalize_folder(Some(folder)) {
            return false;
        }
    }
    if let Some(tag) = &query.tag {
        let wanted = tag.trim().to_lowercase();
        if !prompt.tags.iter().any(|t| t == &wanted) {
            return false;
        }
    }
    if let Some(q) = &query.q {
        let ql = q.trim().to_lowercase();
        if !ql.is_empty() {
            let tags = prompt.tags.join(" ");
            let description = prompt.description.clone().unwrap_or_default();
            let hay = format!(
                "{} {} {} {}",
                prompt.title, prompt.folder, tags, description
            )
            .to_lowercase();
            if !hay.contains(&ql) {
                return false;
            }
        }
    }
    true
}

pub fn list_prompts(query: &PromptListQuery) -> Result<Vec<PromptSummary>, String> {
    let mut prompts = Vec::new();

    for (scope, project_id, root) in list_prompt_roots() {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(&root)
            .into_iter()
            .filter_entry(|e| !is_ignored(e))
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            match build_summary(path, &scope, project_id.clone()) {
                Ok(summary) => {
                    if prompt_matches_filters(&summary, query) {
                        prompts.push(summary);
                    }
                }
                Err(err) => tracing::warn!("Skipping prompt {:?}: {}", path, err),
            }
        }
    }

    prompts.sort_by(|a, b| b.updated.cmp(&a.updated));
    Ok(prompts)
}

fn find_prompt_path(prompt_id: &str) -> Option<(String, Option<String>, PathBuf)> {
    for (scope, project_id, root) in list_prompt_roots() {
        if !root.exists() {
            continue;
        }
        for entry in WalkDir::new(&root)
            .into_iter()
            .filter_entry(|e| !is_ignored(e))
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }

            if let Ok(raw) = fs::read_to_string(path) {
                let (fm, _, _) = frontmatter::parse_frontmatter(&raw);
                let id = frontmatter::get_str(&fm, "id")
                    .unwrap_or_else(|| frontmatter::derive_id_from_path(path));
                if id == prompt_id {
                    return Some((scope, project_id, path.to_path_buf()));
                }
            }
        }
    }
    None
}

pub fn get_prompt(prompt_id: &str) -> Result<Prompt, String> {
    let (scope, project_id, path) =
        find_prompt_path(prompt_id).ok_or_else(|| format!("Prompt not found: {}", prompt_id))?;
    build_prompt(&path, &scope, project_id)
}

pub fn create_prompt(req: PromptCreateRequest) -> Result<Prompt, String> {
    let scope = normalize_scope(req.scope.as_deref());
    let project_id = req
        .project_id
        .as_deref()
        .map(|s| s.trim())
        .map(String::from);
    let root = prompt_root(&scope, project_id.as_deref())?;
    let folder = normalize_folder(req.folder.as_deref());
    let folder_dir = if folder == "root" {
        root
    } else {
        root.join(folder.replace('/', std::path::MAIN_SEPARATOR_STR))
    };
    fs::create_dir_all(&folder_dir).map_err(|e| e.to_string())?;

    let title = req
        .title
        .unwrap_or_else(|| "Untitled Prompt".to_string())
        .trim()
        .to_string();
    let slug = slugify(&title);
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let file_path = folder_dir.join(format!("{}-{}.md", stamp, slug));

    let mut fm = Mapping::new();
    let now = Utc::now().to_rfc3339();
    fm.insert(
        Value::from("id"),
        Value::from(frontmatter::derive_id_from_path(&file_path)),
    );
    fm.insert(Value::from("type"), Value::from("prompt"));
    fm.insert(Value::from("scope"), Value::from(scope.clone()));
    if let Some(pid) = &project_id {
        fm.insert(Value::from("project_id"), Value::from(pid.clone()));
    }
    fm.insert(Value::from("title"), Value::from(title));
    fm.insert(Value::from("folder"), Value::from(folder));
    fm.insert(
        Value::from("description"),
        Value::from(req.description.unwrap_or_default()),
    );
    fm.insert(
        Value::from("tags"),
        Value::Sequence(
            normalize_tags(req.tags)
                .into_iter()
                .map(Value::from)
                .collect(),
        ),
    );
    fm.insert(Value::from("created"), Value::from(now.clone()));
    fm.insert(Value::from("updated"), Value::from(now));

    let content = frontmatter::serialize_frontmatter(&fm, req.content.as_deref().unwrap_or(""))?;
    filesystem::atomic_write(&file_path, content.as_bytes())?;

    build_prompt(&file_path, &scope, project_id)
}

pub fn update_prompt(prompt_id: &str, req: PromptUpdateRequest) -> Result<Prompt, String> {
    let (scope, project_id, path) =
        find_prompt_path(prompt_id).ok_or_else(|| format!("Prompt not found: {}", prompt_id))?;

    let existing = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let (mut fm, old_body, has_fm) = frontmatter::parse_frontmatter(&existing);
    if !has_fm || !frontmatter::is_frontmatter_complete(&fm) {
        frontmatter::ensure_frontmatter(&mut fm, &path);
    } else {
        frontmatter::update_frontmatter(&mut fm);
    }

    if let Some(title) = req.title {
        fm.insert(Value::from("title"), Value::from(title.trim().to_string()));
    }
    if let Some(folder) = req.folder {
        fm.insert(
            Value::from("folder"),
            Value::from(normalize_folder(Some(&folder))),
        );
    }
    if let Some(description) = req.description {
        fm.insert(Value::from("description"), Value::from(description));
    }
    if let Some(tags) = req.tags {
        fm.insert(
            Value::from("tags"),
            Value::Sequence(
                normalize_tags(Some(tags))
                    .into_iter()
                    .map(Value::from)
                    .collect(),
            ),
        );
    }

    let content_body = req
        .content
        .unwrap_or_else(|| old_body.trim_start().to_string());
    let rebuilt = frontmatter::serialize_frontmatter(&fm, &content_body)?;
    filesystem::atomic_write(&path, rebuilt.as_bytes())?;

    build_prompt(&path, &scope, project_id)
}

pub fn delete_prompt(prompt_id: &str) -> Result<(), String> {
    let (_scope, _project_id, path) =
        find_prompt_path(prompt_id).ok_or_else(|| format!("Prompt not found: {}", prompt_id))?;
    fs::remove_file(path).map_err(|e| e.to_string())
}

pub fn list_folders(
    scope: Option<&str>,
    project_id: Option<&str>,
) -> Result<Vec<PromptFolder>, String> {
    let query = PromptListQuery {
        scope: scope.map(|s| s.to_string()),
        project_id: project_id.map(|s| s.to_string()),
        folder: None,
        tag: None,
        q: None,
    };
    let prompts = list_prompts(&query)?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for p in prompts {
        *counts.entry(p.folder).or_insert(0) += 1;
    }
    Ok(counts
        .into_iter()
        .map(|(path, count)| PromptFolder { path, count })
        .collect())
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() > 1)
        .map(String::from)
        .collect()
}

fn semantic_expansions(token: &str) -> &'static [&'static str] {
    match token {
        "write" | "writing" | "draft" => &["compose", "generate", "author"],
        "summary" | "summarize" | "brief" => &["synopsis", "recap", "outline"],
        "refactor" | "cleanup" => &["improve", "simplify", "restructure"],
        "plan" | "roadmap" => &["strategy", "steps", "approach"],
        "review" => &["critique", "analyze", "feedback"],
        "bug" | "fix" => &["issue", "repair", "debug"],
        _ => &[],
    }
}

fn semantic_score(query: &str, prompt: &Prompt) -> (f32, Vec<String>) {
    let query_terms = tokenize(query);
    if query_terms.is_empty() {
        return (0.0, Vec::new());
    }

    let mut expanded_terms = HashSet::new();
    for t in &query_terms {
        expanded_terms.insert(t.clone());
        for extra in semantic_expansions(t) {
            expanded_terms.insert((*extra).to_string());
        }
    }

    let title_tokens: HashSet<String> = tokenize(&prompt.title).into_iter().collect();
    let tag_tokens: HashSet<String> = prompt.tags.iter().map(|t| t.to_lowercase()).collect();
    let body_tokens: HashSet<String> = tokenize(&prompt.content).into_iter().collect();
    let desc_tokens: HashSet<String> = tokenize(prompt.description.as_deref().unwrap_or(""))
        .into_iter()
        .collect();

    let mut matched = BTreeSet::new();
    let mut title_hits = 0.0f32;
    let mut tag_hits = 0.0f32;
    let mut body_hits = 0.0f32;
    let mut desc_hits = 0.0f32;

    for term in &expanded_terms {
        if title_tokens.contains(term) {
            title_hits += 1.0;
            matched.insert(term.clone());
        }
        if tag_tokens.contains(term) {
            tag_hits += 1.0;
            matched.insert(term.clone());
        }
        if body_tokens.contains(term) {
            body_hits += 1.0;
            matched.insert(term.clone());
        }
        if desc_tokens.contains(term) {
            desc_hits += 1.0;
            matched.insert(term.clone());
        }
    }

    let denom = expanded_terms.len() as f32;
    if denom == 0.0 {
        return (0.0, Vec::new());
    }

    let mut score = (title_hits / denom) * 0.40
        + (tag_hits / denom) * 0.25
        + (desc_hits / denom) * 0.20
        + (body_hits / denom) * 0.15;

    if prompt
        .content
        .to_lowercase()
        .contains(&query.to_lowercase())
    {
        score += 0.15;
    }

    (score.min(1.0), matched.into_iter().collect())
}

pub fn semantic_search(query: &PromptSearchQuery) -> Result<Vec<PromptSearchResult>, String> {
    let list_query = PromptListQuery {
        scope: query.scope.clone(),
        project_id: query.project_id.clone(),
        folder: query.folder.clone(),
        tag: query.tag.clone(),
        q: None,
    };
    let prompts = list_prompts(&list_query)?;
    let mut results = Vec::new();
    let max = query.limit.unwrap_or(25).min(100);

    for item in prompts {
        let full = get_prompt(&item.id)?;
        let (score, matched_terms) = semantic_score(&query.q, &full);
        if score > 0.01 {
            results.push(PromptSearchResult {
                prompt: item,
                score,
                matched_terms,
            });
        }
    }

    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    results.truncate(max);
    Ok(results)
}
