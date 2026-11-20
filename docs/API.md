# Ironpad API Reference

Base URL: `http://localhost:37891` (default range **37891–37990**; set `IRONPAD_PORT` to pin a port)

## Notes

### List Notes

```http
GET /api/notes
```

**Response:**
```json
[
  {
    "id": "20260205-123456",
    "title": "My Note",
    "path": "notes/20260205-123456.md",
    "created": "2026-02-05T12:34:56Z",
    "updated": "2026-02-05T12:34:56Z"
  }
]
```

### Create Note

```http
POST /api/notes
Content-Type: application/json

{
  "title": "Optional Title",
  "content": "# My Note\n\nContent here"
}
```

**Response:** `201 Created`
```json
{
  "id": "20260205-123456",
  "title": "Optional Title",
  "path": "notes/20260205-123456.md",
  "content": "# My Note\n\nContent here",
  "created": "2026-02-05T12:34:56Z",
  "updated": "2026-02-05T12:34:56Z"
}
```

### Get Note

```http
GET /api/notes/:id
```

**Response:**
```json
{
  "id": "20260205-123456",
  "title": "My Note",
  "path": "notes/20260205-123456.md",
  "content": "# My Note\n\nFull content...",
  "created": "2026-02-05T12:34:56Z",
  "updated": "2026-02-05T12:34:56Z"
}
```

### Update Note

```http
PUT /api/notes/:id
Content-Type: application/json

{
  "content": "# Updated Content\n\nNew content here"
}
```

**Response:**
```json
{
  "id": "20260205-123456",
  "title": "Updated Content",
  "path": "notes/20260205-123456.md",
  "content": "# Updated Content\n\nNew content here",
  "created": "2026-02-05T12:34:56Z",
  "updated": "2026-02-05T12:35:00Z"
}
```

### Delete (Archive) Note

```http
DELETE /api/notes/:id
```

**Response:** `200 OK`

Note: The note is moved to `archive/`, not permanently deleted.

---

## Projects

### List Projects

```http
GET /api/projects
```

**Response:**
```json
[
  {
    "id": "ferrite",
    "title": "Ferrite",
    "description": "A Rust project",
    "path": "projects/ferrite",
    "created": "2026-02-04T10:00:00Z",
    "updated": "2026-02-05T12:00:00Z",
    "github_repo": "acme/ferrite"
  }
]
```

`github_repo` is optional (`null` when the project `index.md` frontmatter has no `github_repo` key). See [`technical/project-github-repo.md`](./technical/project-github-repo.md).

### Create Project

```http
POST /api/projects
Content-Type: application/json

{
  "title": "New Project",
  "description": "Project description"
}
```

**Response:** `201 Created`
```json
{
  "id": "new-project",
  "title": "New Project",
  "description": "Project description",
  "path": "projects/new-project",
  "created": "2026-02-05T12:34:56Z",
  "updated": "2026-02-05T12:34:56Z",
  "github_repo": null
}
```

### Get Project

```http
GET /api/projects/:id
```

**Response:**
```json
{
  "id": "ferrite",
  "title": "Ferrite",
  "description": "A Rust project",
  "path": "projects/ferrite",
  "created": "2026-02-04T10:00:00Z",
  "updated": "2026-02-05T12:00:00Z",
  "github_repo": "acme/ferrite"
}
```

### Get Project Content

```http
GET /api/projects/:id/content
```

**Response:**
```json
{
  "id": "ferrite",
  "name": "Ferrite",
  "path": "projects/ferrite",
  "created": "2026-02-04T10:00:00Z",
  "github_repo": "acme/ferrite",
  "content": "# Ferrite\n\nProject overview content..."
}
```

### Update Project Content

```http
PUT /api/projects/:id/content
Content-Type: application/json

{
  "content": "# Updated Overview\n\nNew content..."
}
```

---

## GitHub

Read-only github.com issues and pull requests per project. User-facing overview: [`github-link.md`](./github-link.md). Responses never include a token. **Added in 0.4.0.**

