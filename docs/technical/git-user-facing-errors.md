# Git user-facing errors

Push, fetch, and remote-setup failures return a short sentence in JSON. The raw libgit2 or Git text is logged with `tracing::warn` and is not the only UI string.

## Classifier

`backend/src/services/git.rs` calls `map_git_failure` in `backend/src/services/git_errors.rs`.

| Situation | `GitOp` | JSON `message` |
|-----------|---------|----------------|
| Auth failed | any | Could not authenticate. Use an SSH key in the agent, or a Git credential helper for HTTPS. Ironpad does not store tokens. |
| Non-fast-forward or rejected push | `Push` | The remote has commits this copy does not have. Pull first. Ironpad did not force-push. |
| Diverged / not a fast-forward | `Pull`, `Fetch`, `RemoteSet` | The remote and this copy have diverged. Pull did not change your files. |
| Dirty tree | any | Uncommitted changes would be overwritten. Wait for auto-commit, or commit, then pull again. |
| No `origin` | any | No remote yet. Add a private repository URL. |
| Unreachable host | any | Could not reach the remote. |

Unclassified text is logged and returned unchanged. URL validation errors from `validate_remote_url` are not passed through the classifier (the word "credentials" would otherwise look like an auth failure).

`POST /api/git/push` uses the no-remote sentence when `has_remote()` is false. A fetch that fails after `set_remote_origin` is stored in `fetch_error` already mapped. `POST /api/git/pull` uses the same sentences for fetch and checkout failures; see [`git-pull.md`](./git-pull.md). The no-upstream sentence is not passed through this classifier.

## Tests

`cargo test git` covers one stub string per message in `git_errors::tests`.
