//! Library link `source` validation and resolution (PRD §4.3, §4.6).
//! Later endpoints must use these helpers so paths cannot bypass the rules.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use axum::http::StatusCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Project,
    Global,
}

#[derive(Debug)]
pub enum LibraryError {
    SourceMissing,
    OutsideRoot,
    InvalidSource(String),
    RootRequired,
    NotRegularFile,
    TooLarge { size: u64, max: u64 },
    DisallowedSuffix(String),
    NotUtf8,
    Io(io::Error),
}

pub const MSG_SOURCE_MISSING: &str = "The linked file was not found.";
pub const MSG_OUTSIDE_ROOT: &str =
    "That path is outside the project folder root. Change the root or add the file from the global library.";
pub const MSG_ROOT_REQUIRED: &str = "Set a folder root for this project to open this link.";
pub const MSG_NOT_REGULAR_FILE: &str = "Only regular files can be linked.";
pub const MSG_NOT_UTF8: &str = "The file is not UTF-8 text.";

impl LibraryError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            LibraryError::SourceMissing => StatusCode::NOT_FOUND,
            LibraryError::OutsideRoot
            | LibraryError::InvalidSource(_)
            | LibraryError::RootRequired
            | LibraryError::NotRegularFile
            | LibraryError::TooLarge { .. }
            | LibraryError::DisallowedSuffix(_)
            | LibraryError::NotUtf8 => StatusCode::BAD_REQUEST,
            LibraryError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            LibraryError::SourceMissing => MSG_SOURCE_MISSING.to_string(),
            LibraryError::OutsideRoot => MSG_OUTSIDE_ROOT.to_string(),
            LibraryError::InvalidSource(detail) => detail.clone(),
            LibraryError::RootRequired => MSG_ROOT_REQUIRED.to_string(),
            LibraryError::NotRegularFile => MSG_NOT_REGULAR_FILE.to_string(),
            LibraryError::TooLarge { size, max } => {
                format!("Linked file is too large ({size} bytes; maximum is {max} bytes).")
            }
            LibraryError::DisallowedSuffix(ext) => {
                format!("Files with the “{ext}” suffix cannot be linked.")
            }
            LibraryError::NotUtf8 => MSG_NOT_UTF8.to_string(),
            LibraryError::Io(err) => {
                tracing::warn!(error = %err, "library I/O error");
                "Could not read the linked file.".to_string()
            }
        }
    }
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.user_message())
    }
}

impl PartialEq for LibraryError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LibraryError::SourceMissing, LibraryError::SourceMissing) => true,
            (LibraryError::OutsideRoot, LibraryError::OutsideRoot) => true,
            (LibraryError::RootRequired, LibraryError::RootRequired) => true,
            (LibraryError::NotRegularFile, LibraryError::NotRegularFile) => true,
            (LibraryError::NotUtf8, LibraryError::NotUtf8) => true,
            (LibraryError::InvalidSource(a), LibraryError::InvalidSource(b)) => a == b,
            (
                LibraryError::TooLarge { size: s1, max: m1 },
                LibraryError::TooLarge { size: s2, max: m2 },
            ) => s1 == s2 && m1 == m2,
            (LibraryError::DisallowedSuffix(a), LibraryError::DisallowedSuffix(b)) => a == b,
            _ => false,
        }
    }
}

/// Drop characters Windows paste inserts into a path (bidi marks, zero-width chars, a BOM)
/// and one pair of wrapping quotes from Copy as path.
pub fn sanitize_user_path(input: &str) -> String {
    let stripped: String = input.chars().filter(|c| !is_path_noise(*c)).collect();
    let trimmed = stripped.trim();
    strip_wrapping_quotes(trimmed).trim().to_string()
}

