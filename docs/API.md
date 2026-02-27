# Ironpad API Reference

Base URL: `http://localhost:3000`

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
    "updated": "2026-02-05T12:00:00Z"
  }
]
```

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
  "updated": "2026-02-05T12:34:56Z"
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
  "updated": "2026-02-05T12:00:00Z"
}
```

### Get Project Content

```http
GET /api/projects/:id/content
```

**Response:**
```json
{
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
  "due_date": "2026-02-15"
}
```

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

### Fetch from Remote

```http
POST /api/git/fetch
```

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
