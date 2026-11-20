# Track GitHub issue or pull as a local task

`POST /api/projects/{id}/github/issues/{number}/track` and `POST /api/projects/{id}/github/pulls/{number}/track` create an **Active** task from an open snapshot row, or return the existing task id when that `html_url` is already tracked in the same project. No GitHub API calls.

Routes are on `project_github_router()` in `backend/src/routes/github.rs`. Both handlers call `track_item`.

## Lookup and deduplication

| Step | Behavior |
|------|----------|
| Snapshot | `read_snapshot(id)`. Item must be in `issues` or `pulls` for the given number. Missing snapshot or number → **404** plain text `That item is not in the open snapshot.` |
| Duplicate | Scan `data/projects/{id}/tasks/*.md`; if any task frontmatter `github_url` equals the snapshot `html_url`, return **200** `{ "task_id", "created": false }`. Dedup is **per project** (two projects may track the same URL). |

## New task

1. Task body: cached `snapshot_io::read_body` text plus `\n\n` and `html_url` when body text is non-empty; otherwise body is only `html_url`.
2. `create_task_impl` (`routes/tasks.rs`, `pub(crate)`) with snapshot title, section `Active`, no parent.
3. Re-read the file; insert frontmatter `tags` (label names in order), `due_date` when `milestone_due` is set, `github_url`, `github_number` (integer). Priority is not derived from labels.
4. **201** `{ "task_id", "created": true }`.

`github_url` and `github_number` are written directly in this handler so they survive later meta updates that round-trip unknown keys.

## Response

`TrackItemResponse`: `task_id`, `created` (boolean).

## Tests

`cargo test -- routes::github::track` with temp data dir and `set_test_cache_root`: first track of issue #42 creates a task with tags, `due_date`, and `github_url`; second track returns the same id with `created: false`; a second project gets a different task; unknown number returns 404.