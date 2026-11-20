use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use walkdir::{DirEntry, WalkDir};

use crate::config;
use crate::models::prompt::{
    Prompt, PromptFolder, PromptGetResponse, PromptKind, PromptPointerFields, PromptSearchResult,
    PromptSummary,
};
use crate::services::{filesystem, frontmatter, library};

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
    pub kind: Option<PromptKind>,
    pub source: Option<String>,
    pub template: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PromptUpdateRequest {
    pub title: Option<String>,
    pub folder: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub kind: Option<PromptKind>,
    pub source: Option<String>,
    pub template: Option<bool>,
}

/// Whether a prompt service error should map to HTTP 400 in route handlers.
pub fn is_client_error(err: &str) -> bool {
    err.contains("is required")
        || err.contains("not found:")
        || err.contains("cannot be changed")
        || err.contains("link source")
        || err.contains("Project link")
        || err.contains("Global link")
}

const MSG_KIND_IMMUTABLE: &str = "Prompt kind cannot be changed.";
const MSG_LINK_SOURCE_REQUIRED: &str = "source is required for link prompts";

fn normalize_link_source(scope: &str, source: &str) -> Result<String, String> {
    let source = library::sanitize_user_path(source);
    if source.is_empty() {
        return Err(MSG_LINK_SOURCE_REQUIRED.to_string());
    }
    if scope == "project" && library::is_absolute_user_path(&source) {
        library::validate_global_source(&source).map_err(|e| e.user_message())?;
        return Ok(source);
    }
    if scope == "project" {
        library::validate_project_source(&source).map_err(|e| e.user_message())?;
    } else {
        library::validate_global_source(&source).map_err(|e| e.user_message())?;
    }
    Ok(source)
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
    let pointer = PromptPointerFields::from_mapping(&fm);

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
        kind: pointer.kind,
        source: pointer.source,
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

    let pointer = PromptPointerFields::from_mapping(&fm);

    Ok(Prompt {
        id: summary.id,
        title: summary.title,
        path: summary.path,
        scope: summary.scope,
        project_id: summary.project_id,
        folder: summary.folder,
        tags: summary.tags,
        description: summary.description,
        kind: pointer.kind,
        source: pointer.source.clone(),
        template: pointer.template,
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
            let mut hay = format!(
                "{} {} {} {}",
                prompt.title, prompt.folder, tags, description
            );
            if prompt.kind == PromptKind::Link {
                if let Some(source) = &prompt.source {
                    hay.push(' ');
                    hay.push_str(source);
                }
            }
            if !hay.to_lowercase().contains(&ql) {
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

/// Single-item GET: prompts return stored body; links return the source file.
/// Soft link failures stay `Ok` so the route can respond 200 and keep the pointer editable.
pub fn get_prompt_for_read(
    prompt_id: &str,
    root: Option<&Path>,
) -> Result<PromptGetResponse, String> {
    let prompt = get_prompt(prompt_id)?;
    if prompt.kind != PromptKind::Link {
        return Ok(PromptGetResponse {
            prompt,
            read_only: false,
            source_missing: false,
            resolved_path: None,
            supports_rendered: None,
            root_required: false,
            source_error: None,
        });
    }

    let scope = if prompt.scope == "project" {
        library::Scope::Project
    } else {
        library::Scope::Global
    };
    let stored_source = prompt.source.clone().unwrap_or_default();
    let store = library::PromptFileStore::from_config();

    let mut response = PromptGetResponse {
        prompt,
        read_only: true,
        source_missing: false,
        resolved_path: None,
        supports_rendered: None,
        root_required: false,
        source_error: None,
    };

    let resolved = match library::resolve_link_by_id(&store, prompt_id, root) {
        Ok(path) => path,
        Err(library::LibraryError::SourceMissing) => {
            response.prompt.content.clear();
            response.source_missing = true;
            response.resolved_path =
                Some(library::unresolved_link_path(scope, &stored_source, root));
            response.supports_rendered =
                Some(library::supports_rendered(Path::new(&stored_source)));
            return Ok(response);
        }
        Err(library::LibraryError::RootRequired) => {
            response.prompt.content.clear();
            response.root_required = true;
            response.resolved_path =
                Some(library::unresolved_link_path(scope, &stored_source, root));
            response.supports_rendered =
                Some(library::supports_rendered(Path::new(&stored_source)));
            return Ok(response);
        }
        Err(err @ library::LibraryError::TooLarge { .. })
        | Err(err @ library::LibraryError::DisallowedSuffix(_))
        | Err(err @ library::LibraryError::NotUtf8)
        | Err(err @ library::LibraryError::NotRegularFile)
        | Err(err @ library::LibraryError::OutsideRoot)
        | Err(err @ library::LibraryError::InvalidSource(_)) => {
            response.prompt.content.clear();
            response.source_error = Some(err.user_message());
            response.resolved_path =
                Some(library::unresolved_link_path(scope, &stored_source, root));
            return Ok(response);
        }
        Err(library::LibraryError::Io(err)) => {
            return Err(format!("Failed to read prompt: {err}"));
        }
    };

    match library::read_linked_file(&resolved) {
        Ok(text) => {
            response.supports_rendered = Some(library::supports_rendered(&resolved));
            response.resolved_path = Some(resolved.to_string_lossy().into_owned());
            response.prompt.content = text;
            Ok(response)
        }
        Err(err @ library::LibraryError::TooLarge { .. })
        | Err(err @ library::LibraryError::DisallowedSuffix(_))
        | Err(err @ library::LibraryError::NotUtf8)
        | Err(err @ library::LibraryError::NotRegularFile)
        | Err(err @ library::LibraryError::OutsideRoot)
        | Err(err @ library::LibraryError::InvalidSource(_)) => {
            response.prompt.content.clear();
            response.resolved_path = Some(resolved.to_string_lossy().into_owned());
            response.source_error = Some(err.user_message());
            Ok(response)
        }
        Err(library::LibraryError::SourceMissing) | Err(library::LibraryError::RootRequired) => {
            response.prompt.content.clear();
            response.source_missing = true;
            response.resolved_path = Some(resolved.to_string_lossy().into_owned());
            Ok(response)
        }
        Err(library::LibraryError::Io(err)) => Err(format!("Failed to read prompt: {err}")),
    }
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

    let kind = req.kind.unwrap_or(PromptKind::Prompt);
    let template = req.template.unwrap_or(false);
    let body = if kind == PromptKind::Link {
        let source = req
            .source
            .as_deref()
            .ok_or_else(|| MSG_LINK_SOURCE_REQUIRED.to_string())?;
        let source = normalize_link_source(&scope, source)?;
        PromptPointerFields {
            kind: PromptKind::Link,
            source: Some(source),
            template,
        }
        .apply_to_mapping(&mut fm);
        ""
    } else {
        PromptPointerFields::prepare_mapping_for_write(&mut fm);
        req.content.as_deref().unwrap_or("")
    };

    let content = frontmatter::serialize_frontmatter(&fm, body)?;
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

    let mut pointer = PromptPointerFields::from_mapping(&fm);
    if let Some(requested_kind) = req.kind {
        if requested_kind != pointer.kind {
            return Err(MSG_KIND_IMMUTABLE.to_string());
        }
    }

    let content_body = if pointer.kind == PromptKind::Link {
        if let Some(source) = req.source {
            let trimmed = source.trim();
            if trimmed.is_empty() {
                return Err(MSG_LINK_SOURCE_REQUIRED.to_string());
            }
            let cleaned = normalize_link_source(&scope, trimmed)?;
            pointer.source = Some(cleaned);
        }
        if let Some(template) = req.template {
            pointer.template = template;
        }
        pointer.apply_to_mapping(&mut fm);
        old_body.trim_start().to_string()
    } else {
        let body = req
            .content
            .unwrap_or_else(|| old_body.trim_start().to_string());
        PromptPointerFields::prepare_mapping_for_write(&mut fm);
        body
    };

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

/// Metadata-only view for link prompts — never loads the linked file or pointer body.
fn prompt_for_semantic_search(summary: &PromptSummary) -> Prompt {
    Prompt {
        id: summary.id.clone(),
        title: summary.title.clone(),
        path: summary.path.clone(),
        scope: summary.scope.clone(),
        project_id: summary.project_id.clone(),
        folder: summary.folder.clone(),
        tags: summary.tags.clone(),
        description: summary.description.clone(),
        kind: summary.kind,
        source: summary.source.clone(),
        template: false,
        frontmatter: Mapping::new(),
        content: String::new(),
    }
}

fn semantic_score(query: &str, prompt: &Prompt) -> (f32, Vec<String>) {
    let query_terms = tokenize(query);
    if query_terms.is_empty() {
        return (0.0, Vec::new());
    }

    let is_link = prompt.kind == PromptKind::Link;

    let mut expanded_terms = HashSet::new();
    for t in &query_terms {
        expanded_terms.insert(t.clone());
        for extra in semantic_expansions(t) {
            expanded_terms.insert((*extra).to_string());
        }
    }

    let title_tokens: HashSet<String> = tokenize(&prompt.title).into_iter().collect();
    let tag_tokens: HashSet<String> = prompt.tags.iter().map(|t| t.to_lowercase()).collect();
    let body_tokens: HashSet<String> = if is_link {
        HashSet::new()
    } else {
        tokenize(&prompt.content).into_iter().collect()
    };
    let desc_tokens: HashSet<String> = tokenize(prompt.description.as_deref().unwrap_or(""))
        .into_iter()
        .collect();
    let folder_tokens: HashSet<String> = if is_link {
        tokenize(&prompt.folder).into_iter().collect()
    } else {
        HashSet::new()
    };
    let source_tokens: HashSet<String> = if is_link {
        tokenize(prompt.source.as_deref().unwrap_or(""))
            .into_iter()
            .collect()
    } else {
        HashSet::new()
    };

    let mut matched = BTreeSet::new();
    let mut title_hits = 0.0f32;
    let mut tag_hits = 0.0f32;
    let mut body_hits = 0.0f32;
    let mut desc_hits = 0.0f32;
    let mut folder_hits = 0.0f32;
    let mut source_hits = 0.0f32;

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
        if folder_tokens.contains(term) {
            folder_hits += 1.0;
            matched.insert(term.clone());
        }
        if source_tokens.contains(term) {
            source_hits += 1.0;
            matched.insert(term.clone());
        }
    }

    let denom = expanded_terms.len() as f32;
    if denom == 0.0 {
        return (0.0, Vec::new());
    }

    let mut score = if is_link {
        (title_hits / denom) * 0.35
            + (tag_hits / denom) * 0.22
            + (desc_hits / denom) * 0.18
            + (source_hits / denom) * 0.15
            + (folder_hits / denom) * 0.10
    } else {
        (title_hits / denom) * 0.40
            + (tag_hits / denom) * 0.25
            + (desc_hits / denom) * 0.20
            + (body_hits / denom) * 0.15
    };

    let query_lc = query.to_lowercase();
    if is_link {
        if prompt
            .source
            .as_deref()
            .is_some_and(|s| s.to_lowercase().contains(&query_lc))
        {
            score += 0.15;
        }
    } else if prompt.content.to_lowercase().contains(&query_lc) {
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
        let full = if item.kind == PromptKind::Link {
            prompt_for_semantic_search(&item)
        } else {
            get_prompt(&item.id)?
        };
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

#[cfg(test)]
mod link_pointer_tests {
    use super::*;
    use std::sync::{Mutex, Once};
    use uuid::Uuid;

    static INIT: Once = Once::new();
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    fn ensure_test_data_dir() {
        INIT.call_once(|| {
            let base = std::env::temp_dir()
                .join(format!("ironpad-prompt-link-tests-{}", std::process::id()));
            let _ = fs::create_dir_all(&base);
            std::env::set_var("IRONPAD_DATA_DIR", &base);
            crate::config::init_data_dir();
        });
    }

    fn ensure_project(project_id: &str) {
        let dir = crate::config::data_dir().join("projects").join(project_id);
        fs::create_dir_all(&dir).expect("project dir");
        if !dir.join("index.md").exists() {
            fs::write(dir.join("index.md"), "---\ntitle: test\n---\n").expect("index");
        }
    }

    #[test]
    fn create_project_link_ignores_content_writes_empty_body() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();
        let project_id = format!("plink-{}", Uuid::new_v4());
        ensure_project(&project_id);

        let prompt = create_prompt(PromptCreateRequest {
            scope: Some("project".into()),
            project_id: Some(project_id.clone()),
            title: Some("Doc link".into()),
            folder: None,
            tags: None,
            description: None,
            content: Some("should be ignored".into()),
            kind: Some(PromptKind::Link),
            source: Some("docs/index.md".into()),
            template: None,
        })
        .expect("create link");

        assert_eq!(prompt.kind, PromptKind::Link);
        assert_eq!(prompt.source.as_deref(), Some("docs/index.md"));
        assert!(prompt.content.is_empty());

        let file_path = find_prompt_path(&prompt.id).unwrap().2;
        let raw = fs::read_to_string(file_path).expect("read file");
        let (fm, body, _) = frontmatter::parse_frontmatter(&raw);
        assert_eq!(
            fm.get(&Value::from("kind")).and_then(|v| v.as_str()),
            Some("link")
        );
        assert!(body.trim().is_empty());
    }

    #[test]
    fn create_project_link_rejects_escape_source() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();
        let project_id = format!("plink-bad-{}", Uuid::new_v4());
        ensure_project(&project_id);

        let err = create_prompt(PromptCreateRequest {
            scope: Some("project".into()),
            project_id: Some(project_id),
            title: Some("Bad".into()),
            folder: None,
            tags: None,
            description: None,
            content: None,
            kind: Some(PromptKind::Link),
            source: Some("../x.md".into()),
            template: None,
        })
        .expect_err("bad source");

        assert!(err.contains(".."));
        assert!(is_client_error(&err));
    }

    #[test]
    fn create_project_link_accepts_absolute_source() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();
        let project_id = format!("plink-abs-{}", Uuid::new_v4());
        ensure_project(&project_id);

        let prompt = create_prompt(PromptCreateRequest {
            scope: Some("project".into()),
            project_id: Some(project_id),
            title: Some("Outside".into()),
            folder: None,
            tags: None,
            description: None,
            content: None,
            kind: Some(PromptKind::Link),
            source: Some(r"C:\outside\notes.md".into()),
            template: None,
        })
        .expect("absolute project source");

        assert_eq!(prompt.source.as_deref(), Some(r"C:\outside\notes.md"));
    }

    #[test]
    fn create_global_link_rejects_relative_source() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();

        let err = create_prompt(PromptCreateRequest {
            scope: Some("global".into()),
            project_id: None,
            title: Some("Global link".into()),
            folder: None,
            tags: None,
            description: None,
            content: None,
            kind: Some(PromptKind::Link),
            source: Some("relative.md".into()),
            template: None,
        })
        .expect_err("relative global");

        assert!(err.contains("absolute"));
        assert!(is_client_error(&err));
    }

    #[test]
    fn update_link_template_rewrites_pointer_only() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();
        let project_id = format!("plink-upd-{}", Uuid::new_v4());
        ensure_project(&project_id);

        let created = create_prompt(PromptCreateRequest {
            scope: Some("project".into()),
            project_id: Some(project_id),
            title: Some("Updatable".into()),
            folder: None,
            tags: None,
            description: None,
            content: None,
            kind: Some(PromptKind::Link),
            source: Some("docs/index.md".into()),
            template: Some(false),
        })
        .expect("create");

        let updated = update_prompt(
            &created.id,
            PromptUpdateRequest {
                title: None,
                folder: None,
                tags: None,
                description: None,
                content: Some("must ignore".into()),
                kind: None,
                source: None,
                template: Some(true),
            },
        )
        .expect("update");

        assert!(updated.template);
        assert!(updated.content.is_empty());

        let file_path = find_prompt_path(&updated.id).unwrap().2;
        let raw = fs::read_to_string(file_path).expect("read");
        let (fm, body, _) = frontmatter::parse_frontmatter(&raw);
        assert_eq!(
            fm.get(&Value::from("template")).and_then(|v| v.as_bool()),
            Some(true)
        );
        assert!(body.trim().is_empty());
    }

    #[test]
    fn create_plain_prompt_writes_content_without_kind_key() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();

        let prompt = create_prompt(PromptCreateRequest {
            scope: Some("global".into()),
            project_id: None,
            title: Some("Plain".into()),
            folder: None,
            tags: None,
            description: None,
            content: Some("Hello body".into()),
            kind: None,
            source: None,
            template: None,
        })
        .expect("create");

        assert_eq!(prompt.kind, PromptKind::Prompt);
        assert_eq!(prompt.content, "Hello body");

        let file_path = find_prompt_path(&prompt.id).unwrap().2;
        let raw = fs::read_to_string(file_path).expect("read");
        let (fm, body, _) = frontmatter::parse_frontmatter(&raw);
        assert!(!fm.contains_key(Value::from("kind")));
        assert_eq!(body.trim(), "Hello body");
    }

    #[test]
    fn update_rejects_kind_change() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();

        let created = create_prompt(PromptCreateRequest {
            scope: Some("global".into()),
            project_id: None,
            title: Some("Plain".into()),
            folder: None,
            tags: None,
            description: None,
            content: Some("x".into()),
            kind: None,
            source: None,
            template: None,
        })
        .expect("create");

        let err = update_prompt(
            &created.id,
            PromptUpdateRequest {
                title: None,
                folder: None,
                tags: None,
                description: None,
                content: None,
                kind: Some(PromptKind::Link),
                source: Some("C:\\x.md".into()),
                template: None,
            },
        )
        .expect_err("kind change");

        assert!(err.contains("cannot be changed"));
    }

    fn encode_query_value(value: &str) -> String {
        let mut out = String::new();
        for byte in value.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(byte as char);
                }
                _ => out.push_str(&format!("%{byte:02X}")),
            }
        }
        out
    }

    async fn get_json(
        id: &str,
        root: Option<&Path>,
    ) -> (axum::http::StatusCode, serde_json::Value) {
        let app = crate::routes::prompts::router();
        let uri = match root {
            Some(root) => format!("/{id}?root={}", encode_query_value(&root.to_string_lossy())),
            None => format!("/{id}"),
        };
        let response = tower::ServiceExt::oneshot(
            app,
            axum::http::Request::builder()
                .uri(uri)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .expect("response");
        let status = response.status();
        let bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value =
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, json)
    }

    #[test]
    fn get_link_and_prompt_content_follow_request_time_rules() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();

        let root = tempfile::TempDir::new().expect("root");
        let docs = root.path().join("docs");
        fs::create_dir_all(&docs).expect("docs");
        let source_file = docs.join("notes.md");
        fs::write(&source_file, "alpha").expect("write source");

        let project_id = format!("plink-get-{}", Uuid::new_v4());
        ensure_project(&project_id);
        let link = create_prompt(PromptCreateRequest {
            scope: Some("project".into()),
            project_id: Some(project_id),
            title: Some("Doc link".into()),
            folder: None,
            tags: None,
            description: None,
            content: Some("ignored".into()),
            kind: Some(PromptKind::Link),
            source: Some("docs/notes.md".into()),
            template: None,
        })
        .expect("create link");

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");

        let (status, body) = rt.block_on(get_json(&link.id, Some(root.path())));
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(body["content"], "alpha");
        assert_eq!(body["read_only"], true);
        assert_eq!(body["source_missing"], false);
        assert_eq!(body["root_required"], serde_json::Value::Null);
        let resolved = body["resolved_path"].as_str().expect("resolved_path");
        assert!(resolved.ends_with("notes.md"), "{resolved}");

        fs::write(&source_file, "beta").expect("overwrite");
        let (status, body) = rt.block_on(get_json(&link.id, Some(root.path())));
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(body["content"], "beta");

        fs::remove_file(&source_file).expect("delete source");
        let (status, body) = rt.block_on(get_json(&link.id, Some(root.path())));
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(body["source_missing"], true);
        assert_eq!(body["content"], "");

        let (status, body) = rt.block_on(get_json(&link.id, None));
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(body["root_required"], true);
        assert_eq!(body["content"], "");

        let prompt = create_prompt(PromptCreateRequest {
            scope: Some("global".into()),
            project_id: None,
            title: Some("Plain".into()),
            folder: None,
            tags: None,
            description: None,
            content: Some("stored body".into()),
            kind: None,
            source: None,
            template: None,
        })
        .expect("create prompt");
        let (status, body) = rt.block_on(get_json(&prompt.id, None));
        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(body["kind"], "prompt");
        assert_eq!(body["content"], "stored body");
        assert_eq!(body["read_only"], false);
        assert_eq!(body["source_missing"], false);
    }

    #[test]
    fn semantic_search_matches_link_source_not_linked_file_body() {
        let _guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        ensure_test_data_dir();
        let project_id = format!("plink-sem-{}", Uuid::new_v4());
        ensure_project(&project_id);

        let project_root = crate::config::data_dir().join("projects").join(&project_id);
        let docs_dir = project_root.join("docs");
        fs::create_dir_all(&docs_dir).expect("docs dir");
        const BODY_ONLY_TERM: &str = "xyzzybodyonlysemantic";
        fs::write(
            docs_dir.join("ROADMAP.md"),
            format!("## Notes\n{BODY_ONLY_TERM}\n"),
        )
        .expect("roadmap file");

        let link = create_prompt(PromptCreateRequest {
            scope: Some("project".into()),
            project_id: Some(project_id.clone()),
            title: Some("Plan".into()),
            folder: None,
            tags: None,
            description: None,
            content: Some("ignored pointer body".into()),
            kind: Some(PromptKind::Link),
            source: Some("docs/ROADMAP.md".into()),
            template: None,
        })
        .expect("create link");

        let by_roadmap = semantic_search(&PromptSearchQuery {
            q: "roadmap".into(),
            scope: Some("project".into()),
            project_id: Some(project_id.clone()),
            folder: None,
            tag: None,
            limit: Some(25),
        })
        .expect("search roadmap");
        assert!(
            by_roadmap.iter().any(|r| r.prompt.id == link.id),
            "expected link {:?} in {:?}",
            link.id,
            by_roadmap
        );

        let by_body_only = semantic_search(&PromptSearchQuery {
            q: BODY_ONLY_TERM.into(),
            scope: Some("project".into()),
            project_id: Some(project_id),
            folder: None,
            tag: None,
            limit: Some(25),
        })
        .expect("search body term");
        assert!(
            !by_body_only.iter().any(|r| r.prompt.id == link.id),
            "link must not match terms only present in the linked file"
        );
    }
}
