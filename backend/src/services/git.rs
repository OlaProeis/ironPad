use std::path::Path;
use std::time::Duration;

use chrono::Utc;
use git2::{build::CheckoutBuilder, BranchType, Repository, Signature, StatusOptions};
use serde::Serialize;
use tokio::time::interval;

use crate::config;
use crate::services::git_errors::{self, GitOp};

/// Git status for a file
#[derive(Debug, Clone, Serialize)]
pub struct FileStatus {
    pub path: String,
    pub status: String, // "new", "modified", "deleted", "renamed", "untracked"
}

/// Overall repository status
#[derive(Debug, Serialize)]
pub struct RepoStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub files: Vec<FileStatus>,
    pub has_changes: bool,
    pub last_commit: Option<CommitInfo>,
}

/// Commit information
#[derive(Debug, Clone, Serialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub timestamp: String,
}

/// Extended commit info for history
#[derive(Debug, Serialize)]
pub struct CommitDetail {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
    pub files_changed: usize,
}

/// Diff information
#[derive(Debug, Serialize)]
pub struct DiffInfo {
    pub files: Vec<FileDiff>,
    pub stats: DiffStats,
}

/// File diff
#[derive(Debug, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub status: String,
    pub additions: usize,
    pub deletions: usize,
    pub hunks: Vec<DiffHunk>,
}

/// Diff hunk (section of changes)
#[derive(Debug, Serialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// Single diff line
#[derive(Debug, Serialize)]
pub struct DiffLine {
    pub origin: char,
    pub content: String,
}

/// Diff statistics
#[derive(Debug, Serialize)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

/// Remote repository information
#[derive(Debug, Serialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    pub has_upstream: bool,
    pub ahead: usize,
    pub behind: usize,
}

/// Auto-commit is enabled by default.
/// The background task simply tries to commit every interval;
/// commit_all() already handles "no changes" gracefully.

/// Get repository status
pub fn get_status() -> Result<RepoStatus, String> {
    let data_path = config::data_dir();

    // Try to open as git repo
    let repo = match Repository::open(data_path) {
        Ok(r) => r,
        Err(_) => {
            return Ok(RepoStatus {
                is_repo: false,
                branch: None,
                files: Vec::new(),
                has_changes: false,
                last_commit: None,
            });
        }
    };

    // Get current branch
    let branch = repo
        .head()
        .ok()
        .and_then(|h| h.shorthand().map(String::from));

    // Get file statuses
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .exclude_submodules(true);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;

    let files: Vec<FileStatus> = statuses
        .iter()
        .filter_map(|entry| {
            let path = entry.path()?.to_string();
            let status = entry.status();

            let status_str = if status.is_index_new() || status.is_wt_new() {
                "new"
            } else if status.is_index_modified() || status.is_wt_modified() {
                "modified"
            } else if status.is_index_deleted() || status.is_wt_deleted() {
                "deleted"
            } else if status.is_index_renamed() || status.is_wt_renamed() {
                "renamed"
            } else {
                return None;
            };

            Some(FileStatus {
                path,
                status: status_str.to_string(),
            })
        })
        .collect();

    let has_changes = !files.is_empty();

    // Get last commit info
    let last_commit = repo.head().ok().and_then(|head| {
        let commit = head.peel_to_commit().ok()?;
        Some(CommitInfo {
            id: commit.id().to_string()[..8].to_string(),
            message: commit.message()?.trim().to_string(),
            timestamp: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)?.to_rfc3339(),
        })
    });

    Ok(RepoStatus {
        is_repo: true,
        branch,
        files,
        has_changes,
        last_commit,
    })
}

/// Create a commit with all changes
pub fn commit_all(message: Option<&str>) -> Result<CommitInfo, String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    // Stage all changes
    let mut index = repo.index().map_err(|e| e.to_string())?;
    index
        .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;

    // Check if there are changes to commit
    let tree_id = index.write_tree().map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_id).map_err(|e| e.to_string())?;

    // Get parent commit (if any)
    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    // Check if tree is different from parent
    if let Some(ref p) = parent {
        if p.tree().map(|t| t.id()) == Ok(tree_id) {
            return Err("No changes to commit".to_string());
        }
    }

    // Create signature
    let sig = Signature::now("Ironpad", "ironpad@local").map_err(|e| e.to_string())?;

    // Generate commit message
    let msg = message.unwrap_or_else(|| "Auto-save");
    let timestamp = Utc::now().format("%Y-%m-%d %H:%M");
    let full_message = format!("{} ({})", msg, timestamp);

    // Create commit
    let parents: Vec<&git2::Commit> = parent.as_ref().map(|p| vec![p]).unwrap_or_default();
    let commit_id = repo
        .commit(Some("HEAD"), &sig, &sig, &full_message, &tree, &parents)
        .map_err(|e| e.to_string())?;

    Ok(CommitInfo {
        id: commit_id.to_string()[..8].to_string(),
        message: full_message,
        timestamp: Utc::now().to_rfc3339(),
    })
}

