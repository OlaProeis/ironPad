use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::services::library::{self, LibraryError, PickKind, PromptFileStore};

pub fn router() -> Router {
    Router::new()
        .route("/browse", get(browse))
        .route("/open", post(open_link))
        .route("/pick", post(pick))
        .route("/media", get(media))
}

#[derive(Debug, Deserialize)]
pub struct BrowseQuery {
    path: String,
}

async fn browse(Query(query): Query<BrowseQuery>) -> impl IntoResponse {
    match library::browse_directory(&query.path) {
        Ok(response) => Json(response).into_response(),
        Err(err) => (err.status_code(), err.user_message()).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct OpenLinkRequest {
    id: String,
    root: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OpenLinkResponse {
    ok: bool,
    resolved_path: String,
}

/// Resolve a link id for opening: same rules as read, without loading file contents.
pub fn resolve_for_open(
    store: &PromptFileStore,
    id: &str,
    root: Option<&Path>,
) -> Result<PathBuf, LibraryError> {
    let resolved = library::resolve_link_by_id(store, id, root)?;
    library::preflight_linked_file(&resolved)?;
    Ok(resolved)
}

/// Open `path` with the OS default application (not used in unit tests).
pub fn launch(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

fn map_open_error(err: LibraryError) -> (StatusCode, String) {
    match &err {
        LibraryError::InvalidSource(msg) if msg.starts_with("Prompt not found:") => {
            (StatusCode::NOT_FOUND, msg.clone())
        }
        _ => (err.status_code(), err.user_message()),
    }
}

async fn open_link(Json(body): Json<OpenLinkRequest>) -> impl IntoResponse {
    let store = PromptFileStore::from_config();
    let root = body
        .root
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let path = match resolve_for_open(&store, body.id.trim(), root.as_deref()) {
        Ok(path) => path,
        Err(err) => {
            let (status, message) = map_open_error(err);
            return (status, message).into_response();
        }
    };

    if let Err(err) = launch(&path) {
        tracing::warn!(error = %err, path = %path.display(), "failed to open linked file");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not open the linked file.".to_string(),
        )
            .into_response();
    }

    Json(OpenLinkResponse {
        ok: true,
        resolved_path: path.to_string_lossy().into_owned(),
    })
    .into_response()
}

#[derive(Debug, Deserialize)]
struct PickRequest {
    kind: String,
    directory: Option<String>,
}

#[derive(Debug, Serialize)]
struct PickResponse {
    paths: Vec<String>,
}

fn native_pick(kind: PickKind, directory: Option<String>) -> Vec<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(dir) = directory
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        dialog = dialog.set_directory(dir);
    }
    let picked = match kind {
        PickKind::Directory => dialog
            .set_title("Choose a folder")
            .pick_folder()
            .into_iter()
            .collect::<Vec<_>>(),
        PickKind::File => dialog
            .set_title("Choose files to link")
            .add_filter(
                "Text and source",
                &[
                    "md", "markdown", "txt", "rs", "vue", "ts", "js", "toml", "json", "yml",
                    "yaml", "py", "css", "html", "mdc",
                ],
            )
            .add_filter("All files", &["*"])
            .pick_files()
            .unwrap_or_default(),
    };
    picked
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

async fn pick(Json(body): Json<PickRequest>) -> impl IntoResponse {
    let kind = match library::parse_pick_kind(&body.kind) {
        Ok(kind) => kind,
        Err(err) => return (err.status_code(), err.user_message()).into_response(),
    };
    let directory = body.directory;
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = tx.send(native_pick(kind, directory));
    });
    let paths = rx.await.unwrap_or_default();
    Json(PickResponse { paths }).into_response()
}

#[derive(Debug, Deserialize)]
struct MediaQuery {
    path: String,
}

fn image_response(bytes: Vec<u8>, content_type: &'static str) -> axum::response::Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(
        header::CACHE_CONTROL,
        "private, max-age=0, must-revalidate".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        "default-src 'none'; style-src 'unsafe-inline'; sandbox"
            .parse()
            .unwrap(),
    );
    (headers, bytes).into_response()
}

