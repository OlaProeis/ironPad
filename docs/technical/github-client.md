# GitHub REST HTTP client

Async reqwest wrapper for GitHub REST calls used by snapshot refresh and later routes. User-facing error text follows PRD §5.6 via `GithubError::user_message()`.

## Location

`backend/src/services/github/client.rs`, exported from `services/github/mod.rs` as `GithubClient` and `GithubError`.

## `GithubClient`

`GithubClient::new(token)` builds a `reqwest::Client` with default headers:

| Header | Value |
|--------|--------|
| `Accept` | `application/vnd.github+json` |
| `Authorization` | `Bearer {token}` |
| `X-GitHub-Api-Version` | `2022-11-28` (constant via `once_cell::sync::Lazy`) |

The token is stored on the struct but never logged or traced.

## Methods

### `get_json<T>`

- GET `url`; optional `If-None-Match` when `etag` is `Some`.
- Success: `(T, Option<String>)` — body and response `ETag` header.
- Status mapping:

| Status | `GithubError` |
|--------|----------------|
| 304 | `NotModified` |
| 401 | `AuthRefused` |
| 403 | `RateLimit { reset_time }` if `X-RateLimit-Remaining: 0` or `Retry-After` present; else `AuthRefused` |
| 404 | `NotFound` |
| 429 | `RateLimit` when rate-limit signal present; else `Network` |
| Other non-success | `Network(msg)` |

Transport or JSON decode failures become `Network`.

### `get_paginated<T>`

- Expects each page body to be a JSON array (`Vec<T>`).
- Follows `Link` header `rel="next"` for up to **5** pages, merging items in order.
- Returns `(items, last_etag, truncated)` — `truncated` is `true` when a sixth page would have been fetched.
- **304 on page 1** → `NotModified` (same as `get_json`).

`last_etag` is taken from the last successfully fetched page.

## `GithubError`

```text
NotModified | AuthNeeded | AuthRefused | NotFound | RateLimit { reset_time } | Network(String)
```

`user_message()` returns the exact PRD §5.6 sentences. `AuthNeeded` is for callers when `resolve_token()` fails (not produced inside the client). `RateLimit` appends local reset time when `X-RateLimit-Reset` was present on the response (Unix epoch → local `chrono` format).

`NotModified` is an internal cache signal; its `user_message()` is not shown in the PRD table.

## Helpers

- `link_rel_next` — parse GitHub-style `Link` header for `rel="next"`.
- `is_rate_limited` — `Retry-After` or `X-RateLimit-Remaining == "0"`.

## Tests

`mockito` dev-dependency; async tests against a local mock server:

```bash
cd backend && cargo test -- services::github::client
```

Covers pagination merge, 304, 401 + message, 403 rate limit, and 404.

## Out of scope

Snapshot read/write and HTTP routes. Issue/PR list fetch and normalization use this client in [`github-lists.md`](./github-lists.md).