/// Initialize data directory as a git repository if not already
pub fn init_repo() -> Result<(), String> {
    let data_path = config::data_dir();

    if Repository::open(data_path).is_ok() {
        return Ok(()); // Already a repo
    }

    Repository::init(data_path).map_err(|e| format!("Failed to init repo: {}", e))?;

    // Create initial .gitignore
    let gitignore_path = data_path.join(".gitignore");
    if !gitignore_path.exists() {
        std::fs::write(&gitignore_path, "*.tmp\n.DS_Store\n")
            .map_err(|e| format!("Failed to create .gitignore: {}", e))?;
    }

    // Initial commit
    commit_all(Some("Initial commit"))?;

    Ok(())
}

/// Check for merge conflicts
pub fn check_conflicts() -> Result<Vec<String>, String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    let mut conflicts = Vec::new();

    // Check for .git/index.lock (another git operation in progress)
    let lock_path = data_path.join(".git").join("index.lock");
    if lock_path.exists() {
        // This isn't a conflict per se, but indicates git is busy
        tracing::warn!("Git index.lock exists - another operation may be in progress");
    }

    // Check status for conflicted files
    let mut opts = StatusOptions::new();
    opts.include_untracked(false);

    let statuses = repo.statuses(Some(&mut opts)).map_err(|e| e.to_string())?;

    for entry in statuses.iter() {
        let status = entry.status();
        // Check for conflict status flags
        if status.is_conflicted() {
            if let Some(path) = entry.path() {
                conflicts.push(path.to_string());
            }
        }
    }

    // Also check the index for conflicts
    let index = repo.index().map_err(|e| e.to_string())?;
    if index.has_conflicts() {
        for conflict in index.conflicts().map_err(|e| e.to_string())? {
            if let Ok(conflict) = conflict {
                if let Some(ancestor) = conflict.ancestor {
                    if let Some(path) = std::str::from_utf8(&ancestor.path).ok() {
                        if !conflicts.contains(&path.to_string()) {
                            conflicts.push(path.to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(conflicts)
}

/// Push to remote repository
pub fn push_to_remote() -> Result<(), String> {
    push_in(&config::data_dir())
}

fn push_in(path: &Path) -> Result<(), String> {
    let repo = Repository::open(path).map_err(|e| {
        git_errors::map_git_failure(GitOp::Push, &format!("Not a git repository: {e}"))
    })?;

    // Get the current branch
    let head = repo
        .head()
        .map_err(|e| git_errors::map_git_failure(GitOp::Push, &e.to_string()))?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| git_errors::map_git_failure(GitOp::Push, "Could not get branch name"))?;

    // Find the remote (default to "origin")
    let mut remote = repo.find_remote("origin").map_err(|e| {
        git_errors::map_git_failure(GitOp::Push, &format!("Remote 'origin' not found: {e}"))
    })?;

    // Check if remote URL is configured
    let remote_url = remote
        .url()
        .ok_or_else(|| git_errors::map_git_failure(GitOp::Push, "No remote URL configured"))?;
    if remote_url.is_empty() {
        return Err(git_errors::map_git_failure(
            GitOp::Push,
            "No remote URL configured",
        ));
    }

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(remote_credentials);

    // Set up push options
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // Push the current branch
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote
        .push(&[&refspec], Some(&mut push_options))
        .map_err(|e| git_errors::map_git_failure(GitOp::Push, &format!("Push failed: {e}")))?;

    let mut local_branch = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|e| git_errors::map_git_failure(GitOp::Push, &e.to_string()))?;
    if local_branch.upstream().is_err() {
        let upstream = format!("origin/{branch_name}");
        local_branch
            .set_upstream(Some(upstream.as_str()))
            .map_err(|e| git_errors::map_git_failure(GitOp::Push, &e.to_string()))?;
    }

    tracing::info!("Successfully pushed to origin/{}", branch_name);
    Ok(())
}

/// Check if remote is configured
pub fn has_remote() -> bool {
    let data_path = config::data_dir();
    if let Ok(repo) = Repository::open(data_path) {
        if let Ok(remote) = repo.find_remote("origin") {
            return remote.url().is_some();
        }
    }
    false
}

/// Start auto-commit background task.
/// Tries to commit every 60 seconds; commit_all() already handles "no changes" gracefully.
pub fn start_auto_commit() {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            match commit_all(Some("Auto-save")) {
                Ok(info) => {
                    tracing::info!("Auto-commit: {} - {}", info.id, info.message);
                }
                Err(e) => {
                    if !e.contains("No changes") {
                        tracing::warn!("Auto-commit failed: {}", e);
                    }
                }
            }
        }
    });
}

/// Get commit history (most recent first)
pub fn get_log(limit: Option<usize>) -> Result<Vec<CommitDetail>, String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;
    revwalk
        .set_sorting(git2::Sort::TIME)
        .map_err(|e| e.to_string())?;

    let max_commits = limit.unwrap_or(50);
    let mut commits = Vec::new();

    for (i, oid_result) in revwalk.enumerate() {
        if i >= max_commits {
            break;
        }

        let oid = oid_result.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

        // Count files changed in this commit
        let files_changed = if commit.parent_count() > 0 {
            let parent = commit.parent(0).ok();
            let parent_tree = parent.as_ref().and_then(|p| p.tree().ok());
            let commit_tree = commit.tree().ok();

            if let (Some(pt), Some(ct)) = (parent_tree, commit_tree) {
                let diff = repo.diff_tree_to_tree(Some(&pt), Some(&ct), None).ok();
                diff.map(|d| d.deltas().count()).unwrap_or(0)
            } else {
                0
            }
        } else {
            // Initial commit - count all files
            commit
                .tree()
                .ok()
                .map(|t| count_tree_entries(&t))
                .unwrap_or(0)
        };

        let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Unknown".to_string());

        commits.push(CommitDetail {
            id: oid.to_string(),
            short_id: oid.to_string()[..8].to_string(),
            message: commit.message().unwrap_or("").trim().to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            timestamp,
            files_changed,
        });
    }

    Ok(commits)
}

