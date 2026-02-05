use std::time::Duration;

use chrono::Utc;
use git2::{Repository, Signature, StatusOptions};
use serde::Serialize;
use tokio::time::interval;

use crate::config;

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
            timestamp: chrono::DateTime::from_timestamp(commit.time().seconds(), 0)?
                .to_rfc3339(),
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
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    // Get the current branch
    let head = repo.head().map_err(|e| e.to_string())?;
    let branch_name = head
        .shorthand()
        .ok_or_else(|| "Could not get branch name".to_string())?;

    // Find the remote (default to "origin")
    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Remote 'origin' not found: {}", e))?;

    // Check if remote URL is configured
    let remote_url = remote.url().ok_or_else(|| "No remote URL configured".to_string())?;
    if remote_url.is_empty() {
        return Err("No remote URL configured".to_string());
    }

    // Create callbacks for authentication
    let mut callbacks = git2::RemoteCallbacks::new();
    
    // Try to use credential helper from git config
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        // Try SSH agent first
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    // Set up push options
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // Push the current branch
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote
        .push(&[&refspec], Some(&mut push_options))
        .map_err(|e| format!("Push failed: {}. Make sure SSH keys are configured.", e))?;

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
                let diff = repo
                    .diff_tree_to_tree(Some(&pt), Some(&ct), None)
                    .ok();
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

        let timestamp =
            chrono::DateTime::from_timestamp(commit.time().seconds(), 0)
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
    let head_tree = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok());

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
        let upstream_oid = up
            .get()
            .target()
            .unwrap_or_else(git2::Oid::zero);

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

/// Fetch from remote
pub fn fetch_from_remote() -> Result<(), String> {
    let data_path = config::data_dir();
    let repo = Repository::open(data_path).map_err(|e| format!("Not a git repository: {}", e))?;

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Remote 'origin' not found: {}", e))?;

    // Create callbacks for authentication
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    remote
        .fetch(&[] as &[&str], Some(&mut fetch_options), None)
        .map_err(|e| format!("Fetch failed: {}", e))?;

    Ok(())
}
