# AI Context — Ironpad

## Rules for This File (READ FIRST)

**What belongs in ai-context.md:**
- Architecture and system design
- Critical patterns and technical decisions
- Active constraints and known issues
- Key workflows and conventions
- Active work items (In Progress)

**What does NOT belong here:**
- History of completed work → use `CHANGELOG.md`
- Session handover notes → use `HANDOVER.md`
- Feature wish lists → use `ROADMAP.md`
- Full API documentation → use `docs/API.md`

**When to update:**
- Adding new critical patterns or constraints
- Changing architecture decisions
- Adding/removing in-progress items
- Documenting new known issues

---

## Project Overview

**Ironpad** is a local-first, file-based personal project & knowledge management system.

| Component | Technology |
|-----------|------------|
| Backend | Rust (Axum 0.8, Tokio) |
| Frontend | Vue 3 (Vite, TypeScript) |
| Editor | Milkdown (ProseMirror-based WYSIWYG) |
| State | Pinia |
| Data | Plain Markdown + YAML frontmatter |
| Versioning | Local Git repository |
| UI | System browser (no Electron) |

---

## Core Principles

1. **Files are the database** — filesystem is source of truth
2. **Local-first** — works fully offline
3. **External editing supported** — VS Code, Obsidian, Vim all work
4. **Backend owns metadata** — `id`, `created`, `updated` are auto-managed
5. **Low ceremony** — minimal config, no manual metadata editing

---

## Architecture

### Directory Structure

```
ironpad/
├── backend/           # Rust Axum server
│   └── src/
│       ├── main.rs           # Server bootstrap, WebSocket, routes
│       ├── routes/           # API endpoints
│       ├── services/         # Business logic (filesystem, git, search, tasks)
│       ├── models/           # Data structures
│       └── watcher.rs        # File system watching
├── frontend/          # Vue 3 SPA
│   └── src/
│       ├── App.vue           # Root component
│       ├── router/           # Vue Router config
│       ├── stores/           # Pinia stores
│       ├── views/            # Route views
│       ├── components/       # Reusable components
│       ├── api/              # API client
│       └── types/            # TypeScript types
└── data/              # User data (separate git repo)
    ├── prompts/       # Global prompts
    ├── projects/      # Project folders
    │   └── {project}/
    │       ├── index.md      # Project overview
    │       ├── notes/        # Project notes
    │       ├── prompts/      # Project prompts
    │       └── tasks/        # Task files (task-YYYYMMDD-HHMMSS.md)
    ├── daily/         # Daily notes (YYYY-MM-DD.md)
    ├── archive/       # Archived items
    └── inbox.md       # Quick capture
```

### Key Technical Decisions

| Decision | Choice |
|----------|--------|
| Data path | `../data` relative to backend |
| Port | Dynamic 3000-3010 |
| Auto-save | 1s debounce in frontend |
| Git commits | 60s batch + manual button |
| File watcher | notify crate, 500ms debounce |
| Search | ripgrep CLI, fallback to manual |
| Frontmatter | serde_yaml, auto-generated IDs |
| Editor | Milkdown (WYSIWYG ProseMirror-based) |
| State management | Pinia stores |
| Routing | Vue Router (history mode) |
| File locking | WebSocket-based, per-client locks |
| Project note ID | `{slug}-index` format |
| Task storage | Individual .md files in `tasks/` folder |
| List sorting | By created date (stable) |
| Backend mode | API-only (dev); frontend-serving + system tray (production) |
| Theme | Dark by default, toggle to light, persists to localStorage |

---

## Critical Patterns

### Milkdown Editor Lifecycle

The editor requires careful handling when switching content:

```javascript
// CORRECT order:
editorContent.value = loadedContent  // 1. Set content first
editorKey.value = noteId             // 2. Recreate editor

// WRONG order (causes stale content):
editorKey.value = noteId             // Editor recreates with empty/stale content
editorContent.value = loadedContent  // Too late!
```

**Why:** The editor uses `defaultValue` from props at creation time. State in `MilkdownEditorCore` must be refs, not module-level variables.

### Crepe Raw Reference (Vue Proxy Workaround)

Milkdown's `Crepe` class uses ES private fields (`#editor`). Vue's reactive proxy breaks getter access to these fields because `this` becomes the proxy. Always use the raw reference stored at creation time:

