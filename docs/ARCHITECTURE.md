# Ironpad Architecture

This document describes the technical architecture of Ironpad.

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    Vue 3 SPA                                ││
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐ ││
│  │  │  Views   │  │Components│  │  Stores  │  │ Composables │ ││
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘  └──────┬──────┘ ││
│  │       │             │             │               │         ││
│  │       └─────────────┴──────┬──────┴───────────────┘         ││
│  │                            │                                ││
│  │                    ┌───────▼───────┐                        ││
│  │                    │  API Client   │                        ││
│  └────────────────────┴───────┬───────┴────────────────────────┘│
└───────────────────────────────┼─────────────────────────────────┘
                                │
                    HTTP REST   │   WebSocket
                                │
┌───────────────────────────────┼─────────────────────────────────┐
│                               │                                  │
│  ┌────────────────────────────▼────────────────────────────────┐│
│  │                    Axum Router                               ││
│  │  ┌─────────────────────────────────────────────────────────┐││
│  │  │                    Routes                                │││
│  │  │  /notes /projects /tasks /backlinks /git /prompts /ws   │││
│  │  └───────────────────────────┬─────────────────────────────┘││
│  └──────────────────────────────┼───────────────────────────────┘│
│                                 │                                │
│  ┌──────────────────────────────▼───────────────────────────────┐│
│  │                      Services                                 ││
│  │  ┌──────────┐ ┌───────────┐ ┌─────┐ ┌──────┐ ┌─────┐ ┌─────────┐││
│  │  │Filesystem│ │Frontmatter│ │ Git │ │Search│ │Locks│ │Backlinks│││
│  │  └────┬─────┘ └─────┬─────┘ └──┬──┘ └──┬───┘ └──┬──┘ └────┬───┘││
│  └───────┼──────────────┼───────────┼──────────┼─────────┼──────┘│
│          │              │           │          │         │       │
│          └──────────────┴─────┬─────┴──────────┴─────────┘       │
│                               │                                  │
│                        ┌──────▼──────┐                           │
│                        │  File System │                          │
│                        │    (data/)   │                          │
│                        └─────────────┘                           │
│                                                                  │
│                      Rust Backend                                │
└──────────────────────────────────────────────────────────────────┘
```

## Core Principles

### 1. Files Are the Database

All data is stored as Markdown files with YAML frontmatter:

```markdown
---
id: note-20260205-123456
title: My Note
created: 2026-02-05T12:34:56Z
updated: 2026-02-05T12:34:56Z
---

# My Note

Content goes here...
```

**Benefits:**
- Portable — files can be copied, backed up, synced
- Editable — any text editor works
- Versionable — Git tracks all changes
- Debuggable — human-readable format

### 2. Backend Owns Metadata

The backend automatically manages:
- `id` — Generated from timestamp (YYYYMMDD-HHMMSS)
- `created` — Set once when file is created
- `updated` — Updated on every save

Clients send content; backend handles metadata consistency.

### 3. Local-First

The application works fully offline:
- No cloud dependencies
- No external API calls
- Git push is optional

## Backend Architecture

### Technology Stack

- **Rust** — Memory safety, performance
- **Axum 0.8** — Async web framework
- **Tokio** — Async runtime
- **serde/serde_yaml** — Serialization
- **notify** — File system watching

### Service Layer

```
services/
├── backlinks.rs    # Link extraction, in-memory index, queries
├── filesystem.rs   # File read/write operations
├── frontmatter.rs  # YAML parsing/generation
├── git.rs          # Git CLI wrapper
├── locks.rs        # File locking state
├── markdown.rs     # Markdown utilities
└── search.rs       # ripgrep integration
```

#### Filesystem Service

Handles all file operations with atomic writes:

```rust
// Atomic write pattern
fn write_note(path: &Path, content: &str) -> Result<()> {
    let temp = path.with_extension("tmp");
    fs::write(&temp, content)?;
    fs::rename(temp, path)?;  // Atomic on most filesystems
    Ok(())
}
```

#### Frontmatter Service

Parses and generates YAML frontmatter:

```rust
struct Frontmatter {
    id: String,
    title: Option<String>,
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
    // ... other fields
}
```

#### Git Service

Wraps Git CLI commands:

```rust
impl GitService {
    fn status(&self) -> Result<GitStatus>;
    fn commit(&self, message: &str) -> Result<()>;
    fn push(&self) -> Result<()>;
    fn log(&self, limit: usize) -> Result<Vec<Commit>>;
    fn diff(&self, commit: Option<&str>) -> Result<String>;
}
```

Auto-commit runs every 60 seconds when changes exist.

#### Backlinks Service

Maintains an in-memory link index across all notes. See [`backlinks.md`](./backlinks.md) for full details.

```rust
// Global state (Lazy<Arc<Mutex<...>>>)
LINK_INDEX:  HashMap<target_id, Vec<NoteLink>>  // who links to whom
NOTE_TITLES: HashMap<note_id, title>             // title lookup

