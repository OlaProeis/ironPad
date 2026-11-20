# Git remote origin (backend)

Add or replace the `origin` remote on the `data/` repository from the API, with URL validation and an automatic fetch after save.

## API

| Method | Path | Body |
|--------|------|------|
| `POST` | `/api/git/remote` | `{ "url": string }` |

**Success (200):** URL saved; response includes whether `origin` was added or replaced, and optional `fetch_error` if the post-save fetch failed (the new URL is **not** rolled back).

**Validation error (400):** Invalid URL, empty URL, or not a git repo.

See [`API.md`](../API.md) for request/response JSON.

## Service layer

`backend/src/services/git.rs`

| Export | Role |
|--------|------|
| `validate_remote_url(url)` | Non-empty; allows `https://`, `ssh://`, and `git@host:path`; rejects filesystem paths and embedded credentials in HTTP(S) userinfo. |
| `set_remote_origin(url)` | Validates, then `repo.remote("origin", url)` or `repo.remote_set_url("origin", url)`; calls `fetch_from_remote()` once. |
| `SetRemoteOutcome` | `url`, `replaced`, `fetch_error`. |

## URL rules

- **Accept:** `https://github.com/org/repo.git`, `ssh://git@github.com/org/repo.git`, `git@github.com:org/repo.git`
- **Reject:** empty/whitespace, `file://`, absolute filesystem paths, `https://token@host/...` or `https://user:pass@host/...`
- **SSH URL userinfo:** `ssh://git@host/path` is allowed; `user:password@` in userinfo is rejected.

## Route

`backend/src/routes/git.rs` — `GET` and `POST` on `/remote` (mounted under `/api/git`).

## Tests

Table-driven and temp-repo tests live in `tests::remote` in `backend/src/services/git.rs`. See [`git-remote-url-tests.md`](./git-remote-url-tests.md).

```bash
cd backend && cargo test remote:: -- --test-threads=2
```

## Related

- Existing: `GET /api/git/remote`, `POST /api/git/fetch`, push/pull flows (PRD 0.4.0)
- Product scope: [`PRD-0.4.0.md`](../ai-workflow/PRD-0.4.0.md) §9.2