/// Get commit history that touched a specific file path (most recent first).
pub fn get_log_for_path(path: &str, limit: Option<usize>) -> Result<Vec<CommitDetail>, String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    let normalized_path = path.replace('\\', "/").trim_start_matches('/').to_string();
    let mut revwalk = repo.revwalk().map_err(|e| e.to_string())?;
    revwalk.push_head().map_err(|e| e.to_string())?;
    revwalk
        .set_sorting(git2::Sort::TIME)
        .map_err(|e| e.to_string())?;

    let max_commits = limit.unwrap_or(50);
    let mut commits = Vec::new();

    for oid_result in revwalk {
        if commits.len() >= max_commits {
            break;
        }

        let oid = oid_result.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

        let commit_tree = commit.tree().ok();
        let parent_tree = if commit.parent_count() > 0 {
            commit.parent(0).ok().and_then(|p| p.tree().ok())
        } else {
            None
        };

        let Some(ct) = commit_tree else {
            continue;
        };

        let diff = repo
            .diff_tree_to_tree(parent_tree.as_ref(), Some(&ct), None)
            .map_err(|e| e.to_string())?;

        let mut touched = false;
        let mut files_changed = 0usize;
        for delta in diff.deltas() {
            files_changed += 1;
            let candidate = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();
            if candidate == normalized_path {
                touched = true;
            }
        }
        if !touched {
            continue;
        }

        let timestamp = chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "Unknown".to_string());

        commits.push(CommitDetail {
            id: oid.to_string(),
            short_id: oid.to_string()[..8].to_string(),
            message: commit.message().unwrap_or("").trim().to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            timestamp,
            files_changed,
        });
    }

    Ok(commits)
}

/// Helper to count entries in a tree recursively
fn count_tree_entries(tree: &git2::Tree) -> usize {
    tree.iter()
        .filter(|entry| entry.kind() == Some(git2::ObjectType::Blob))
        .count()
}

/// Get working directory diff (uncommitted changes)
pub fn get_working_diff() -> Result<DiffInfo, String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    // Get HEAD tree (or empty tree if no commits)
    let head_tree = repo.head().ok().and_then(|h| h.peel_to_tree().ok());

    // Diff against working directory
    let diff = repo
        .diff_tree_to_workdir_with_index(head_tree.as_ref(), None)
        .map_err(|e| e.to_string())?;

    parse_diff(&diff)
}

/// Get diff for a specific commit
pub fn get_commit_diff(commit_id: &str) -> Result<DiffInfo, String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    let oid = git2::Oid::from_str(commit_id).map_err(|e| format!("Invalid commit ID: {}", e))?;
    let commit = repo
        .find_commit(oid)
        .map_err(|e| format!("Commit not found: {}", e))?;

    let commit_tree = commit.tree().map_err(|e| e.to_string())?;

    let parent_tree = if commit.parent_count() > 0 {
        commit.parent(0).ok().and_then(|p| p.tree().ok())
    } else {
        None
    };

    let diff = repo
        .diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)
        .map_err(|e| e.to_string())?;

    parse_diff(&diff)
}