```javascript
let crepeRaw: Crepe | null = null

// In useEditor factory - store BEFORE Vue can wrap it:
crepeRaw = crepe

// CORRECT: use raw reference for programmatic updates
crepeRaw.editor.action(replaceAll(newContent))

// WRONG: get() returns a proxy-wrapped instance
const crepe = get()
crepe.editor.action(...)  // Fails - proxy can't access #editor
```

If the raw reference also fails, `MilkdownEditor.vue` supports a `force-remount` event that increments an internal key, destroying and recreating the editor with updated content.

### Note ID vs Filename

Project notes have two identifiers:
- **Filename**: `20260227-153000` (used in routes and file paths)
- **Frontmatter ID**: `ironpad-20260227-153000` (used in link targets and backlink index)

When querying the backlinks API, always pass the **frontmatter ID** (`selectedNote.id`), not the filename (`currentNoteId`).

### Atomic File Writes

```rust
// Pattern used in filesystem service
let temp = path.with_extension("tmp");
fs::write(&temp, content)?;
fs::rename(temp, path)?;  // Atomic on most filesystems
```

---

## API Summary

Key endpoints (see `docs/API.md` for complete reference):

```
Projects:   GET/POST    /api/projects
           GET/PUT     /api/projects/:id
           GET/PUT     /api/projects/:id/content

Tasks:      GET/POST    /api/projects/:id/tasks
           GET/PUT/DEL /api/projects/:id/tasks/:task_id
           PUT         /api/projects/:id/tasks/:task_id/toggle
           PUT         /api/projects/:id/tasks/:task_id/meta
           POST/DEL    /api/projects/:id/tasks/:task_id/comments

Notes:      GET/POST    /api/projects/:id/notes
           GET/PUT/DEL /api/projects/:id/notes/:note_id
           GET         /api/projects/:id/notes-titles
           GET         /api/projects/:id/notes-search

Backlinks:  GET         /api/backlinks/notes/:id/links
           GET         /api/backlinks/notes/:id/backlinks
           GET         /api/backlinks/notes/:id/forward-links
           POST        /api/backlinks/links/rebuild
           GET         /api/backlinks/links/stats

Prompts:    GET/POST    /api/prompts
           GET/PUT/DEL /api/prompts/:id
           GET         /api/prompts/folders
           GET         /api/prompts/search/semantic?q=

Daily:      GET/POST    /api/daily/:date
           GET         /api/daily/today

Git:        GET         /api/git/status
           POST        /api/git/commit
           POST        /api/git/push
           GET         /api/git/log
           GET         /api/git/diff

WebSocket:  WS          /ws
```

---

## Known Issues / Technical Debt

1. **Axum nested route limitation**: Path parameters from parent routes are NOT automatically available in nested route handlers. Project task routes use explicit routes instead of `.nest()`.

2. **Some warnings remain**: Unused methods in `locks.rs` and `git.rs` (reserved for future use).

---

## Completed (v0.3.0)

- [x] **Task time estimates** — `estimated_minutes` field for workload planning
- [x] **Quick-add task** — Global floating button to create tasks from any view (Ctrl+Shift+A)
- [x] **Priority colors** — Visual color indicators (High=red, Medium=yellow, Low=gray)
- [x] **Backlinks between notes** — `/link` slash command, BacklinksPanel, link index with API

## In Progress (v0.4.0)

See `ROADMAP.md` for v0.4.0 planning.

---

## Planned (Post v0.3.0)

- UI polish and animations
- Responsive sidebar
- Global hotkey (Ctrl+Shift+Space)
- Graph view of note connections
- Export (PDF / HTML)
- Custom themes
- Tantivy search (if >5000 notes)
- Task dependencies (blocked by)
- Sort task list by due date / priority
- Overdue indicator in sidebar and dashboard

---

## Development

```bash
# Backend (from backend/)
cargo run              # API server on :3000

# Frontend (from frontend/)
npm run dev            # Dev server on :5173
npm run build          # Build to dist/
```

**Environment:**
- Windows + PowerShell
- Rust 2021 edition
- Branch: `master`

---

## Documentation Index

See `docs/index.md` for complete documentation guide.

Key docs:
- `docs/API.md` — Complete REST API reference
- `docs/ARCHITECTURE.md` — System design and technical details
- `HANDOVER.md` — Session handover notes and next tasks
- `ROADMAP.md` — Feature roadmap and release planning
- `CHANGELOG.md` — Version history and completed work

---

## AI Rules

- Follow existing code patterns and conventions
- Use Context7 MCP tool to fetch library documentation when needed
- Prefer incremental, verifiable changes
- File system is source of truth
- No databases, no cloud services