fn is_path_noise(c: char) -> bool {
    matches!(
        c,
        '\u{061C}'
            | '\u{200B}'
            | '\u{200C}'
            | '\u{200D}'
            | '\u{200E}'
            | '\u{200F}'
            | '\u{202A}'
            | '\u{202B}'
            | '\u{202C}'
            | '\u{202D}'
            | '\u{202E}'
            | '\u{2066}'
            | '\u{2067}'
            | '\u{2068}'
            | '\u{2069}'
            | '\u{FEFF}'
    )
}

fn strip_wrapping_quotes(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn sanitized_root(root: &Path) -> PathBuf {
    PathBuf::from(sanitize_user_path(&root.to_string_lossy()))
}

/// Absolute on this OS, or a Windows drive path (`C:/...`, `C:\...`).
/// Drive paths stay absolute when the same check runs on Unix (CI).
pub fn is_absolute_user_path(source: &str) -> bool {
    if Path::new(source).is_absolute() {
        return true;
    }
    let bytes = source.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

/// Project-relative `source`: forward slashes, no leading `/`, no `..`, not absolute.
pub fn validate_project_source(source: &str) -> Result<(), LibraryError> {
    if source.is_empty() {
        return Err(LibraryError::InvalidSource(
            "Project link source must not be empty.".into(),
        ));
    }
    if source.starts_with('/') {
        return Err(LibraryError::InvalidSource(
            "Project link source must be relative to the folder root (no leading slash).".into(),
        ));
    }
    if source.contains('\\') {
        return Err(LibraryError::InvalidSource(
            "Project link source must use forward slashes.".into(),
        ));
    }
    if is_absolute_user_path(source) {
        return Err(LibraryError::InvalidSource(
            "Project link source must be relative to the folder root.".into(),
        ));
    }
    if source.split('/').any(|segment| segment == "..") {
        return Err(LibraryError::InvalidSource(
            "Project link source must not contain '..'.".into(),
        ));
    }
    Ok(())
}

/// Global `source` must be an absolute path on this machine.
pub fn validate_global_source(source: &str) -> Result<(), LibraryError> {
    if source.is_empty() {
        return Err(LibraryError::InvalidSource(
            "Global link source must not be empty.".into(),
        ));
    }
    if !is_absolute_user_path(source) {
        return Err(LibraryError::InvalidSource(
            "Global link source must be an absolute path.".into(),
        ));
    }
    Ok(())
}

/// Resolve a stored `source` to an absolute path, enforcing scope rules.
pub fn resolve_source(
    kind_scope: Scope,
    source: &str,
    root: Option<&Path>,
) -> Result<PathBuf, LibraryError> {
    match kind_scope {
        Scope::Project => resolve_project_source(source, root),
        Scope::Global => resolve_global_source(source),
    }
}

/// Drop the `\\?\` prefix that `canonicalize` adds on Windows so clients get a normal path.
pub(crate) fn plain_absolute(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        return PathBuf::from(format!(r"\\{rest}"));
    }
    if let Some(rest) = text.strip_prefix(r"\\?\") {
        return PathBuf::from(rest);
    }
    path
}

fn resolve_absolute_file(source: &str) -> Result<PathBuf, LibraryError> {
    let source = sanitize_user_path(source);
    validate_global_source(&source)?;
    let path = PathBuf::from(source);
    match path.canonicalize() {
        Ok(path) => Ok(plain_absolute(path)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(LibraryError::SourceMissing),
        Err(err) => Err(LibraryError::Io(err)),
    }
}

fn resolve_project_source(source: &str, root: Option<&Path>) -> Result<PathBuf, LibraryError> {
    let source = sanitize_user_path(source);
    // Absolute sources are files picked anywhere on disk. They do not need the
    // folder root, and they are allowed to sit outside it.
    if is_absolute_user_path(&source) {
        return resolve_absolute_file(&source);
    }
    let root = root.ok_or(LibraryError::RootRequired)?;
    let root = sanitized_root(root);
    validate_project_source(&source)?;

    let joined = root.join(&source);
    let canonical_root = match root.canonicalize() {
        Ok(path) => path,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(LibraryError::InvalidSource(
                "The folder root was not found.".into(),
            ));
        }
        Err(err) => return Err(LibraryError::Io(err)),
    };
    let canonical_file = match joined.canonicalize() {
        Ok(path) => path,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(LibraryError::SourceMissing);
        }
        Err(err) => return Err(LibraryError::Io(err)),
    };

    if !canonical_file.starts_with(&canonical_root) {
        return Err(LibraryError::OutsideRoot);
    }

    Ok(plain_absolute(canonical_file))
}

