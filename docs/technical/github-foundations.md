# GitHub backend foundations (snapshot, cache, token)

Release **0.4.0** GitHub work starts in `backend/src/services/github/`. This module holds shared types and helpers used by later snapshot I/O, REST sync, and routes. HTTP calls and route handlers are not in this layer yet.

## Module layout

| File | Role |
|------|------|
| `mod.rs` | Submodules and re-exports |
| `snapshot.rs` | Serde types for cached issue/PR data |
| `auth.rs` | `resolve_token()` |
| `client.rs` | `GithubClient`, `GithubError` — see [`github-client.md`](./github-client.md) |
| `lists.rs` | `fetch_lists`, `fetch_login`, PRD §5.2 normalization — see [`github-lists.md`](./github-lists.md) |
| `cache_dir.rs` | Per-project OS cache directory |
| `snapshot_io.rs` | Snapshot, body, and comment cache I/O — see [`github-snapshot-io.md`](./github-snapshot-io.md) |

Registered as `pub mod github` in `backend/src/services/mod.rs`.

## Snapshot types (`snapshot.rs`)

All structs derive `Serialize`, `Deserialize`, `Clone`, `Debug` (and `PartialEq` in unit tests).

- **`GithubLabel`** — `name`, `color: Option<String>`
- **`GithubIssue`** — `number`, `title`, `html_url`, `updated_at`, `author_login`, `labels`, `comments_count`, `milestone_due`, `assignee_logins`
- **`GithubPull`** — same as issue fields plus `draft`, `requested_reviewer_logins`, `head_sha`
- **`Snapshot`** — `repo`, `login`, `refreshed_at`, `issue_etag`, `pull_etag`, `issues`, `pulls`, `issues_truncated`, `pulls_truncated`

JSON field names are snake_case. These types match the on-disk snapshot shape defined in `docs/ai-workflow/PRD-0.4.0-github.md`.

## Token resolution (`auth.rs`)

`pub fn resolve_token() -> Result<String, String>` runs in the backend process only. Order (PRD §12):

1. `gh auth token` — subprocess via `Command`, 5 s timeout (`mpsc::recv_timeout` on a worker thread). On exit 0, use trimmed stdout. Never log the token.
2. `GH_TOKEN` environment variable (non-empty after trim)
3. `GITHUB_TOKEN` (same)
4. `Err` with: `GitHub needs a login. Run gh auth login, or set GH_TOKEN for this app. Ironpad does not store a token.`

When `gh` returns a token, it wins even if env vars are set (see PRD).

## Cache directory (`cache_dir.rs`)

`pub fn github_cache_dir(project_id: &str) -> PathBuf`:

- Base: `dirs::data_local_dir()` on **Windows** (LOCALAPPDATA), else `dirs::cache_dir()`
- Path: `{base}/Ironpad/github/{project_id}`
- Calls `fs::create_dir_all` on first use

Snapshot and body files under this directory are written by [`snapshot_io.rs`](./github-snapshot-io.md).

## Tests

Targeted filter:

```bash
cd backend && cargo test -- services::github
```

| Module | What is tested |
|--------|----------------|
| `snapshot.rs` | JSON round-trip equality for a full `Snapshot` |
| `cache_dir.rs` | Path contains `Ironpad` and the project id |
| `auth.rs` | `GH_TOKEN` when `gh` is not on `PATH` (tests narrow `PATH`); error message when no login |
| `lists.rs` | Issues page drops `pull_request` rows; label/null color; milestone prefix; team reviewer skip |
| `snapshot_io.rs` | Snapshot round-trip, `delete_snapshot_dir`, body read/write/delete |

## Out of scope here

- API routes and frontend GitHub view

HTTP client: [`github-client.md`](./github-client.md). List fetch: [`github-lists.md`](./github-lists.md).