| Method and path | Behavior | Added in 0.4.0 |
|-----------------|----------|----------------|
| `GET /api/projects/{id}/github` | Saved link and snapshot. No GitHub call. | yes |
| `PUT /api/projects/{id}/github` | Body `{ "repo": "owner/name" }` saves, or `{ "repo": null }` removes. Save does not call GitHub; the client then calls refresh. Remove deletes the snapshot directory. | yes |
| `POST /api/projects/{id}/github/refresh` | Fetches open lists and login from GitHub; merges snapshot. Returns the same shape as GET. | yes |
| `GET /api/projects/{id}/github/issues/{number}` | Cached or freshly fetched issue body, up to 30 comments, and `html_url`. | yes |
| `GET /api/projects/{id}/github/pulls/{number}` | Same for a pull request, plus combined commit status when available. | yes |
| `POST /api/projects/{id}/github/issues/{number}/track` | Creates or returns existing Active task for that issue URL. | yes |
| `POST /api/projects/{id}/github/pulls/{number}/track` | Same for a pull request. | yes |
| `GET /api/github/summary` | Dashboard assigned/review counts from snapshots only. No GitHub call. | yes |

An item number absent from the open snapshot returns **404** with `That item is not in the open snapshot.`

### Get Project GitHub Snapshot

```http
GET /api/projects/:id/github
```

Reads the per-project snapshot from disk. Does not call GitHub.

**Response:** `200 OK`

```json
{
  "repo": "acme/ferrite",
  "login": "octocat",
  "refreshed_at": "2026-02-05T12:00:00Z",
  "stale": false,
  "issues": [],
  "pulls": [],
  "issues_truncated": false,
  "pulls_truncated": false,
  "error": null
}
```

`stale` is `true` when there is no snapshot, `refreshed_at` is missing or invalid, or the snapshot is older than 15 minutes. See [`technical/project-github-routes.md`](./technical/project-github-routes.md).

### Save or Remove Project GitHub Link

```http
PUT /api/projects/:id/github
Content-Type: application/json

{ "repo": "acme/ferrite" }
```

**Response:** `200 OK`

```json
{ "repo": "acme/ferrite" }
```

Writes `github_repo` to the project `index.md` frontmatter. Does not refresh the snapshot.

**Remove link:**

```json
{ "repo": null }
```

**Response:** `204 No Content`

Deletes the project’s GitHub snapshot cache directory and removes `github_repo` from frontmatter.

**Validation error:** `400 Bad Request` with body `Use owner/name, with no URL and no .git suffix.`

**Project not found:** `404 Not Found`

### Refresh GitHub Snapshot

```http
POST /api/projects/:id/github/refresh
```

Calls GitHub for open issues, open pull requests, and the authenticated login; updates the on-disk snapshot (including ETag-based 304 handling). **Response:** same JSON shape as GET. See [`technical/github-refresh.md`](./technical/github-refresh.md).

### Get Issue Detail

```http
GET /api/projects/:id/github/issues/:number
```

Returns rendered body metadata, up to 30 newest comments, and `html_url`. Body fetches are capped at 1 MiB. See [`technical/github-item-detail.md`](./technical/github-item-detail.md).

### Get Pull Request Detail

```http
GET /api/projects/:id/github/pulls/:number
```

Same as issue detail, plus `commit_status` when the status API succeeds.

### Track Issue as Task

```http
POST /api/projects/:id/github/issues/:number/track
```

**Response:** `200 OK` — `{ "task_id": "...", "created": true }` or existing task with `created: false`. See [`technical/github-track.md`](./technical/github-track.md).

### Track Pull Request as Task

```http
POST /api/projects/:id/github/pulls/:number/track
```

Same response shape as issue track.

### GitHub Dashboard Summary

```http
GET /api/github/summary
```

Reads every project snapshot from disk. Does not call GitHub.

**Response:** `200 OK` — JSON array of objects with `project_id`, `repo`, `assigned_issues`, `review_pulls`, and `refreshed_at`. Only projects with a snapshot that has a non-empty `login` and `repo` are included. See [`technical/github-summary.md`](./technical/github-summary.md).

---

## Project Notes

### List Project Notes

```http
GET /api/projects/:id/notes
```

**Response:**
```json
[
  {
    "id": "20260205-123456",
    "title": "Project Note",
    "path": "projects/ferrite/notes/20260205-123456.md",
    "created": "2026-02-05T12:34:56Z",
    "updated": "2026-02-05T12:34:56Z"
  }
]
```