/// Parse a git2::Diff into our DiffInfo structure
fn parse_diff(diff: &git2::Diff) -> Result<DiffInfo, String> {
    let stats = diff.stats().map_err(|e| e.to_string())?;

    let mut files = Vec::new();

    for delta_idx in 0..diff.deltas().count() {
        let delta = diff.get_delta(delta_idx).ok_or("Missing delta")?;

        let path = delta
            .new_file()
            .path()
            .or_else(|| delta.old_file().path())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let status = match delta.status() {
            git2::Delta::Added => "added",
            git2::Delta::Deleted => "deleted",
            git2::Delta::Modified => "modified",
            git2::Delta::Renamed => "renamed",
            git2::Delta::Copied => "copied",
            _ => "unknown",
        };

        let mut hunks = Vec::new();
        let mut additions = 0;
        let mut deletions = 0;

        // Get patch for this file
        if let Ok(patch) = git2::Patch::from_diff(diff, delta_idx) {
            if let Some(p) = patch {
                for hunk_idx in 0..p.num_hunks() {
                    if let Ok((hunk, _)) = p.hunk(hunk_idx) {
                        let mut lines = Vec::new();

                        for line_idx in 0..p.num_lines_in_hunk(hunk_idx).unwrap_or(0) {
                            if let Ok(line) = p.line_in_hunk(hunk_idx, line_idx) {
                                let origin = line.origin();
                                let content = std::str::from_utf8(line.content())
                                    .unwrap_or("")
                                    .to_string();

                                match origin {
                                    '+' => additions += 1,
                                    '-' => deletions += 1,
                                    _ => {}
                                }

                                lines.push(DiffLine { origin, content });
                            }
                        }

                        hunks.push(DiffHunk {
                            header: std::str::from_utf8(hunk.header())
                                .unwrap_or("")
                                .trim()
                                .to_string(),
                            lines,
                        });
                    }
                }
            }
        }

        files.push(FileDiff {
            path,
            status: status.to_string(),
            additions,
            deletions,
            hunks,
        });
    }

    Ok(DiffInfo {
        files,
        stats: DiffStats {
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
        },
    })
}

/// Get remote repository information
pub fn get_remote_info() -> Result<Option<RemoteInfo>, String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    let remote = match repo.find_remote("origin") {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let url = remote.url().unwrap_or("").to_string();
    if url.is_empty() {
        return Ok(None);
    }

    // Get current branch
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => {
            return Ok(Some(RemoteInfo {
                name: "origin".to_string(),
                url,
                has_upstream: false,
                ahead: 0,
                behind: 0,
            }));
        }
    };

    let branch_name = head.shorthand().unwrap_or("HEAD");

    // Try to find upstream branch
    let local_branch = repo.find_branch(branch_name, git2::BranchType::Local).ok();
    let upstream = local_branch.as_ref().and_then(|b| b.upstream().ok());

    let (ahead, behind) = if let Some(ref up) = upstream {
        // Calculate ahead/behind
        let local_oid = head.target().unwrap_or_else(git2::Oid::zero);
        let upstream_oid = up.get().target().unwrap_or_else(git2::Oid::zero);

        repo.graph_ahead_behind(local_oid, upstream_oid)
            .unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    Ok(Some(RemoteInfo {
        name: "origin".to_string(),
        url,
        has_upstream: upstream.is_some(),
        ahead,
        behind,
    }))
}

/// Result of saving `origin` and optionally fetching.
#[derive(Debug, Serialize)]
pub struct SetRemoteOutcome {
    pub url: String,
    pub replaced: bool,
    pub fetch_error: Option<String>,
}

/// Validate a Git remote URL: allowed forms only, no embedded credentials or filesystem paths.
pub fn validate_remote_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() {
        return Err("Remote URL cannot be empty".to_string());
    }

    if url.starts_with("file://") {
        return Err(
            "Filesystem paths are not allowed. Use https://, ssh://, or git@host:path.".to_string(),
        );
    }

    let bytes = url.as_bytes();
    if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        return Err(
            "Filesystem paths are not allowed. Use https://, ssh://, or git@host:path.".to_string(),
        );
    }

    if url.starts_with('/') && !url.starts_with("//") {
        return Err(
            "Filesystem paths are not allowed. Use https://, ssh://, or git@host:path.".to_string(),
        );
    }

    // SCP-style: git@host:path (no scheme, single path colon after host)
    if !url.contains("://") {
        if let Some(at) = url.find('@') {
            let host_path = &url[at + 1..];
            if host_path.contains(':') && !host_path.contains('\\') {
                return Ok(());
            }
        }
        return Err("Invalid remote URL. Use https://, ssh://, or git@host:path.".to_string());
    }

    if url.starts_with("https://") || url.starts_with("http://") || url.starts_with("ssh://") {
        if forbidden_userinfo_in_scheme_url(url) {
            return Err(
                "URLs must not embed credentials. Use SSH or a Git credential helper instead."
                    .to_string(),
            );
        }
        return Ok(());
    }

    Err("Invalid remote URL. Use https://, ssh://, or git@host:path.".to_string())
}

