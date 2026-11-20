# GitHub list fetch and normalization

Fetches open issues and pull requests from GitHub REST, normalizes API JSON into `GithubIssue` / `GithubPull`, and resolves the authenticated user login. Implements PRD §5.2 list fetch semantics (no snapshot file I/O or routes).

## Location

`backend/src/services/github/lists.rs` — submodule of `services/github/`.

Depends on [`GithubClient`](./github-client.md) (`get_paginated`, `get_json`) and types in `snapshot.rs`.

## `fetch_lists`

```text
pub async fn fetch_lists(
    client: &GithubClient,
    owner: &str,
    repo: &str,
    issue_etag: Option<&str>,
    pull_etag: Option<&str>,
) -> Result<FetchResult, GithubError>
```

### Endpoints

| List | URL |
|------|-----|
| Issues | `GET …/repos/{owner}/{repo}/issues?state=open&sort=updated&direction=desc&per_page=100` |
| Pulls | `GET …/repos/{owner}/{repo}/pulls?state=open&sort=updated&direction=desc&per_page=100` |

Both use `get_paginated::<serde_json::Value>` (up to five pages). Raw values are filtered and mapped in Rust before becoming `Vec<GithubIssue>` / `Vec<GithubPull>`.

### `FetchResult`

| Field | Meaning |
|-------|---------|
| `issues`, `pulls` | Normalized rows (empty when that list returned 304) |
| `issue_etag`, `pull_etag` | ETag from the last page on 200; on 304, the incoming etag is echoed back |
| `issues_truncated`, `pulls_truncated` | From client pagination (sixth page would exist) |
| `issues_304`, `pulls_304` | `true` when that list’s first page was `NotModified` |

A 304 on one list does not block fetching the other; callers merge with the previous snapshot using the `*_304` flags.

### Issue list rules

1. Drop any item whose JSON object contains a `pull_request` key (still listed on the pulls endpoint).
2. Map remaining objects via `normalize_issue`.

### Issue field mapping (`serde_json::Value` → `GithubIssue`)

| API | Snapshot field |
|-----|----------------|
| `number`, `title`, `html_url`, `updated_at` | same |
| `user.login` | `author_login` |
| `labels[].name`, `labels[].color` | `GithubLabel` (`color` is `None` when missing or null) |
| `comments` | `comments_count` (default 0) |
| `milestone.due_on` | `milestone_due`: first 10 characters (`YYYY-MM-DD`) when present |
| `assignees[].login` | `assignee_logins` |

### Pull field mapping

Same common fields as issues (except no assignees on `GithubPull`), plus:

| API | Snapshot field |
|-----|----------------|
| `draft` | `draft` (default false) |
| `requested_reviewers[].login` | `requested_reviewer_logins` — skip reviewer objects that contain a `slug` key (teams) |
| `head.sha` | `head_sha` |

Malformed required fields on a kept row return `GithubError::Network` with a short parse message.

## `fetch_login`

`GET https://api.github.com/user` via `get_json`, then `.login` string. Missing login → `Network`.

## Tests

Inline JSON fixtures in unit tests (no live GitHub):

```bash
cd backend && cargo test -- services::github::lists
```

Covers: dropping `pull_request` shapes from the issues page, full `GithubIssue` mapping, null label color, milestone date prefix, and team reviewer skip on pulls.

## Out of scope

Writing `snapshot.json`, body/comment caches, and `POST`/`GET` project GitHub routes — later tasks compose `fetch_lists`, `fetch_login`, and snapshot I/O.