### Create Project Note

```http
POST /api/projects/:id/notes
Content-Type: application/json

{
  "title": "New Note",
  "content": "Note content..."
}
```

### Get Project Note

```http
GET /api/projects/:id/notes/:noteId
```

### Update Project Note

```http
PUT /api/projects/:id/notes/:noteId
Content-Type: application/json

{
  "content": "Updated content..."
}
```

### Delete Project Note

```http
DELETE /api/projects/:id/notes/:noteId
```

### Get Note Titles (for link autocomplete)

```http
GET /api/projects/:id/notes-titles
```

Returns all notes in the project with their frontmatter IDs and titles. Used by the `/link` slash command dropdown.

**Response:**

```json
{
  "notes": [
    { "id": "ferrite-index", "title": "Ferrite", "path": "projects/ferrite/index.md" },
    { "id": "ferrite-20260227-153000", "title": "Meeting Notes", "path": "projects/ferrite/notes/20260227-153000.md" }
  ]
}
```

### Search Project Notes

```http
GET /api/projects/:id/notes-search?q=meeting&limit=10
```

Filter notes by partial title or ID match within a project.

**Response:**

```json
{
  "query": "meeting",
  "results": [
    { "id": "ferrite-20260227-153000", "title": "Meeting Notes", "path": "projects/ferrite/notes/20260227-153000.md" }
  ]
}
```

---

## Backlinks

See [`backlinks.md`](./backlinks.md) for full feature documentation.

### Get Links for a Note

Returns both backlinks (incoming) and forward links (outgoing).

```http
GET /api/backlinks/notes/:noteId/links
```

**Response:**

```json
{
  "note_id": "ferrite-20260227-153000",
  "backlinks": [
    {
      "source_id": "ferrite-20260226-100000",
      "source_title": "Project Overview",
      "source_path": "projects/ferrite/notes/20260226-100000.md",
      "context": "See [Meeting Notes](ferrite-20260227-153000) for details",
      "line_number": 5
    }
  ],
  "forward_links": [
    {
      "target_id": "ferrite-20260225-090000",
      "target_title": "Architecture Notes",
      "context": "Based on the [Architecture Notes](ferrite-20260225-090000)",
      "line_number": 12
    }
  ]
}
```

### Get Backlinks Only

```http
GET /api/backlinks/notes/:noteId/backlinks
```

**Response:**

```json
{
  "note_id": "ferrite-20260227-153000",
  "backlinks": [...],
  "count": 1
}
```

### Get Forward Links Only

```http
GET /api/backlinks/notes/:noteId/forward-links
```

**Response:**

```json
{
  "note_id": "ferrite-20260227-153000",
  "forward_links": [...],
  "count": 2
}
```

### Rebuild Link Index

Force a full rescan of all note files and rebuild the in-memory link index.

```http
POST /api/backlinks/links/rebuild
```

**Response:**

```json
{
  "success": true,
  "indexed_notes": 42,
  "message": "Indexed 42 notes"
}
```

### Get Link Statistics

```http
GET /api/backlinks/links/stats
```

**Response:**

```json
{
  "total_links": 15,
  "unique_targets": 8
}
```

---

## Project Tasks

### List Project Tasks

```http
GET /api/projects/:id/tasks
```

**Response:**
```json
[
  {
    "id": "task-20260205-123456",
    "title": "Implement feature X",
    "completed": false,
    "section": "Active",
    "priority": "high",
    "due_date": "2026-02-10",
    "is_active": true,
    "tags": ["backend", "api"],
    "parent_id": null,
    "recurrence": null,
    "recurrence_interval": null,
    "project_id": "ferrite",
    "last_comment": "API endpoint done, moving to frontend",
    "path": "projects/ferrite/tasks/task-20260205-123456.md",
    "created": "2026-02-05T12:34:56Z",
    "updated": "2026-02-05T12:34:56Z"
  }
]
```

### Create Task

```http
POST /api/projects/:id/tasks
Content-Type: application/json

{
  "title": "New Task",
  "content": "Task description..."
}
```

### Get Task