async fn media(Query(query): Query<MediaQuery>) -> impl IntoResponse {
    match library::read_image_file(&query.path) {
        Ok((bytes, content_type)) => image_response(bytes, content_type),
        Err(err) => (err.status_code(), err.user_message()).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use std::fs;
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn fixture_root() -> (TempDir, std::path::PathBuf) {
        let dir = TempDir::new().expect("temp dir");
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("docs")).expect("docs");
        fs::write(root.join("ai-context.md"), "# ctx\n").expect("ai-context");
        let big = root.join("big.md");
        let file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&big)
            .expect("open big");
        file.set_len(library::MAX_LINK_BYTES + 1).expect("set_len");
        fs::write(root.join("app.exe"), b"MZ").expect("exe");
        (dir, root)
    }

    fn link_fixture() -> (TempDir, TempDir, PromptFileStore) {
        let data = TempDir::new().expect("data");
        let prompts = data.path().join("projects").join("p1").join("prompts");
        fs::create_dir_all(&prompts).expect("prompts");
        let root = TempDir::new().expect("root");
        fs::write(root.path().join("note.md"), "from-disk").expect("note");
        fs::write(
            prompts.join("link.md"),
            "---\nid: link-open-1\nscope: project\nkind: link\nsource: note.md\n---\n",
        )
        .expect("pointer");
        let store = PromptFileStore::new(data.path());
        (data, root, store)
    }

    use library::browse_directory;

    #[test]
    fn browse_directory_lists_and_flags_linkable() {
        let (_guard, root) = fixture_root();
        let root_str = root.to_string_lossy();
        let resp = browse_directory(&root_str).expect("browse");
        assert_eq!(resp.entries.len(), 4);
        assert!(resp.entries[0].is_dir);
        assert_eq!(resp.entries[0].name, "docs");

        let by_name: std::collections::HashMap<_, _> =
            resp.entries.iter().map(|e| (e.name.as_str(), e)).collect();
        assert_eq!(by_name["ai-context.md"].linkable, true);
        assert_eq!(by_name["big.md"].linkable, false);
        assert_eq!(by_name["app.exe"].linkable, false);
    }

    #[test]
    fn browse_rejects_relative_path() {
        let err = browse_directory("relative/dir").expect_err("relative");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn browse_rejects_file_path() {
        let dir = TempDir::new().expect("temp");
        let file = dir.path().join("only.md");
        fs::write(&file, "x").expect("write");
        let err = browse_directory(&file.to_string_lossy()).expect_err("file");
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn browse_missing_directory_not_found() {
        let dir = TempDir::new().expect("temp");
        let missing = dir.path().join("nope");
        let err = browse_directory(&missing.to_string_lossy()).expect_err("missing");
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    fn encode_query_value(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z'
                | b'a'..=b'z'
                | b'0'..=b'9'
                | b'-'
                | b'_'
                | b'.'
                | b'~'
                | b'/'
                | b':'
                | b'\\' => out.push(byte as char),
                _ => out.push_str(&format!("%{byte:02X}")),
            }
        }
        out
    }

    #[tokio::test]
    async fn browse_route_returns_json() {
        let (_guard, root) = fixture_root();
        let path = encode_query_value(&root.to_string_lossy());
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/browse?path={path}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json["entries"].as_array().map(|a| a.len()), Some(4));
    }

    #[test]
    fn resolve_for_open_existing_link() {
        let (_data, root, store) = link_fixture();
        let resolved = resolve_for_open(&store, "link-open-1", Some(root.path())).expect("resolve");
        let expected =
            library::plain_absolute(root.path().join("note.md").canonicalize().expect("canon"));
        assert_eq!(resolved, expected);
    }

    #[test]
    fn resolve_for_open_missing_source() {
        let data = TempDir::new().expect("data");
        let prompts = data.path().join("prompts");
        fs::create_dir_all(&prompts).expect("prompts");
        let root = TempDir::new().expect("root");
        fs::write(
            prompts.join("link.md"),
            "---\nid: link-missing\nscope: project\nkind: link\nsource: gone.md\n---\n",
        )
        .expect("pointer");
        let store = PromptFileStore::new(data.path());
        let err = resolve_for_open(&store, "link-missing", Some(root.path())).expect_err("missing");
        assert_eq!(err, LibraryError::SourceMissing);
    }

    #[test]
    fn resolve_for_open_non_link_prompt() {
        let data = TempDir::new().expect("data");
        let prompts = data.path().join("prompts");
        fs::create_dir_all(&prompts).expect("prompts");
        fs::write(
            prompts.join("plain.md"),
            "---\nid: plain-prompt\nkind: prompt\n---\nbody\n",
        )
        .expect("prompt");
        let store = PromptFileStore::new(data.path());
        let err = resolve_for_open(&store, "plain-prompt", None).expect_err("not link");
        assert!(matches!(err, LibraryError::InvalidSource(_)));
        assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn resolve_for_open_unknown_id_maps_not_found() {
        let data = TempDir::new().expect("data");
        let store = PromptFileStore::new(data.path());
        let err = resolve_for_open(&store, "no-such-id", None).expect_err("unknown");
        assert!(matches!(err, LibraryError::InvalidSource(_)));
        let (status, _) = map_open_error(err);
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn media_route_returns_image_bytes() {
        let dir = TempDir::new().expect("temp");
        let file = dir.path().join("banner.png");
        fs::write(&file, b"png-bytes").expect("write");
        let path = encode_query_value(&file.to_string_lossy());
        let app = router();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/media?path={path}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get(axum::http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok()),
            Some("image/png")
        );
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        assert_eq!(&bytes[..], b"png-bytes");
    }
}