// Key operations
fn rebuild_link_index() -> Result<usize>;     // full scan of data/
fn update_note_links(note_id, content);       // called on save (triggers full rebuild)
fn get_backlinks(note_id) -> Vec<Backlink>;   // notes linking TO this note
fn get_forward_links(note_id) -> Vec<ForwardLink>; // notes this note links TO
```

The index is rebuilt:
- **On startup** — initial scan of all markdown files.
- **On note save** — `update_project_note` calls `update_note_links()`.
- **On external file change** — file watcher triggers rebuild for note files.

### WebSocket System

Real-time updates via WebSocket:

```
Client                  Server
   │                       │
   │──── connect ─────────▶│
   │◀─── accepted ─────────│
   │                       │
   │──── lock_file ───────▶│
   │◀─── file_locked ──────│
   │                       │
   │                       │ (file changed on disk)
   │◀─── file_modified ────│
   │                       │
   │──── unlock_file ─────▶│
   │◀─── file_unlocked ────│
```

**Message Types:**
- `lock_file` / `unlock_file` — File locking for concurrent editing
- `file_modified` — Broadcast when files change on disk
- `git_status` — Git status updates

### File Watcher

Uses `notify` crate to watch the data directory:

```rust
// Debounce: 500ms to batch rapid changes
// Filter: Ignores changes from own writes
watcher.watch(data_path, RecursiveMode::Recursive)?;
```

## Frontend Architecture

### Technology Stack

- **Vue 3** — Composition API
- **TypeScript** — Type safety
- **Vite** — Build tooling
- **Pinia** — State management
- **Vue Router** — Navigation
- **Milkdown** — WYSIWYG editor

### Component Hierarchy

```
App.vue
├── Sidebar.vue
│   ├── NoteList.vue
│   ├── ProjectList.vue
│   └── GitStatus.vue
├── TopBar.vue
├── SearchPanel.vue
├── GitPanel.vue
└── <router-view>
    ├── NotesView.vue
    ├── ProjectView.vue
    ├── ProjectNotesView.vue
    │   ├── MilkdownEditor.vue
    │   │   └── MilkdownEditorCore.vue
    │   │       └── LinkAutocomplete.vue
    │   └── BacklinksPanel.vue
    ├── TasksView.vue
    └── DailyView.vue
```

### State Management (Pinia)

Each domain has a dedicated store:

```typescript
// Example: notesStore
export const useNotesStore = defineStore('notes', () => {
  const notes = ref<Note[]>([])
  const currentNote = ref<NoteWithContent | null>(null)
  const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
  
  async function loadNote(id: string) { ... }
  async function saveNote(content: string) { ... }
  
  return { notes, currentNote, saveStatus, loadNote, saveNote }
})
```

### Milkdown Editor Integration

The editor uses a two-component architecture:

```
MilkdownEditor.vue (wrapper — manages key for remounts)
└── MilkdownEditorCore.vue (actual editor — Crepe instance, /link detection)
    └── LinkAutocomplete.vue (note picker dropdown for /link command)
```

**Critical Lifecycle:**

1. `MilkdownProvider` provides Vue context
2. `useEditor` hook creates `Crepe` instance
3. `Crepe.editor` is the ProseMirror editor
4. `editor.action(replaceAll(content))` updates content

**Key Pattern 1:** Content must be set BEFORE the editor key changes:

```javascript
// View component
watch(noteId, async (newId) => {
  const note = await api.getNote(newId)
  
  // CORRECT ORDER:
  editorContent.value = note.content  // 1. Set content
  editorKey.value = newId             // 2. Recreate editor
})
```

**Key Pattern 2:** Always use a raw Crepe reference for programmatic updates:

```javascript
// Crepe uses ES private fields — Vue's proxy breaks getter access.
let crepeRaw: Crepe | null = null

