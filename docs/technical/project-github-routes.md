# Project GitHub link and snapshot GET/PUT

Read linked `owner/name` state from the on-disk snapshot and save or clear `github_repo` on the project `index.md`. These routes do **not** call the GitHub API. Refresh is [`github-refresh.md`](./github-refresh.md).

## Router

- Module: `backend/src/routes/github.rs`
- Export: `project_github_router()`
- Registration: second `.nest("/projects", routes::github::project_github_router())` in `main.rs` beside the existing projects router (paths merge under `/api/projects`).

## `GET /api/projects/{id}/github`

- Calls `snapshot_io::read_snapshot(id)` only.
- Response type: `GithubSnapshotResponse` (JSON).

| Field | Source |
|-------|--------|
| `repo`, `login`, `refreshed_at`, issues, pulls, `issues_truncated`, `pulls_truncated` | `Snapshot` when file exists |
| Missing snapshot | `repo`/`login`/`refreshed_at` null, empty issue/PR lists, truncation flags false |
| `stale` | `true` if no snapshot, `refreshed_at` missing, unparsable RFC3339, or older than 15 minutes UTC |
| `error` | Always `null` on GET. Refresh may set it when GitHub fails and a snapshot exists ([`github-refresh.md`](./github-refresh.md)) |

Always **200** when the route matches (no GitHub call).

## `PUT /api/projects/{id}/github`

Body: `{ "repo": "owner/name" }` or `{ "repo": null }`.

| Input | Behavior |
|-------|----------|
| Valid string | Trim; validate (see below); set `github_repo` in frontmatter via `parse_frontmatter` + `serialize_frontmatter` + `atomic_write`; **200** `{ "repo": "<validated>" }`. Does not refresh snapshot. |
| `null` / absent link | `delete_snapshot_dir(id)`; remove `github_repo` key from frontmatter; **204** empty body. |
| Invalid string | **400** plain text: `Use owner/name, with no URL and no .git suffix.` (`GITHUB_REPO_VALIDATION_MSG`) |
| Missing project `index.md` | **404** |

Validation (`validate_github_repo`): one `/`; regex `^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$`; reject empty, `://`, `.git` suffix, `..`, or more than one slash.

Frontmatter persistence matches `routes/projects.rs` patterns. List/detail GETs still expose `github_repo` as in [`project-github-repo.md`](./project-github-repo.md).

## Tests

```bash
cd backend && cargo test -- routes::github
```

Axum `oneshot` + `http_body_util::BodyExt`. Tests use one `IRONPAD_DATA_DIR` init per process (`Once` + mutex), same constraint as other route tests; snapshot tests use `snapshot_io::set_test_cache_root`.

Scenarios: GET with no snapshot (stale, empty lists); PUT save; PUT URL and `.git` 400; PUT null clears frontmatter and 204; GET after `write_snapshot` with recent `refreshed_at` (not stale, lists populated).

## Related

- Snapshot I/O: [`github-snapshot-io.md`](./github-snapshot-io.md)
- Types: [`github-foundations.md`](./github-foundations.md)
- PRD: [`../ai-workflow/PRD-0.4.0-github.md`](../ai-workflow/PRD-0.4.0-github.md) §10