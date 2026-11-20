//! Persist and read GitHub snapshot metadata and per-item body/comment caches.

use std::cell::RefCell;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::services::filesystem;

use super::cache_dir::github_cache_dir;
use super::snapshot::Snapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Issue,
    Pull,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedComment {
    pub author_login: String,
    pub created_at: String,
    pub body: String,
}

#[cfg(test)]
thread_local! {
    static TEST_CACHE_ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_test_cache_root(root: Option<PathBuf>) {
    TEST_CACHE_ROOT.with(|cell| *cell.borrow_mut() = root);
}

fn project_cache_dir(project_id: &str) -> PathBuf {
    #[cfg(test)]
    {
        let override_root = TEST_CACHE_ROOT.with(|cell| cell.borrow().clone());
        if let Some(root) = override_root {
            let dir = root.join(project_id);
            let _ = fs::create_dir_all(&dir);
            return dir;
        }
    }
    github_cache_dir(project_id)
}

fn snapshot_path(project_id: &str) -> PathBuf {
    project_cache_dir(project_id).join("snapshot.json")
}

pub fn read_snapshot(project_id: &str) -> Option<Snapshot> {
    let path = snapshot_path(project_id);
    let bytes = fs::read(&path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn write_snapshot(project_id: &str, snap: &Snapshot) -> Result<(), String> {
    let path = snapshot_path(project_id);
    let parent = path.parent().ok_or("Invalid snapshot path")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let json = serde_json::to_vec(snap).map_err(|e| e.to_string())?;
    filesystem::atomic_write(&path, &json)
}

pub fn delete_snapshot_dir(project_id: &str) -> Result<(), String> {
    let dir = project_cache_dir(project_id);
    match fs::remove_dir_all(&dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

impl ItemKind {
    fn stem(self) -> &'static str {
        match self {
            ItemKind::Issue => "issue",
            ItemKind::Pull => "pull",
        }
    }
}

fn bodies_dir(project_id: &str) -> PathBuf {
    project_cache_dir(project_id).join("bodies")
}

pub fn body_path(project_id: &str, kind: ItemKind, number: u64) -> PathBuf {
    bodies_dir(project_id).join(format!("{}-{}.md", kind.stem(), number))
}

pub fn read_body(project_id: &str, kind: ItemKind, number: u64) -> Option<String> {
    let path = body_path(project_id, kind, number);
    fs::read_to_string(path).ok()
}

pub fn write_body(project_id: &str, kind: ItemKind, number: u64, text: &str) -> Result<(), String> {
    let path = body_path(project_id, kind, number);
    fs::create_dir_all(path.parent().ok_or("Invalid body path")?).map_err(|e| e.to_string())?;
    filesystem::atomic_write(&path, text.as_bytes())
}

pub fn delete_body(project_id: &str, kind: ItemKind, number: u64) {
    let path = body_path(project_id, kind, number);
    let _ = fs::remove_file(path);
}

fn comments_path(project_id: &str, kind: ItemKind, number: u64) -> PathBuf {
    bodies_dir(project_id).join(format!("comments-{}-{}.json", kind.stem(), number))
}

pub fn read_comments(project_id: &str, kind: ItemKind, number: u64) -> Option<Vec<CachedComment>> {
    let path = comments_path(project_id, kind, number);
    let bytes = fs::read(&path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn write_comments(
    project_id: &str,
    kind: ItemKind,
    number: u64,
    comments: &[CachedComment],
) -> Result<(), String> {
    let path = comments_path(project_id, kind, number);
    fs::create_dir_all(path.parent().ok_or("Invalid comments path")?).map_err(|e| e.to_string())?;
    let json = serde_json::to_vec(comments).map_err(|e| e.to_string())?;
    filesystem::atomic_write(&path, &json)
}

pub fn delete_comments(project_id: &str, kind: ItemKind, number: u64) {
    let path = comments_path(project_id, kind, number);
    let _ = fs::remove_file(path);
}

/// Remove body and comment files whose item numbers are no longer in the open set.
pub fn delete_caches_for_closed_items(
    project_id: &str,
    open_issue_numbers: &[u64],
    open_pull_numbers: &[u64],
) {
    let Ok(entries) = fs::read_dir(bodies_dir(project_id)) else {
        return;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().into_string().ok() else {
            continue;
        };
        let Some((kind, number)) = parse_cache_filename(&name) else {
            continue;
        };
        let still_open = match kind {
            ItemKind::Issue => open_issue_numbers.contains(&number),
            ItemKind::Pull => open_pull_numbers.contains(&number),
        };
        if !still_open {
            let _ = fs::remove_file(entry.path());
        }
    }
}

fn parse_cache_filename(name: &str) -> Option<(ItemKind, u64)> {
    if let Some(rest) = name.strip_prefix("comments-") {
        let rest = rest.strip_suffix(".json")?;
        return parse_kind_number(rest);
    }
    let rest = name.strip_suffix(".md")?;
    parse_kind_number(rest)
}

fn parse_kind_number(rest: &str) -> Option<(ItemKind, u64)> {
    if let Some(number) = rest.strip_prefix("issue-") {
        return Some((ItemKind::Issue, number.parse().ok()?));
    }
    if let Some(number) = rest.strip_prefix("pull-") {
        return Some((ItemKind::Pull, number.parse().ok()?));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::github::snapshot::{GithubIssue, GithubLabel, GithubPull};
    use tempfile::TempDir;

    static INIT_DATA_DIR: std::sync::Once = std::sync::Once::new();

    struct TestCacheGuard {
        _dir: TempDir,
    }

    impl TestCacheGuard {
        fn new() -> Self {
            INIT_DATA_DIR.call_once(crate::config::init_data_dir);
            let dir = TempDir::new().expect("tempdir");
            set_test_cache_root(Some(dir.path().to_path_buf()));
            Self { _dir: dir }
        }
    }

    impl Drop for TestCacheGuard {
        fn drop(&mut self) {
            set_test_cache_root(None);
        }
    }

    fn sample_snapshot() -> Snapshot {
        Snapshot {
            repo: "owner/repo".to_string(),
            login: Some("octocat".to_string()),
            refreshed_at: Some("2026-01-01T00:00:00Z".to_string()),
            issue_etag: None,
            pull_etag: None,
            issues: vec![GithubIssue {
                number: 1,
                title: "Bug".to_string(),
                html_url: "https://github.com/owner/repo/issues/1".to_string(),
                updated_at: "2026-01-02T00:00:00Z".to_string(),
                author_login: "alice".to_string(),
                labels: vec![GithubLabel {
                    name: "bug".to_string(),
                    color: Some("ff0000".to_string()),
                }],
                comments_count: 0,
                milestone_due: None,
                assignee_logins: vec![],
            }],
            pulls: vec![GithubPull {
                number: 2,
                title: "Fix".to_string(),
                html_url: "https://github.com/owner/repo/pull/2".to_string(),
                updated_at: "2026-01-03T00:00:00Z".to_string(),
                author_login: "bob".to_string(),
                labels: vec![],
                comments_count: 0,
                milestone_due: None,
                draft: false,
                requested_reviewer_logins: vec![],
                head_sha: "abc".to_string(),
            }],
            issues_truncated: false,
            pulls_truncated: false,
        }
    }

    const PROJECT: &str = "test-project";

    #[test]
    fn write_snapshot_then_read_returns_original() {
        let _guard = TestCacheGuard::new();
        let snap = sample_snapshot();
        write_snapshot(PROJECT, &snap).expect("write");
        let read = read_snapshot(PROJECT).expect("read");
        assert_eq!(read, snap);
    }

    #[test]
    fn delete_snapshot_dir_removes_directory_and_second_call_ok() {
        let _guard = TestCacheGuard::new();
        write_snapshot(PROJECT, &sample_snapshot()).expect("write");
        let dir = project_cache_dir(PROJECT);
        assert!(dir.exists());
        delete_snapshot_dir(PROJECT).expect("first delete");
        assert!(!dir.exists());
        delete_snapshot_dir(PROJECT).expect("second delete");
    }

    #[test]
    fn write_body_then_read_returns_original() {
        let _guard = TestCacheGuard::new();
        let text = "# Hello\n\nWorld";
        write_body(PROJECT, ItemKind::Issue, 42, text).expect("write body");
        assert_eq!(
            read_body(PROJECT, ItemKind::Issue, 42).as_deref(),
            Some(text)
        );
        assert!(body_path(PROJECT, ItemKind::Pull, 7)
            .to_string_lossy()
            .contains("pull-7.md"));
    }

    #[test]
    fn delete_body_on_missing_file_is_no_op() {
        let _guard = TestCacheGuard::new();
        delete_body(PROJECT, ItemKind::Issue, 999);
    }

    #[test]
    fn write_comments_round_trip() {
        let _guard = TestCacheGuard::new();
        let comments = vec![CachedComment {
            author_login: "alice".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            body: "LGTM".to_string(),
        }];
        write_comments(PROJECT, ItemKind::Pull, 3, &comments).expect("write comments");
        assert_eq!(
            read_comments(PROJECT, ItemKind::Pull, 3).as_ref(),
            Some(&comments)
        );
        delete_comments(PROJECT, ItemKind::Pull, 3);
        assert!(read_comments(PROJECT, ItemKind::Pull, 3).is_none());
    }
}
