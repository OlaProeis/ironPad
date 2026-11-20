//! Map raw Git/libgit2 failures to the user-facing sentences in PRD §9.5.
//! The original error is always logged; callers put only the friendly string in JSON.

/// Which Git operation produced the failure. Push and pull share some libgit2 wording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitOp {
    Push,
    Pull,
    Fetch,
    RemoteSet,
}

impl std::fmt::Display for GitOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            GitOp::Push => "push",
            GitOp::Pull => "pull",
            GitOp::Fetch => "fetch",
            GitOp::RemoteSet => "remote",
        })
    }
}

pub const MSG_AUTH: &str = "Could not authenticate. Use an SSH key in the agent, or a Git credential helper for HTTPS. Ironpad does not store tokens.";
pub const MSG_PUSH_NON_FF: &str =
    "The remote has commits this copy does not have. Pull first. Ironpad did not force-push.";
pub const MSG_PULL_DIVERGED: &str =
    "The remote and this copy have diverged. Pull did not change your files.";
pub const MSG_DIRTY: &str =
    "Uncommitted changes would be overwritten. Wait for auto-commit, or commit, then pull again.";
pub const MSG_NO_REMOTE: &str = "No remote yet. Add a private repository URL.";
pub const MSG_NO_UPSTREAM: &str = "No upstream yet. The first successful push sets it.";
pub const MSG_UNREACHABLE: &str = "Could not reach the remote.";

/// Classify `raw` into a PRD §9.5 sentence. Unknown failures stay as `raw` (still logged).
pub fn map_git_failure(op: GitOp, raw: &str) -> String {
    tracing::warn!(operation = %op, error = %raw, "git operation failed");
    let lower = raw.to_ascii_lowercase();

    if is_no_remote(&lower) {
        return MSG_NO_REMOTE.to_string();
    }
    if is_auth(&lower) {
        return MSG_AUTH.to_string();
    }
    if is_dirty(&lower) {
        return MSG_DIRTY.to_string();
    }
    if is_non_fast_forward(&lower) || is_diverged(&lower) {
        return match op {
            GitOp::Push => MSG_PUSH_NON_FF.to_string(),
            GitOp::Pull => MSG_PULL_DIVERGED.to_string(),
            GitOp::Fetch | GitOp::RemoteSet => MSG_PULL_DIVERGED.to_string(),
        };
    }
    if op == GitOp::Push && lower.contains("rejected") {
        return MSG_PUSH_NON_FF.to_string();
    }
    if is_unreachable(&lower) {
        return MSG_UNREACHABLE.to_string();
    }

    raw.to_string()
}

fn is_no_remote(lower: &str) -> bool {
    lower.contains("no remote")
        || lower.contains("origin' not found")
        || lower.contains("origin not found")
        || lower.contains("remote 'origin' not found")
        || (lower.contains("remote") && lower.contains("not found"))
}

fn is_auth(lower: &str) -> bool {
    lower.contains("authenticat")
        || lower.contains("auth fail")
        || lower.contains("permission denied")
        || lower.contains("publickey")
        || lower.contains("credential")
        || lower.contains("401")
}

fn is_dirty(lower: &str) -> bool {
    lower.contains("would be overwritten")
        || lower.contains("local changes")
        || lower.contains("uncommitted")
        || lower.contains("dirty")
        || lower.contains("unstaged changes")
}

fn is_non_fast_forward(lower: &str) -> bool {
    lower.contains("non-fast-forward")
        || lower.contains("non-fastforward")
        || lower.contains("not a fast-forward")
        || lower.contains("not fast-forward")
        || lower.contains("fast-forwardable")
}

fn is_diverged(lower: &str) -> bool {
    lower.contains("diverged")
}

fn is_unreachable(lower: &str) -> bool {
    lower.contains("failed to connect")
        || lower.contains("could not resolve")
        || lower.contains("couldn't resolve")
        || lower.contains("could not reach")
        || lower.contains("connection refused")
        || lower.contains("network is unreachable")
        || lower.contains("no route to host")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("unable to access")
        || lower.contains("failed to connect to")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_auth_failure() {
        let msg = map_git_failure(
            GitOp::Push,
            "authentication required but no callback set: Auth failed",
        );
        assert_eq!(msg, MSG_AUTH);
    }

    #[test]
    fn maps_non_fast_forward_push() {
        let msg = map_git_failure(
            GitOp::Push,
            "cannot push non-fastforwardable reference: updates were rejected",
        );
        assert_eq!(msg, MSG_PUSH_NON_FF);
    }

    #[test]
    fn maps_rejected_push_without_ff_wording() {
        let msg = map_git_failure(GitOp::Push, "! [rejected] main -> main (fetch first)");
        assert_eq!(msg, MSG_PUSH_NON_FF);
    }

    #[test]
    fn maps_diverged_pull() {
        let msg = map_git_failure(
            GitOp::Pull,
            "pull is not a fast-forward; histories have diverged",
        );
        assert_eq!(msg, MSG_PULL_DIVERGED);
    }

    #[test]
    fn maps_dirty_tree() {
        let msg = map_git_failure(GitOp::Pull, "local changes would be overwritten by merge");
        assert_eq!(msg, MSG_DIRTY);
    }

    #[test]
    fn maps_missing_remote() {
        let msg = map_git_failure(GitOp::Fetch, "Remote 'origin' not found: -3");
        assert_eq!(msg, MSG_NO_REMOTE);
    }

    #[test]
    fn maps_unreachable_host() {
        let msg = map_git_failure(
            GitOp::RemoteSet,
            "failed to connect to github.com: connection timed out",
        );
        assert_eq!(msg, MSG_UNREACHABLE);
    }

    #[test]
    fn unknown_error_is_preserved() {
        let raw = "index is locked";
        assert_eq!(map_git_failure(GitOp::Push, raw), raw);
    }
}