fn resolve_global_source(source: &str) -> Result<PathBuf, LibraryError> {
    let source = sanitize_user_path(source);
    validate_global_source(&source)?;
    let path = PathBuf::from(source);
    match path.metadata() {
        Ok(_) => Ok(path),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Err(LibraryError::SourceMissing),
        Err(err) => Err(LibraryError::Io(err)),
    }
}

/// Store project-relative paths with forward slashes (Windows).
pub fn normalize_relative(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub const MAX_LINK_BYTES: u64 = 1_048_576;
pub const ALLOWED_SUFFIXES: &[&str] = &[
    "md", "markdown", "txt", "rs", "vue", "ts", "js", "toml", "json", "yml", "yaml", "py", "css",
    "html", "mdc",
];

fn extension_lower(path: &Path) -> Option<String> {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_ascii_lowercase())
}

/// Whether the path extension is in [`ALLOWED_SUFFIXES`] (case-insensitive).
pub fn suffix_allowed(path: &Path) -> bool {
    extension_lower(path)
        .map(|ext| ALLOWED_SUFFIXES.iter().any(|&s| s == ext))
        .unwrap_or(false)
}

/// Markdown sources can be opened with the rendered editor view.
pub fn supports_rendered(path: &Path) -> bool {
    matches!(
        extension_lower(path).as_deref(),
        Some("md") | Some("markdown")
    )
}

pub const MAX_IMAGE_BYTES: u64 = 8 * 1024 * 1024;

fn image_content_type(path: &Path) -> Result<&'static str, LibraryError> {
    match extension_lower(path).as_deref() {
        Some("png") => Ok("image/png"),
        Some("jpg") | Some("jpeg") => Ok("image/jpeg"),
        Some("gif") => Ok("image/gif"),
        Some("webp") => Ok("image/webp"),
        Some("svg") => Ok("image/svg+xml"),
        Some(ext) => Err(LibraryError::InvalidSource(format!(
            "“{ext}” is not an image the library can show."
        ))),
        None => Err(LibraryError::InvalidSource(
            "That file is not an image the library can show.".into(),
        )),
    }
}

/// Read an image next to a linked markdown file. Only image suffixes, and only
/// up to [`MAX_IMAGE_BYTES`].
pub fn read_image_file(path_str: &str) -> Result<(Vec<u8>, &'static str), LibraryError> {
    let path = resolve_absolute_file(path_str)?;
    let content_type = image_content_type(&path)?;
    let meta = match fs::metadata(&path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(LibraryError::SourceMissing);
        }
        Err(err) => return Err(LibraryError::Io(err)),
    };
    if !meta.is_file() {
        return Err(LibraryError::NotRegularFile);
    }
    let size = meta.len();
    if size > MAX_IMAGE_BYTES {
        return Err(LibraryError::TooLarge {
            size,
            max: MAX_IMAGE_BYTES,
        });
    }
    let bytes = fs::read(&path).map_err(LibraryError::Io)?;
    Ok((bytes, content_type))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickKind {
    File,
    Directory,
}

pub fn parse_pick_kind(kind: &str) -> Result<PickKind, LibraryError> {
    match kind.trim() {
        "file" => Ok(PickKind::File),
        "directory" => Ok(PickKind::Directory),
        _ => Err(LibraryError::InvalidSource(
            "Pick kind must be file or directory.".into(),
        )),
    }
}

/// Library root used to turn a prompt id into a stored link `source`.
pub struct PromptFileStore {
    data_dir: PathBuf,
}