/// True when scheme-based URL authority contains disallowed userinfo.
fn forbidden_userinfo_in_scheme_url(url: &str) -> bool {
    let scheme_end = url.find("://").map(|i| i + 3).unwrap_or(0);
    let rest = &url[scheme_end..];
    let authority_end = rest.find('/').unwrap_or(rest.len());
    let authority = &rest[..authority_end];

    let Some(at) = authority.find('@') else {
        return false;
    };
    let userinfo = &authority[..at];
    if userinfo.is_empty() {
        return false;
    }

    if url.starts_with("https://") || url.starts_with("http://") {
        return true;
    }

    // ssh:// — reject embedded passwords (user:pass@), allow e.g. ssh://git@host/path
    userinfo.contains(':')
}

/// Add or replace `origin` with `url`, then fetch once (fetch errors are returned but not rolled back).
pub fn set_remote_origin(url: &str) -> Result<SetRemoteOutcome, String> {
    let url = url.trim();
    validate_remote_url(url)?;

    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| {
        git_errors::map_git_failure(GitOp::RemoteSet, &format!("Not a git repository: {e}"))
    })?;

    let replaced = repo.find_remote("origin").is_ok();

    if replaced {
        repo.remote_set_url("origin", url).map_err(|e| {
            git_errors::map_git_failure(
                GitOp::RemoteSet,
                &format!("Failed to update remote URL: {e}"),
            )
        })?;
    } else {
        repo.remote("origin", url).map_err(|e| {
            git_errors::map_git_failure(
                GitOp::RemoteSet,
                &format!("Failed to add remote 'origin': {e}"),
            )
        })?;
    }

    let fetch_error = match fetch_from_remote() {
        Ok(()) => None,
        Err(e) => Some(e),
    };

    Ok(SetRemoteOutcome {
        url: url.to_string(),
        replaced,
        fetch_error,
    })
}

/// How a remote URL authenticates. HTTPS uses the Git credential helper; SSH uses the agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemoteAuthKind {
    SshAgent,
    HttpsCredentialHelper,
}

fn remote_auth_kind(url: &str) -> RemoteAuthKind {
    let scheme = url.trim().to_ascii_lowercase();
    if scheme.starts_with("https://") || scheme.starts_with("http://") {
        RemoteAuthKind::HttpsCredentialHelper
    } else {
        RemoteAuthKind::SshAgent
    }
}

/// SSH agent for SSH / scp-style URLs; `credential.helper` (gitcredentials) for HTTP(S).
fn remote_credentials(
    url: &str,
    username_from_url: Option<&str>,
    _allowed_types: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    match remote_auth_kind(url) {
        RemoteAuthKind::SshAgent => {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        }
        RemoteAuthKind::HttpsCredentialHelper => {
            let config = git2::Config::open_default()?;
            git2::Cred::credential_helper(&config, url, username_from_url)
        }
    }
}

/// Fetch from remote
pub fn fetch_from_remote() -> Result<(), String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| {
        git_errors::map_git_failure(GitOp::Fetch, &format!("Not a git repository: {e}"))
    })?;
    fetch_in(&repo)
}

fn fetch_in(repo: &Repository) -> Result<(), String> {
    let mut remote = repo.find_remote("origin").map_err(|e| {
        git_errors::map_git_failure(GitOp::Fetch, &format!("Remote 'origin' not found: {e}"))
    })?;

    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(remote_credentials);

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    remote
        .fetch(&[] as &[&str], Some(&mut fetch_options), None)
        .map_err(|e| git_errors::map_git_failure(GitOp::Fetch, &format!("Fetch failed: {e}")))?;

    Ok(())
}

/// Outcome of a fast-forward pull.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullOutcome {
    pub commits: usize,
    pub message: String,
}

/// Fetch, then fast-forward the current branch to its upstream.
/// Diverged histories and checkouts that would overwrite local edits are refused.
pub fn pull_from_remote() -> Result<PullOutcome, String> {
    pull_in(&config::data_dir())
}