```http
GET /api/projects/:id/tasks/:taskId
```

### Update Task Content

```http
PUT /api/projects/:id/tasks/:taskId
Content-Type: application/json

{
  "content": "Updated task description..."
}
```

### Update Task Metadata

```http
PUT /api/projects/:id/tasks/:taskId/meta
Content-Type: application/json

{
  "title": "New Title",
  "is_active": false,
  "section": "Backlog",
  "priority": "low",
  "due_date": "2026-02-15",
  "completed": false
}
```

**`completed` (optional):** `false` clears completion in frontmatter (e.g. when moving out of Completed). `true` is ignored — use the toggle endpoint to mark complete and to spawn the next recurring instance.

### Toggle Task Completion

```http
PUT /api/projects/:id/tasks/:taskId/toggle
```

**Response:**
```json
{
  "completed": true
}
```

### Delete Task

```http
DELETE /api/projects/:id/tasks/:taskId
```

### Add Comment

```http
POST /api/projects/:id/tasks/:taskId/comments
Content-Type: application/json

{
  "text": "Started work on this — API integration is in progress."
}
```

**Response:** `201 Created`
```json
{
  "id": "task-20260216-120000",
  "title": "Implement feature X",
  "completed": false,
  "section": "Active",
  "is_active": true,
  "comments": [
    {
      "date": "2026-02-16T10:30:00+00:00",
      "text": "Created initial spec"
    },
    {
      "date": "2026-02-16T12:00:00+00:00",
      "text": "Started work on this — API integration is in progress."
    }
  ],
  "content": "## Requirements\n\n- Item 1\n- Item 2",
  "...": "other task fields"
}
```

Comments are stored as a YAML sequence in the task's frontmatter. The response returns the full `TaskWithContent` object with all comments.

### Delete Comment

```http
DELETE /api/projects/:id/tasks/:taskId/comments/:commentIndex
```

Removes the comment at the given zero-based index.

**Response:**
```json
{
  "id": "task-20260216-120000",
  "comments": [],
  "...": "full TaskWithContent"
}
```

### Comment in List Views

When listing tasks (`GET /api/projects/:id/tasks` or `GET /api/tasks`), each task includes a `last_comment` field with the text of the most recent comment (or `null` if no comments exist). This enables showing a quick status summary without loading the full task.

```json
{
  "id": "task-20260216-120000",
  "title": "Implement feature X",
  "last_comment": "Started work on this — API integration is in progress.",
  "...": "other task fields"
}
```

---

## All Tasks

### List All Tasks (across projects)

```http
GET /api/tasks
```

Returns tasks from all projects, useful for global task views.

---

## Daily Notes

### List Daily Notes

```http
GET /api/daily
```

**Response:**
```json
[
  {
    "date": "2026-02-05",
    "path": "daily/2026-02-05.md",
    "created": "2026-02-05T08:00:00Z",
    "updated": "2026-02-05T12:00:00Z"
  }
]
```

### Get Today's Note

```http
GET /api/daily/today
```

Creates the daily note if it doesn't exist.

**Response:**
```json
{
  "date": "2026-02-05",
  "content": "# 2026-02-05\n\n## Todo\n\n- [ ] Task 1",
  "path": "daily/2026-02-05.md",
  "created": "2026-02-05T08:00:00Z",
  "updated": "2026-02-05T12:00:00Z"
}
```

### Get/Create Daily Note by Date

```http
GET /api/daily/:date
POST /api/daily/:date
```

Date format: `YYYY-MM-DD`

---

## Assets

### Upload Asset

```http
POST /api/assets/upload
Content-Type: multipart/form-data

project: ferrite
file: (binary data)
```

**Response:**
```json
{
  "url": "/api/assets/ferrite/image-20260205-123456.png",
  "filename": "image-20260205-123456.png"
}
```

### Get Asset

```http
GET /api/assets/:project/:filename
```

Returns the binary file with appropriate Content-Type header.

---

## Search

### Search Content

```http
GET /api/search?q=search+term
```

**Response:**
```json
{
  "results": [
    {
      "path": "notes/20260205-123456.md",
      "title": "My Note",
      "matches": [
        {
          "line": 5,
          "text": "This is a **search term** example"
        }
      ]
    }
  ]
}
```

