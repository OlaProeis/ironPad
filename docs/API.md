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
    "content": "## Requirements\n\n- Item 1\n- Item 2",
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