impl PromptFileStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn from_config() -> Self {
        Self::new(crate::config::data_dir())
    }
}

/// Path that would be opened for a link, even when the file is absent.
/// When a project link has no root, this is the stored `source` string.
pub fn unresolved_link_path(scope: Scope, source: &str, root: Option<&Path>) -> String {
    let source = sanitize_user_path(source);
    if is_absolute_user_path(&source) {
        return source;
    }
    match scope {
        Scope::Project => match root {
            Some(root) => sanitized_root(root)
                .join(source)
                .to_string_lossy()
                .into_owned(),
            None => source,
        },
        Scope::Global => source,
    }
}

/// Resolve a library link id to an absolute path using the same rules as [`resolve_source`].
pub fn resolve_link_by_id(
    store: &PromptFileStore,
    id: &str,
    root: Option<&Path>,
) -> Result<PathBuf, LibraryError> {
    let (scope, source) = store.load_link_source(id)?;
    resolve_source(scope, &source, root)
}

impl PromptFileStore {
    pub fn load_link_source(&self, id: &str) -> Result<(Scope, String), LibraryError> {
        let mut roots = vec![self.data_dir.join("prompts")];
        let projects = self.data_dir.join("projects");
        if let Ok(entries) = fs::read_dir(projects) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    roots.push(entry.path().join("prompts"));
                }
            }
        }

        for root in roots {
            if !root.exists() {
                continue;
            }
            for entry in walkdir::WalkDir::new(&root)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                    continue;
                }
                let raw = fs::read_to_string(path).map_err(LibraryError::Io)?;
                let (fm, _, _) = crate::services::frontmatter::parse_frontmatter(&raw);
                let found = crate::services::frontmatter::get_str(&fm, "id")
                    .unwrap_or_else(|| crate::services::frontmatter::derive_id_from_path(path));
                if found != id {
                    continue;
                }
                let pointer = crate::models::prompt::PromptPointerFields::from_mapping(&fm);
                if pointer.kind != crate::models::prompt::PromptKind::Link {
                    return Err(LibraryError::InvalidSource("Prompt is not a link.".into()));
                }
                let source = pointer.source.ok_or_else(|| {
                    LibraryError::InvalidSource("source is required for link prompts".into())
                })?;
                let scope = match crate::services::frontmatter::get_str(&fm, "scope").as_deref() {
                    Some("project") => Scope::Project,
                    _ => Scope::Global,
                };
                return Ok((scope, source));
            }
        }

        Err(LibraryError::InvalidSource(format!(
            "Prompt not found: {id}"
        )))
    }
}

/// Metadata and suffix checks shared by read/open (does not read file contents).
pub fn preflight_linked_file(path: &Path) -> Result<(), LibraryError> {
    let meta = fs::metadata(path).map_err(LibraryError::Io)?;
    if !meta.is_file() {
        return Err(LibraryError::NotRegularFile);
    }
    if !suffix_allowed(path) {
        let ext = extension_lower(path).unwrap_or_default();
        return Err(LibraryError::DisallowedSuffix(ext));
    }
    Ok(())
}

/// Read a resolved link target: regular file, allowed suffix, size cap, strict UTF-8.
pub fn read_linked_file(path: &Path) -> Result<String, LibraryError> {
    preflight_linked_file(path)?;
    let meta = fs::metadata(path).map_err(LibraryError::Io)?;
    let size = meta.len();
    if size > MAX_LINK_BYTES {
        return Err(LibraryError::TooLarge {
            size,
            max: MAX_LINK_BYTES,
        });
    }
    let bytes = fs::read(path).map_err(LibraryError::Io)?;
    String::from_utf8(bytes).map_err(|_| LibraryError::NotUtf8)
}

#[derive(Debug)]
pub enum BrowseError {
    PathNotAbsolute,
    NotADirectory,
    DirectoryNotFound,
    Io(io::Error),
}

