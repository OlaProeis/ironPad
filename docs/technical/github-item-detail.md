# GitHub issue and pull request detail

`GET /api/projects/{id}/github/issues/{number}` and `GET /api/projects/{id}/github/pulls/{number}` return the open-snapshot title and `html_url`, a Markdown body, and up to 30 issue-thread comments. Pull requests also return `commit_status` when the status call succeeds.

Routes are on `project_github_router()` in `backend/src/routes/github.rs`. Both handlers call `fetch_item_detail`.

## Lookup

The number must appear on the snapshot for that kind (`issues` or `pulls`). Missing snapshot or missing number → **404** plain text `That item is not in the open snapshot.`

The GitHub owner/name comes from `snapshot.repo`. URLs are built with `lists::github_api_url`, so tests can point at a local API via `set_test_api_base`.

## Body and comments

| Step | Behavior |
|------|----------|
| Cached body | `snapshot_io::read_body` (`bodies/issue-{n}.md` or `bodies/pull-{n}.md`). When a file exists, the body endpoint is not called again. |
| Fresh body | Issues: `GET /repos/{owner}/{repo}/issues/{n}`. Pulls: `GET /repos/{owner}/{repo}/pulls/{n}`. Text is stored with `write_body`. |
| Over 1 MiB | `body` length greater than 1,048,576 bytes: do not store, delete any previous body file, respond with `body: null` and `body_too_large: true`. |
| Comments | Always `GET /repos/{owner}/{repo}/issues/{n}/comments?per_page=100` (issues and pulls). Sort `created_at` descending, keep 30, set `more_comments` when the payload had more than 30. Cache with `write_comments` as `CachedComment` JSON. |
| Fetch failure | **200** with `error` set to `GithubError::user_message()`. A cached body is kept. With no cache, `body` is null and `html_url` still comes from the snapshot. A failed comment fetch keeps cached comments when they exist. No token uses the same shape and the `resolve_token` message. |

## Pull request status

After the body and comments, pulls call `GET /repos/{owner}/{repo}/commits/{head_sha}/status` and copy the `state` string into `commit_status`. A failed or empty status call omits the field and does not set `error`. Issue responses omit `commit_status`.

## Response

`GithubItemDetail`: `number`, `title`, `html_url`, `body`, `body_too_large`, `comments` (`author_login`, `created_at`, `body`), `more_comments`, `error`, and optional `commit_status`.

## Tests

`cargo test -- routes::github::detail` with `set_test_cache_root` and a local mock: first issue fetch caches body and comments; the second request does not hit the issue body route; a body over 1 MiB sets `body_too_large`; a pull returns `commit_status: "success"`; an unknown number is 404; HTTP 500 with a cached body returns that body and `error`.