fn pull_in(path: &Path) -> Result<PullOutcome, String> {
    let repo = Repository::open(path).map_err(|e| {
        git_errors::map_git_failure(GitOp::Pull, &format!("Not a git repository: {e}"))
    })?;

    if repo.find_remote("origin").is_err() {
        return Err(git_errors::MSG_NO_REMOTE.to_string());
    }

    let head = repo
        .head()
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    if !head.is_branch() {
        return Err(git_errors::map_git_failure(
            GitOp::Pull,
            "HEAD is not a branch",
        ));
    }
    let branch_name = head
        .shorthand()
        .ok_or_else(|| git_errors::map_git_failure(GitOp::Pull, "Could not get branch name"))?;

    let local = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    if local.upstream().is_err() {
        return Err(git_errors::MSG_NO_UPSTREAM.to_string());
    }

    let local_oid = head
        .target()
        .ok_or_else(|| git_errors::map_git_failure(GitOp::Pull, "Current branch has no commit"))?;

    fetch_in(&repo)?;

    let local = repo
        .find_branch(branch_name, BranchType::Local)
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    let upstream = local
        .upstream()
        .map_err(|_| git_errors::MSG_NO_UPSTREAM.to_string())?;
    let their_oid = upstream
        .get()
        .target()
        .ok_or_else(|| git_errors::map_git_failure(GitOp::Pull, "Upstream branch has no commit"))?;

    let annotated = repo
        .find_annotated_commit(their_oid)
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    let (analysis, _) = repo
        .merge_analysis(&[&annotated])
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;

    if analysis.is_up_to_date() {
        return Ok(PullOutcome {
            commits: 0,
            message: "Already up to date.".to_string(),
        });
    }

    if !analysis.is_fast_forward() {
        return Err(git_errors::MSG_PULL_DIVERGED.to_string());
    }

    let (_, behind) = repo
        .graph_ahead_behind(local_oid, their_oid)
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;

    let their_commit = repo
        .find_commit(their_oid)
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    let tree = their_commit
        .tree()
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    repo.checkout_tree(tree.as_object(), Some(&mut CheckoutBuilder::new()))
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;

    let refname = format!("refs/heads/{branch_name}");
    let mut reference = repo
        .find_reference(&refname)
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    reference
        .set_target(their_oid, "fast-forward pull")
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;
    repo.set_head(&refname)
        .map_err(|e| git_errors::map_git_failure(GitOp::Pull, &e.to_string()))?;

    let message = if behind == 1 {
        "Pulled 1 commit.".to_string()
    } else {
        format!("Pulled {behind} commits.")
    };
    Ok(PullOutcome {
        commits: behind,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ironpad-pull-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    mod remote {
        use super::*;
        use git2::Repository;

        struct ValidateCase {
            url: &'static str,
            expect_ok: bool,
            err_substr: Option<&'static str>,
        }

        #[test]
        fn validate_remote_url_table() {
            let cases = [
                ValidateCase {
                    url: "https://github.com/org/repo.git",
                    expect_ok: true,
                    err_substr: None,
                },
                ValidateCase {
                    url: "  https://github.com/org/repo.git  ",
                    expect_ok: true,
                    err_substr: None,
                },
                ValidateCase {
                    url: "http://git.example.com/org/repo.git",
                    expect_ok: true,
                    err_substr: None,
                },
                ValidateCase {
                    url: "ssh://git@github.com/org/repo.git",
                    expect_ok: true,
                    err_substr: None,
                },
                ValidateCase {
                    url: "git@github.com:org/repo.git",
                    expect_ok: true,
                    err_substr: None,
                },
                ValidateCase {
                    url: "",
                    expect_ok: false,
                    err_substr: Some("empty"),
                },
                ValidateCase {
                    url: "   ",
                    expect_ok: false,
                    err_substr: Some("empty"),
                },
                ValidateCase {
                    url: "/home/user/repo",
                    expect_ok: false,
                    err_substr: Some("Filesystem"),
                },
                ValidateCase {
                    url: "file:///tmp/repo",
                    expect_ok: false,
                    err_substr: Some("Filesystem"),
                },
                ValidateCase {
                    url: "C:\\Users\\dev\\repo",
                    expect_ok: false,
                    err_substr: Some("Filesystem"),
                },
                ValidateCase {
                    url: "https://token@github.com/org/repo.git",
                    expect_ok: false,
                    err_substr: Some("credentials"),
                },
                ValidateCase {
                    url: "https://user:pass@github.com/org/repo.git",
                    expect_ok: false,
                    err_substr: Some("credentials"),
                },
                ValidateCase {
                    url: "ssh://user:secret@github.com/org/repo.git",
                    expect_ok: false,
                    err_substr: Some("credentials"),
                },
                ValidateCase {
                    url: "ftp://host/repo.git",
                    expect_ok: false,
                    err_substr: Some("Invalid remote URL"),
                },
                ValidateCase {
                    url: "git@github.com",
                    expect_ok: false,
                    err_substr: Some("Invalid remote URL"),
                },
            ];

            for case in cases {
                let result = validate_remote_url(case.url);
                if case.expect_ok {
                    assert!(
                        result.is_ok(),
                        "expected {:?} to be accepted: {:?}",
                        case.url,
                        result
                    );
                } else {
                    let err = result.unwrap_err();
                    if let Some(sub) = case.err_substr {
                        assert!(
                            err.contains(sub),
                            "url {:?}: expected err containing {:?}, got {:?}",
                            case.url,
                            sub,
                            err
                        );
                    }
                }
            }
        }

        fn add_origin_if_valid(repo: &Repository, url: &str) -> Result<(), String> {
            validate_remote_url(url)?;
            let trimmed = url.trim();
            if repo.find_remote("origin").is_ok() {
                repo.remote_set_url("origin", trimmed)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            } else {
                repo.remote("origin", trimmed)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }
        }

        #[test]
        fn valid_https_url_adds_origin_on_temp_repo() {
            let dir = super::scratch_dir("remote-add");
            let repo = Repository::init(&dir).unwrap();
            let url = "https://example.invalid/org/repo.git";
            add_origin_if_valid(&repo, url).unwrap();
            let origin = repo.find_remote("origin").unwrap();
            assert_eq!(origin.url(), Some(url));
            assert!(repo.find_remote("origin").is_ok());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn valid_urls_add_origin_without_network() {
            let urls = [
                "https://example.invalid/a.git",
                "ssh://git@example.invalid/a.git",
                "git@example.invalid:org/a.git",
            ];
            for url in urls {
                let dir = super::scratch_dir("remote-schemes");
                let repo = Repository::init(&dir).unwrap();
                add_origin_if_valid(&repo, url).unwrap();
                assert_eq!(repo.find_remote("origin").unwrap().url(), Some(url));
                let _ = std::fs::remove_dir_all(&dir);
            }
        }

        #[test]
        fn invalid_url_leaves_repo_without_origin() {
            let dir = super::scratch_dir("remote-reject");
            let repo = Repository::init(&dir).unwrap();
            let bad = "https://user:pass@github.com/org/repo.git";
            assert!(add_origin_if_valid(&repo, bad).is_err());
            assert!(repo.find_remote("origin").is_err());
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn replace_origin_url_when_valid() {
            let dir = super::scratch_dir("remote-replace");
            let repo = Repository::init(&dir).unwrap();
            let first = "https://example.invalid/first.git";
            let second = "git@example.invalid:org/second.git";
            add_origin_if_valid(&repo, first).unwrap();
            add_origin_if_valid(&repo, second).unwrap();
            assert_eq!(repo.find_remote("origin").unwrap().url(), Some(second));
            let _ = std::fs::remove_dir_all(&dir);
        }

        #[test]
        fn set_remote_origin_rejects_before_repo_access() {
            assert!(set_remote_origin("").unwrap_err().contains("empty"));
            let cred_err = set_remote_origin("https://token@host/x.git").unwrap_err();
            assert!(cred_err.contains("credentials"));
            assert!(set_remote_origin("/abs/path")
                .unwrap_err()
                .contains("Filesystem"));
        }
    }

    #[test]
    fn auth_kind_https_uses_credential_helper() {
        assert_eq!(
            remote_auth_kind("https://github.com/org/repo.git"),
            RemoteAuthKind::HttpsCredentialHelper
        );
        assert_eq!(
            remote_auth_kind("http://git.example.com/org/repo.git"),
            RemoteAuthKind::HttpsCredentialHelper
        );
    }

    #[test]
    fn auth_kind_ssh_uses_agent() {
        assert_eq!(
            remote_auth_kind("ssh://git@github.com/org/repo.git"),
            RemoteAuthKind::SshAgent
        );
        assert_eq!(
            remote_auth_kind("git@github.com:org/repo.git"),
            RemoteAuthKind::SshAgent
        );
    }

    fn commit_file(repo: &Repository, name: &str, contents: &str, message: &str) {
        let workdir = repo.workdir().unwrap();
        std::fs::write(workdir.join(name), contents).unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new(name)).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = parent.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .unwrap();
    }

    fn init_with_file(path: &std::path::Path, name: &str, contents: &str) -> Repository {
        let repo = Repository::init(path).unwrap();
        commit_file(&repo, name, contents, "initial");
        repo
    }

    #[test]
    fn pull_fast_forward_brings_commits() {
        let origin_dir = scratch_dir("ff-origin");
        let clone_dir = scratch_dir("ff-clone");
        let origin = init_with_file(&origin_dir, "note.md", "one\n");
        let url = origin_dir.to_string_lossy().to_string();
        git2::build::RepoBuilder::new()
            .clone(&url, &clone_dir)
            .unwrap();
        commit_file(&origin, "note.md", "one\ntwo\n", "second");

        let outcome = pull_in(&clone_dir).unwrap();
        assert_eq!(outcome.commits, 1);
        assert_eq!(outcome.message, "Pulled 1 commit.");
        let body = std::fs::read_to_string(clone_dir.join("note.md")).unwrap();
        assert_eq!(body.replace("\r\n", "\n"), "one\ntwo\n");

        let again = pull_in(&clone_dir).unwrap();
        assert_eq!(again.commits, 0);
        assert_eq!(again.message, "Already up to date.");

        let _ = std::fs::remove_dir_all(&origin_dir);
        let _ = std::fs::remove_dir_all(&clone_dir);
    }

    #[test]
    fn pull_diverged_leaves_worktree_unchanged() {
        let origin_dir = scratch_dir("div-origin");
        let clone_dir = scratch_dir("div-clone");
        let origin = init_with_file(&origin_dir, "note.md", "base\n");
        let url = origin_dir.to_string_lossy().to_string();
        git2::build::RepoBuilder::new()
            .clone(&url, &clone_dir)
            .unwrap();

        commit_file(&origin, "note.md", "from-origin\n", "origin side");
        let clone = Repository::open(&clone_dir).unwrap();
        commit_file(&clone, "local.md", "from-clone\n", "clone side");
        let head_before = clone.head().unwrap().target().unwrap();
        drop(clone);

        let err = pull_in(&clone_dir).unwrap_err();
        assert_eq!(err, git_errors::MSG_PULL_DIVERGED);
        assert_eq!(
            std::fs::read_to_string(clone_dir.join("note.md"))
                .unwrap()
                .replace("\r\n", "\n"),
            "base\n"
        );
        assert!(clone_dir.join("local.md").exists());
        let clone = Repository::open(&clone_dir).unwrap();
        assert_eq!(clone.head().unwrap().target().unwrap(), head_before);

        let _ = std::fs::remove_dir_all(&origin_dir);
        let _ = std::fs::remove_dir_all(&clone_dir);
    }

    #[test]
    fn pull_without_upstream_reports_push_sets_it() {
        let dir = scratch_dir("no-up");
        let repo = init_with_file(&dir, "note.md", "x\n");
        repo.remote("origin", "https://example.invalid/repo.git")
            .unwrap();
        let err = pull_in(&dir).unwrap_err();
        assert_eq!(err, git_errors::MSG_NO_UPSTREAM);
        assert_eq!(
            std::fs::read_to_string(dir.join("note.md"))
                .unwrap()
                .replace("\r\n", "\n"),
            "x\n"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn push_sets_upstream_on_first_success() {
        let bare_dir = scratch_dir("push-bare");
        let local_dir = scratch_dir("push-local");
        let _bare = Repository::init_bare(&bare_dir).unwrap();
        let local = init_with_file(&local_dir, "note.md", "x\n");
        let url = bare_dir.to_string_lossy().to_string();
        local.remote("origin", &url).unwrap();

        let branch_name = local.head().unwrap().shorthand().unwrap().to_string();
        let local_branch = local.find_branch(&branch_name, BranchType::Local).unwrap();
        assert!(local_branch.upstream().is_err());

        push_in(&local_dir).unwrap();

        let repo = Repository::open(&local_dir).unwrap();
        let branch = repo.find_branch(&branch_name, BranchType::Local).unwrap();
        let upstream = branch.upstream().unwrap();
        let expected_upstream = format!("origin/{branch_name}");
        assert_eq!(
            upstream.name().unwrap().as_deref(),
            Some(expected_upstream.as_str())
        );

        let head_oid = repo.head().unwrap().target().unwrap();
        let upstream_oid = upstream.get().target().unwrap();
        let (ahead, behind) = repo.graph_ahead_behind(head_oid, upstream_oid).unwrap();
        assert_eq!(ahead, 0);
        assert_eq!(behind, 0);

        let _ = std::fs::remove_dir_all(&bare_dir);
        let _ = std::fs::remove_dir_all(&local_dir);
    }
}