---

## Prompts (Library)

Ironpad’s **Library** uses the existing prompt routes and storage (`data/prompts/`, `data/projects/{id}/prompts/`). A **prompt** is content Ironpad owns. A **link** (`kind: link`) is a pointer at a file on disk; the API reads the source at request time and does not copy it into `data/`.

Pointer fields on create, update, and list/get payloads:

| Field | Prompt | Link |
|-------|--------|------|
| `kind` | omitted or `"prompt"` | `"link"` |
| `source` | — | Project: relative to the client folder root. Global: absolute path. |
| `template` | — | Optional; default `false`. When `true`, `{{variables}}` apply like a prompt (in memory only). |

On **create** and **update**, `content` is **ignored** for links. **`kind` cannot change** on update (400). Plain prompts behave as in 0.3.0.

### List prompts

```http
GET /api/prompts?scope=global&folder=&tag=&q=
```

List rows include `kind` and `source` from frontmatter only (external files are not opened).

### Create prompt or link

```http
POST /api/prompts
Content-Type: application/json
```

**Body (prompt):**

```json
{
  "scope": "global",
  "title": "My prompt",
  "folder": "general",
  "tags": ["review"],
  "description": "Optional",
  "content": "Hello {{name}}"
}
```

**Body (link):**

```json
{
  "scope": "project",
  "project_id": "ferrite",
  "title": "AI context",
  "folder": "repo",
  "kind": "link",
  "source": "ai-context.md",
  "template": false
}
```

**Response:** `201 Created` — `Prompt` JSON (`kind`, `source`, `template` on links).

Validation failures (missing `source`, bad path shape, kind change, etc.) return **400** plain text.

### Get prompt or link content

```http
GET /api/prompts/:id
GET /api/prompts/:id?root=C:\path\to\repo
```

| Query | Role |
|-------|------|
| `root` | Optional absolute directory for **project** links. Stored in browser `localStorage`, not in `data/`. Empty/whitespace is ignored. |

**Response:** `200` JSON — prompt fields plus link metadata:

| Field | Meaning |
|-------|---------|
| `read_only` | `true` for links (source is not writable via Save). |
| `source_missing` | `true` when the resolved source file does not exist. |
| `resolved_path` | Absolute path when known (including unresolved/missing cases). |
| `supports_rendered` | On links: whether `.md`/`.markdown` may use rendered mode. |
| `root_required` | Present and `true` when a project link needs `root` and none was sent. |
| `source_error` | Present when size, suffix, encoding, or path rules block the read (user message). |

For an existing link pointer, Ironpad returns **200** even when the source is missing or invalid so the client can edit or relink. Unknown id is **404**.

Prompts return stored `content`, `read_only: false`, and `source_missing: false`. Link `content` is the current source file text when read succeeds.

### Update prompt or link

```http
PUT /api/prompts/:id
Content-Type: application/json
```

Same optional fields as create. Links may update metadata and `source`/`template`; body `content` does not change the linked file on disk.

### Delete prompt

```http
DELETE /api/prompts/:id
```

Deletes the pointer file only; linked source files on disk are untouched.

### Folders and semantic search

```http
GET /api/prompts/folders?scope=global&project_id=
GET /api/prompts/search/semantic?q=term&scope=global&project_id=
```

Unchanged from 0.3.0. Semantic search for links uses pointer metadata (`source`, title, tags); it does not read external file bodies.

---

## Library (browse and open)

Mounted at `/api/library`. Used by **Add link** and **Open externally**. See also [`technical/library-browse.md`](./technical/library-browse.md) and [`technical/library-open.md`](./technical/library-open.md).

### Browse directory

```http
GET /api/library/browse?path=C:\path\to\repo
```

| Query | Required | Rule |
|-------|----------|------|
| `path` | yes | Absolute path to an existing **directory** (non-recursive listing). |

**Response (200):**

```json
{
  "path": "C:\\path\\to\\repo",
  "parent": "C:\\path\\to",
  "entries": [
    {
      "name": "docs",
      "path": "C:\\path\\to\\repo\\docs",
      "is_dir": true,
      "size": 0,
      "linkable": false
    },
    {
      "name": "ai-context.md",
      "path": "C:\\path\\to\\repo\\ai-context.md",
      "is_dir": false,
      "size": 42,
      "linkable": true
    }
  ]
}
```