// Store before Vue wraps it:
const { get, loading } = useEditor((root) => {
  const crepe = new Crepe({ root, ... })
  crepeRaw = crepe  // raw reference
  return crepe
})

// Use raw reference (not get()) for action calls:
crepeRaw.editor.action(replaceAll(newContent))
```

If the direct update fails, `MilkdownEditor.vue` supports a `force-remount` event that increments an internal key, destroying and recreating the editor with the current model value.

### Auto-save System

Smart auto-save that prevents unnecessary saves:

```javascript
// Track original content
const lastSavedContent = ref<string | null>(null)

// Only save when content differs
watch(editorContent, (newContent) => {
  if (lastSavedContent.value !== null && 
      newContent !== lastSavedContent.value) {
    scheduleAutoSave()  // 1-second debounce
  }
})
```

## Data Model

### Note

```typescript
interface Note {
  id: string           // e.g., "20260205-123456"
  title?: string
  path: string         // e.g., "notes/20260205-123456.md"
  created: string      // ISO 8601
  updated: string
}

interface NoteWithContent extends Note {
  content: string      // Markdown body
}
```

### Project

```typescript
interface Project {
  id: string           // e.g., "ferrite" (slug)
  title: string
  description?: string
  path: string
  created: string
  updated: string
}
```

### Task

```typescript
interface Comment {
  date: string         // ISO 8601 timestamp
  text: string         // Comment body
}

interface Task {
  id: string           // e.g., "task-20260205-123456"
  title: string
  completed: boolean
  section?: string     // "Active" | "Backlog"
  priority?: string
  due_date?: string
  is_active: boolean
  tags: string[]
  parent_id?: string   // Links subtask to parent
  recurrence?: string  // "daily" | "weekly" | "monthly" | "yearly"
  recurrence_interval?: number
  last_comment?: string // Most recent comment text (list views)
  path: string
  created: string
  updated: string
}

interface TaskWithContent extends Task {
  content: string      // Markdown description
  comments: Comment[]  // Full comment history
}
```

#### Task File Format

```markdown
---
id: ferrite-task-20260216-120000
type: task
title: Implement feature X
completed: false
section: Active
priority: normal
is_active: true
tags:
  - backend
  - api
comments:
  - date: "2026-02-16T10:30:00+00:00"
    text: Started initial research
  - date: "2026-02-16T14:00:00+00:00"
    text: API endpoint done, moving to frontend
created: "2026-02-16T12:00:00+00:00"
updated: "2026-02-16T14:00:00+00:00"
---

# Implement feature X

Detailed description in markdown...
```

Comments are stored as a YAML sequence directly in frontmatter, keeping everything in a single file. The `last_comment` field in list views is derived at read time from the last entry in the sequence.

## API Design

### REST Conventions

- `GET /api/resource` — List all
- `POST /api/resource` — Create new
- `GET /api/resource/:id` — Get one
- `PUT /api/resource/:id` — Update
- `DELETE /api/resource/:id` — Delete (usually archives)

### Error Handling

```json
{
  "error": "Note not found",
  "code": "NOT_FOUND"
}
```

HTTP status codes:
- `200` — Success
- `201` — Created
- `400` — Bad request
- `404` — Not found
- `500` — Server error

## Security Considerations

### Current State

Ironpad is designed for **local, single-user** operation:

- No authentication (local access assumed)
- No HTTPS (localhost only)
- No input sanitization for XSS (trusted user)

### Production Deployment

For multi-user or remote deployment:

1. Add authentication (JWT, session-based)
2. Enable HTTPS
3. Sanitize markdown output
4. Rate limit API endpoints
5. Validate file paths to prevent directory traversal

## Performance Considerations

### Backend

- **Atomic writes** — Prevent corruption on crash
- **File caching** — Read once, cache in memory (not yet implemented)
- **Ripgrep search** — Fast full-text search

### Frontend

- **Virtual scrolling** — For large note lists (not yet needed)
- **Debounced saves** — 1-second delay batches rapid edits
- **Lazy loading** — Routes loaded on demand

## Future Considerations

### Scalability

Current design handles ~5000 files comfortably. For larger datasets:

- Add Tantivy full-text search index
- Implement pagination for note lists
- Add lazy loading for project trees

### Features

See `ai-context.md` for planned features:

- Tag extraction and filtering
- Graph view of note connections (builds on the backlinks index)
- Export (PDF/HTML)
- Custom themes
