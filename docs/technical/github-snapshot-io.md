# GitHub snapshot cache I/O

Per-project GitHub data lives outside `data_dir()` in the OS cache (`github_cache_dir`, see [`github-foundations.md`](./github-foundations.md)). `backend/src/services/github/snapshot_io.rs` reads and writes `snapshot.json`, Markdown bodies, and JSON comment arrays under that tree (PRD §5.3).

## On-disk layout

| Path | Content |
|------|---------|
| `{cache}/snapshot.json` | `Snapshot` (serde JSON) |
| `{cache}/bodies/issue-{n}.md` | Issue body text |
| `{cache}/bodies/pull-{n}.md` | Pull request body text |
| `{cache}/bodies/comments-issue-{n}.json` | `Vec<CachedComment>` |
| `{cache}/bodies/comments-pull-{n}.json` | Same for pulls |

`CachedComment` fields: `author_login`, `created_at`, `body`.

## API (`snapshot_io.rs`)

- **Snapshot:** `read_snapshot` → `Option<Snapshot>` (missing or invalid → `None`); `write_snapshot` (JSON via `filesystem::atomic_write`); `delete_snapshot_dir` (`remove_dir_all`, ignores `NotFound`).
- **Bodies:** `ItemKind::{Issue, Pull}`, `body_path`, `read_body`, `write_body`, `delete_body` (missing file is a no-op).
- **Comments:** `read_comments`, `write_comments`, `delete_comments` (same read/write/delete pattern as bodies).

All paths resolve through `github_cache_dir(project_id)` unless tests override the root (below).

## Tests

```bash
cd backend && cargo test -- services::github::snapshot_io
```

- `tempfile::TempDir` and thread-local `set_test_cache_root` so parallel tests do not share one cache root.
- Tests call `config::init_data_dir()` once because `atomic_write` touches `is_note_file`, which needs an initialized data directory even for cache paths under a temp dir.

## Related

- Types: [`github-foundations.md`](./github-foundations.md) (`Snapshot`, issues, pulls)
- Routes and refresh merge use this module in later GitHub tasks.