- Entries: directories first, then files, case-insensitive by `name`. `.git` is skipped.
- `linkable` is `true` only for regular files within the 1 MiB cap and allowed suffixes (see library PRD).

| Condition | HTTP | Body |
|-----------|------|------|
| Relative or non-absolute `path` | 400 | Plain text |
| Path exists but is not a directory | 400 | Plain text |
| Directory not found | 404 | Plain text |
| Unexpected I/O | 500 | Plain text |

### Open linked file externally

```http
POST /api/library/open
Content-Type: application/json
```

**Body:**

```json
{
  "id": "link-pointer-id",
  "root": "C:\\path\\to\\repo"
}
```

| Field | Required | Rule |
|-------|----------|------|
| `id` | yes | Link prompt id (same resolution as `GET /api/prompts/:id`). |
| `root` | no | Project folder root; same semantics as the GET `root` query. |

The client does not send a file path; the server resolves via the pointer.

**Response (200):**

```json
{
  "ok": true,
  "resolved_path": "C:\\path\\to\\repo\\ai-context.md"
}
```

Opens the file with the OS default application (`cmd /C start` on Windows, `open` on macOS, `xdg-open` elsewhere).

| Condition | HTTP |
|-----------|------|
| Missing source file | 404 |
| Unknown prompt id | 404 |
| Not a link, bad source, outside root, disallowed suffix, root required, etc. | 400 |
| OS failed to launch | 500 |

---

## Git Operations

### Get Status

```http
GET /api/git/status
```

**Response:**
```json
{
  "branch": "main",
  "ahead": 2,
  "behind": 0,
  "staged": [],
  "modified": ["notes/20260205-123456.md"],
  "untracked": [],
  "has_conflicts": false
}
```

### Commit Changes

```http
POST /api/git/commit
Content-Type: application/json

{
  "message": "Update notes"
}
```

### Push to Remote

```http
POST /api/git/push
```

On success, sets `origin/<branch>` as upstream when the branch had none. Details: [`technical/git-push-upstream.md`](./technical/git-push-upstream.md).

Failure JSON uses `success: false` and a short `message`. Wording is in [`technical/git-user-facing-errors.md`](./technical/git-user-facing-errors.md).

### Fetch from Remote

```http
POST /api/git/fetch
```

Failure JSON uses the same friendly `message` strings as push.

### Pull (fast-forward only)

```http
POST /api/git/pull
```

No request body. Fetches `origin`, then fast-forwards the current branch to its upstream. Does not merge, rebase, or force-update. Details: [`technical/git-pull.md`](./technical/git-pull.md).

**Response (200):**

```json
{
  "success": true,
  "message": "Already up to date.",
  "commits": 0
}
```

After a fast-forward, `message` is `"Pulled 1 commit."` or `"Pulled N commits."` (N > 1) and `commits` matches N.

**Response (400):**

```json
{
  "success": false,
  "message": "The remote and this copy have diverged. Pull did not change your files.",
  "commits": 0
}
```

| Situation | HTTP | `message` (representative) |
|-----------|------|----------------------------|
| No `origin` | 400 | No remote yet. Add a private repository URL. |
| Branch has no upstream (fetch not run) | 400 | No upstream yet. The first successful push sets it. |
| Histories not a fast-forward | 400 | The remote and this copy have diverged. Pull did not change your files. |
| Checkout would overwrite local edits | 400 | Uncommitted changes would be overwritten. Wait for auto-commit, or commit, then pull again. |
| Fetch / auth / network (after upstream exists) | 400 | Friendly sentence from [`map_git_failure`](./technical/git-user-facing-errors.md) (`Pull`); unclassified libgit2 text may appear verbatim. |

The no-remote and no-upstream sentences are returned directly, not via the classifier. Wording for push, fetch, and remote setup is in [`technical/git-user-facing-errors.md`](./technical/git-user-facing-errors.md).

### Get Commit Log

```http
GET /api/git/log?limit=20
```

