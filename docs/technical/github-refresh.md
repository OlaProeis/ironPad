# GitHub snapshot refresh

`POST /api/projects/{id}/github/refresh` loads the linked `github_repo`, calls GitHub, and writes `snapshot.json`. The JSON body matches `GET /api/projects/{id}/github` (`GithubSnapshotResponse`).

## Handler

Module: `backend/src/routes/github.rs`, route `/{id}/github/refresh` on `project_github_router()`.

1. Read `github_repo` from `projects/{id}/index.md`. Missing, empty, or not `owner/name` → **400** plain text `No repository linked`.
2. `auth::resolve_token()`. On error → **400** with that message (includes the `gh auth login` sentence when no `gh` token and no `GH_TOKEN` / `GITHUB_TOKEN`).
3. `GithubClient::new(&token)`.
4. `read_snapshot` for ETags and cached lists.
5. `lists::fetch_login`. Failure is ignored; `login` on a newly built snapshot stays unset. A both-304 update does not clear a stored login.
6. `lists::fetch_lists` with the stored issue and pull ETags.
   - Both lists `*_304`: set `refreshed_at` to `Utc::now()` RFC3339 on the existing snapshot (or an empty one), `write_snapshot`, return it with `error: null`.
   - One list 304: keep that list, its truncation flag, and its echoed ETag from cache; replace the other list from the 200 body.
   - `Err`: if a snapshot exists, **200** with that snapshot and `error` set to `GithubError::user_message()`. If none, **503** with the same message. The previous file is left unchanged.
7. New snapshot sets `repo`, `login` from step 5, `refreshed_at`, ETags, lists, and truncation flags.
8. `snapshot_io::delete_caches_for_closed_items` removes `bodies/issue-{n}.md`, `bodies/pull-{n}.md`, and `comments-*-{n}.json` whose numbers are not in the open set.
9. `write_snapshot`. Write failure → **500**.

`stale` is still derived from `refreshed_at` (older than 15 minutes). A successful refresh clears `error`.

## Test API origin

`lists::set_test_api_base` (`cfg(test)`, thread-local) replaces `https://api.github.com` for `fetch_lists` and `fetch_login`. Route tests bind an Axum listener on port 0 and point the base at `http://127.0.0.1:<port>`.

## Tests

`cargo test -- routes::github::refresh` in `backend/`:

- Mock issues and pulls return one row each → snapshot written, response lists those numbers, closed body/comment files removed.
- Both mock lists return 304 when `If-None-Match` is sent → `refreshed_at` changes, cached titles stay.
- No token and no `gh` on `PATH` → **400** containing `gh auth login`.
- No `github_repo` → **400** `No repository linked`.