impl BrowseError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            BrowseError::PathNotAbsolute | BrowseError::NotADirectory => StatusCode::BAD_REQUEST,
            BrowseError::DirectoryNotFound => StatusCode::NOT_FOUND,
            BrowseError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            BrowseError::PathNotAbsolute => "Browse path must be an absolute directory.".into(),
            BrowseError::NotADirectory => "Browse path must be an existing directory.".into(),
            BrowseError::DirectoryNotFound => "That directory was not found.".into(),
            BrowseError::Io(err) => {
                tracing::warn!(error = %err, "library browse I/O error");
                "Could not read the directory.".into()
            }
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct BrowseEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub linkable: bool,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct BrowseResponse {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<BrowseEntry>,
}

/// Non-recursive one-level directory listing for the Add-link dialog.
pub fn browse_directory(path_str: &str) -> Result<BrowseResponse, BrowseError> {
    let cleaned = sanitize_user_path(path_str);
    let path = Path::new(&cleaned);
    if !path.is_absolute() {
        return Err(BrowseError::PathNotAbsolute);
    }

    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return Err(BrowseError::DirectoryNotFound);
        }
        Err(err) => return Err(BrowseError::Io(err)),
    };
    if !meta.is_dir() {
        return Err(BrowseError::NotADirectory);
    }

    let canonical = plain_absolute(path.canonicalize().map_err(BrowseError::Io)?);
    let canonical_path = canonical.to_string_lossy().into_owned();

    let parent = canonical
        .parent()
        .and_then(|p| p.canonicalize().ok())
        .map(plain_absolute)
        .map(|p| p.to_string_lossy().into_owned());

    let mut entries = Vec::new();
    for dir_entry in fs::read_dir(&canonical).map_err(BrowseError::Io)? {
        let dir_entry = match dir_entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = dir_entry.file_name().to_string_lossy().into_owned();
        if name == ".git" {
            continue;
        }
        let entry_path = dir_entry.path();
        let meta = match fs::metadata(&entry_path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_dir = meta.is_dir();
        let size = if is_dir { 0 } else { meta.len() };
        let resolved = entry_path
            .canonicalize()
            .map(plain_absolute)
            .unwrap_or(entry_path);
        let path_display = resolved.to_string_lossy().into_owned();
        let linkable = !is_dir && suffix_allowed(&resolved) && size <= MAX_LINK_BYTES;
        entries.push(BrowseEntry {
            name,
            path: path_display,
            is_dir,
            size,
            linkable,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a
            .name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase()),
    });

    Ok(BrowseResponse {
        path: canonical_path,
        parent,
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn temp_root_with_index() -> (TempDir, PathBuf) {
        let dir = TempDir::new().expect("temp dir");
        let root = dir.path().to_path_buf();
        let docs = root.join("docs");
        fs::create_dir_all(&docs).expect("mkdir docs");
        fs::write(docs.join("index.md"), "# index\n").expect("write index");
        (dir, root)
    }

    #[test]
    fn resolve_project_file_inside_root() {
        let (_guard, root) = temp_root_with_index();
        let resolved =
            resolve_source(Scope::Project, "docs/index.md", Some(root.as_path())).expect("resolve");
        let expected = plain_absolute(
            root.join("docs/index.md")
                .canonicalize()
                .expect("canonical"),
        );
        assert_eq!(resolved, expected);
    }

    #[test]
    fn validate_project_rejects_bad_sources() {
        for bad in ["../x.md", "/abs.md", "C:/abs.md", r"a\b.md"] {
            assert!(
                validate_project_source(bad).is_err(),
                "expected reject: {bad}"
            );
        }
    }

    #[test]
    fn resolve_link_by_id_reads_pointer() {
        let data = TempDir::new().expect("data");
        let prompts = data.path().join("projects").join("p1").join("prompts");
        fs::create_dir_all(&prompts).expect("prompts");
        let root = TempDir::new().expect("root");
        fs::write(root.path().join("note.md"), "from-disk").expect("note");
        fs::write(
            prompts.join("link.md"),
            "---\nid: link-1\nscope: project\nkind: link\nsource: note.md\n---\n",
        )
        .expect("pointer");

        let store = PromptFileStore::new(data.path());
        let resolved = resolve_link_by_id(&store, "link-1", Some(root.path())).expect("resolve");
        assert!(resolved.ends_with("note.md"));
        assert_eq!(read_linked_file(&resolved).expect("read"), "from-disk");
    }

    #[test]
    fn resolve_project_missing_file() {
        let (_guard, root) = temp_root_with_index();
        let err = resolve_source(Scope::Project, "docs/nope.md", Some(root.as_path()))
            .expect_err("missing");
        assert_eq!(err, LibraryError::SourceMissing);
    }

    #[test]
    fn resolve_project_requires_root() {
        let err = resolve_source(Scope::Project, "docs/index.md", None).expect_err("no root");
        assert_eq!(err, LibraryError::RootRequired);
    }

    #[test]
    fn sanitize_user_path_strips_bidi_mark_and_quotes() {
        assert_eq!(
            sanitize_user_path("\u{202A}\"G:\\DEV\\proman2k\""),
            "G:\\DEV\\proman2k"
        );
    }

    #[test]
    fn resolve_project_strips_bidi_mark_and_accepts_absolute_source_inside_root() {
        let (_guard, root) = temp_root_with_index();
        let absolute = root.join("docs").join("index.md");
        let display = absolute.to_string_lossy().replace('\\', "/");
        let dirty_source = format!("\u{202A}{display}");
        let dirty_root = PathBuf::from(format!("\u{202A}{}", root.to_string_lossy()));
        let resolved = resolve_source(Scope::Project, &dirty_source, Some(dirty_root.as_path()))
            .expect("resolve");
        let expected = plain_absolute(absolute.canonicalize().expect("canonical"));
        assert_eq!(resolved, expected);
    }

    #[test]
    fn resolve_project_absolute_source_outside_root() {
        let (_guard, root) = temp_root_with_index();
        let outside = TempDir::new().expect("outside");
        let file = outside.path().join("x.md");
        fs::write(&file, "x").expect("write");
        let display = file.to_string_lossy().replace('\\', "/");
        let resolved =
            resolve_source(Scope::Project, &display, Some(root.as_path())).expect("outside file");
        assert_eq!(
            resolved,
            plain_absolute(file.canonicalize().expect("canon"))
        );
    }

    #[test]
    fn resolve_project_absolute_source_without_root() {
        let outside = TempDir::new().expect("outside");
        let file = outside.path().join("x.md");
        fs::write(&file, "x").expect("write");
        let display = file.to_string_lossy().replace('\\', "/");
        let resolved = resolve_source(Scope::Project, &display, None).expect("no root needed");
        assert_eq!(
            resolved,
            plain_absolute(file.canonicalize().expect("canon"))
        );
    }

    #[test]
    fn read_image_file_returns_bytes_and_type() {
        let dir = TempDir::new().expect("temp");
        let path = dir.path().join("banner.png");
        fs::write(&path, b"png-bytes").expect("write");
        let (bytes, content_type) = read_image_file(&path.to_string_lossy()).expect("image");
        assert_eq!(bytes, b"png-bytes");
        assert_eq!(content_type, "image/png");
    }

    #[test]
    fn read_image_file_rejects_markdown_and_relative() {
        let dir = TempDir::new().expect("temp");
        let path = dir.path().join("note.md");
        fs::write(&path, b"# hi").expect("write");
        let err = read_image_file(&path.to_string_lossy()).expect_err("md");
        assert!(matches!(err, LibraryError::InvalidSource(_)));
        let err = read_image_file("docs/a.png").expect_err("relative");
        assert!(matches!(err, LibraryError::InvalidSource(_)));
    }

    #[test]
    fn parse_pick_kind_accepts_file_and_directory() {
        assert_eq!(parse_pick_kind("file").expect("file"), PickKind::File);
        assert_eq!(
            parse_pick_kind(" directory ").expect("directory"),
            PickKind::Directory
        );
        assert!(parse_pick_kind("folder").is_err());
    }

    #[test]
    fn browse_directory_strips_leading_bidi_mark() {
        let dir = TempDir::new().expect("temp");
        fs::write(dir.path().join("note.md"), "x").expect("write");
        let dirty = format!("\u{202A}{}", dir.path().to_string_lossy());
        let resp = browse_directory(&dirty).expect("browse");
        assert!(resp.entries.iter().any(|entry| entry.name == "note.md"));
    }

    #[cfg(unix)]
    #[test]
    fn resolve_project_symlink_outside_root() {
        use std::os::unix::fs::symlink;

        let outside = TempDir::new().expect("outside");
        let outside_file = outside.path().join("secret.md");
        fs::write(&outside_file, "secret").expect("write outside");

        let dir = TempDir::new().expect("root");
        let link_path = dir.path().join("escape.md");
        symlink(&outside_file, &link_path).expect("symlink");

        let err = resolve_source(Scope::Project, "escape.md", Some(dir.path()))
            .expect_err("outside root");
        assert_eq!(err, LibraryError::OutsideRoot);
    }

    #[test]
    fn validate_global_source_rules() {
        assert!(validate_global_source("relative.md").is_err());
        let file = TempDir::new().expect("tmpdir");
        let abs = file.path().join("x.md");
        fs::write(&abs, "x").expect("write");
        let abs_str = abs.to_string_lossy().to_string();
        assert!(validate_global_source(&abs_str).is_ok());
    }

    #[test]
    fn normalize_relative_uses_forward_slashes() {
        assert_eq!(
            normalize_relative(Path::new(r"docs\index.md")),
            "docs/index.md"
        );
    }

    #[test]
    fn read_linked_uppercase_md_utf8() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("note.MD");
        let content = "Hello 世界";
        fs::write(&path, content).expect("write");
        assert_eq!(read_linked_file(&path).expect("read"), content);
    }

    #[test]
    fn read_linked_directory_not_regular_file() {
        let dir = TempDir::new().expect("temp dir");
        let err = read_linked_file(dir.path()).expect_err("directory");
        assert_eq!(err, LibraryError::NotRegularFile);
    }

    #[test]
    fn read_linked_disallowed_suffix() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("file.exe");
        fs::write(&path, b"bin").expect("write");
        let err = read_linked_file(&path).expect_err("exe");
        assert_eq!(err, LibraryError::DisallowedSuffix("exe".into()));
    }

    #[test]
    fn read_linked_too_large_display_includes_size() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("big.md");
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .expect("open");
        file.set_len(1_048_577).expect("set_len");
        let err = read_linked_file(&path).expect_err("too large");
        assert!(matches!(
            err,
            LibraryError::TooLarge {
                size: 1_048_577,
                max: MAX_LINK_BYTES
            }
        ));
        assert!(err.to_string().contains("1048577"));
    }

    #[test]
    fn read_linked_max_size_passes() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("max.md");
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&path)
            .expect("open");
        file.set_len(MAX_LINK_BYTES).expect("set_len");
        assert!(read_linked_file(&path).is_ok());
    }

    #[test]
    fn read_linked_not_utf8() {
        let dir = TempDir::new().expect("temp dir");
        let path = dir.path().join("bad.md");
        fs::write(&path, [0xff, 0xfe, 0x00]).expect("write");
        let err = read_linked_file(&path).expect_err("not utf8");
        assert_eq!(err, LibraryError::NotUtf8);
    }

    #[test]
    fn supports_rendered_markdown_only() {
        assert!(supports_rendered(Path::new("a.md")));
        assert!(supports_rendered(Path::new("a.MARKDOWN")));
        assert!(!supports_rendered(Path::new("a.txt")));
    }
}