**Response:**
```json
[
  {
    "id": "abc123...",
    "message": "Update notes",
    "author": "User Name",
    "date": "2026-02-05T12:34:56Z",
    "files_changed": 3
  }
]
```

### Get Working Directory Diff

```http
GET /api/git/diff
```

**Response:**
```json
{
  "diff": "diff --git a/notes/... "
}
```

### Get Commit Diff

```http
GET /api/git/diff/:commitId
```

### Get Remote Info

```http
GET /api/git/remote
```

**Response:**
```json
{
  "name": "origin",
  "url": "git@github.com:user/repo.git",
  "ahead": 2,
  "behind": 0
}
```

### Add or Replace Remote Origin

```http
POST /api/git/remote
Content-Type: application/json

{
  "url": "git@github.com:user/repo.git"
}
```

**Request body**

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| `url` | string | yes | Trimmed before validation |

**URL validation** (`validate_remote_url` — errors are returned in `message` as-is, not through `map_git_failure`):

- **Accept:** `https://…`, `http://…`, `ssh://…`, or SCP-style `git@host:path`
- **Reject:** empty/whitespace; `file://` or filesystem paths; URLs without an allowed scheme; `https://user@host/…` or `https://user:pass@host/…` (use SSH or a Git credential helper)
- Representative validation `message` values: `Remote URL cannot be empty`, `Filesystem paths are not allowed. Use https://, ssh://, or git@host:path.`, `Invalid remote URL. Use https://, ssh://, or git@host:path.`, `URLs must not embed credentials. Use SSH or a Git credential helper instead.`

On success the service adds `origin` or calls `remote_set_url`, then runs one fetch. Details: [`technical/git-remote-origin.md`](./technical/git-remote-origin.md).

**Response (200):**

```json
{
  "success": true,
  "message": "Remote origin was added.",
  "url": "git@github.com:user/repo.git",
  "replaced": false,
  "fetch_error": null
}
```

| Field | Type | Notes |
|-------|------|-------|
| `success` | boolean | always `true` on HTTP 200 |
| `message` | string | `Remote origin was added.` or `Remote origin URL was replaced.` |
| `url` | string | Saved URL (trimmed) |
| `replaced` | boolean | `true` when `origin` already existed |
| `fetch_error` | string \| null | Post-save fetch failure, mapped via `map_git_failure` (`Fetch`); URL is not rolled back |

**Response (400):**

```json
{
  "success": false,
  "message": "Invalid remote URL. Use https://, ssh://, or git@host:path.",
  "url": "",
  "replaced": false,
  "fetch_error": null
}
```

Repo open / remote add failures use HTTP 400 with the same JSON shape; `message` comes from `map_git_failure` (`RemoteSet`) except for validation strings above. See [`technical/git-user-facing-errors.md`](./technical/git-user-facing-errors.md).

### Check for Conflicts

```http
GET /api/git/conflicts
```

**Response:**
```json
{
  "has_conflicts": false,
  "files": []
}
```

---

## WebSocket

### Connect

```
WS /ws
```

### Messages (Client → Server)

**Lock File:**
```json
{
  "type": "lock_file",
  "path": "notes/20260205-123456.md",
  "lock_type": "editor"
}
```

**Unlock File:**
```json
{
  "type": "unlock_file",
  "path": "notes/20260205-123456.md"
}
```

### Messages (Server → Client)

**File Locked:**
```json
{
  "type": "file_locked",
  "path": "notes/20260205-123456.md",
  "client_id": "client-123"
}
```

**File Unlocked:**
```json
{
  "type": "file_unlocked",
  "path": "notes/20260205-123456.md"
}
```

**File Modified (broadcast):**
```json
{
  "type": "file_modified",
  "path": "notes/20260205-123456.md"
}
```

**Git Status Update:**
```json
{
  "type": "git_status",
  "status": { ... }
}
```

---

## Error Responses

All endpoints return errors in this format:

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE"
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `NOT_FOUND` | 404 | Resource doesn't exist |
| `BAD_REQUEST` | 400 | Invalid request data |
| `CONFLICT` | 409 | Resource conflict (e.g., Git) |
| `INTERNAL_ERROR` | 500 | Server error |
