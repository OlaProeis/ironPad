# AI Context — Ironpad

Paste this into every new AI chat for project context.

---

## What It Is

**Ironpad** is a local-first, file-based personal project & knowledge management system.

- **Backend**: Rust (Axum 0.8, Tokio)
- **Frontend**: Vue 3 (Vite)
- **Data**: Plain Markdown files with YAML frontmatter
- **Versioning**: Local Git repository
- **UI**: System browser (no Electron)

---

## Core Principles

1. **Files are the database** — filesystem is source of truth
2. **Local-first** — works fully offline
3. **External editing supported** — VS Code, Obsidian, Vim all work
4. **Backend owns metadata** — `id`, `created`, `updated` are auto-managed
5. **Low ceremony** — minimal config, no manual metadata editing

---

## Current Architecture

```
ironpad/
├── backend/           # Rust Axum server
│   └── src/
│       ├── main.rs           # Server bootstrap, WebSocket, routes
│       ├── routes/           # API endpoints
│       ├── services/         # Business logic (filesystem, git, search)
│       ├── models/           # Data structures
│       ├── websocket.rs      # Real-time sync
│       └── watcher.rs        # File system watching
├── frontend/          # Vue 3 SPA
│   └── src/
│       ├── App.vue           # Root component with router-view
│       ├── main.ts           # Entry point (Pinia + Vue Router)
│       ├── router/           # Vue Router config
│       ├── stores/           # Pinia stores (notes, projects, tasks, ui, websocket, git)
│       ├── views/            # Route views (DashboardView, ProjectView, TasksView, CalendarView, DailyView)
│       ├── components/       # Reusable components (Sidebar, MarkdownEditor, etc.)
│       ├── composables/      # Vue composables (useWebSocket)
│       ├── api/              # API client (client.ts)
│       └── types/            # TypeScript types (index.ts)
└── data/              # User data (separate git repo)
    ├── notes/         # Standalone notes
    ├── projects/      # Project folders
    │   └── {project}/
    │       ├── index.md    # Project overview
    │       ├── notes/      # Project-specific notes
    │       └── tasks/      # Individual task files (task-YYYYMMDD-HHMMSS.md)
    ├── daily/         # Daily notes (YYYY-MM-DD.md)
    ├── archive/       # Archived items
    ├── index.md       # Landing page
    └── inbox.md       # Quick capture
```

---

## Implemented Features

### Backend
- API-only server (no frontend serving, no browser auto-open)
- Dynamic port (3000-3010)
- Notes CRUD with atomic writes
- Frontmatter auto-management
- WebSocket server for real-time sync + file locking
- File watcher (filters own saves)
- Search (ripgrep with fallback)
- Git status + auto-commit (60s batching) + push + conflict detection
- **Full Git panel** with commit history, diff viewer, custom commit messages
- Git remote info (ahead/behind tracking), fetch support
- Projects API with notes management
- **File-based Tasks API** — each task is a markdown file with frontmatter
  - Fields: id, title, completed, section, priority, due_date, is_active, tags, parent_id, recurrence, recurrence_interval
  - Rich text descriptions with markdown support
  - Sorted by created date (stable ordering)
  - **Subtasks** — tasks with `parent_id` link to a parent task
  - **Tags** — YAML sequence in frontmatter, per-task labels for filtering
  - **Recurring tasks** — when completing a recurring task, auto-creates next instance with advanced due date
- Daily notes API (`/api/daily`, `/api/daily/today`, `/api/daily/:date`)
- Assets API (upload + serve)

### Frontend
- Vue Router navigation
- Pinia state management
- **Milkdown WYSIWYG editor** — ProseMirror-based, renders markdown as you type
  - CommonMark + GFM support (tables, strikethrough, task lists)
  - **Toolbar** with formatting buttons (bold, italic, headings, links, images, code, lists, quotes)
  - **Image upload** — click image button, select file, auto-uploads and inserts markdown
  - History (undo/redo), clipboard, indentation plugins
- **Dark theme by default** with toggle button (persists to localStorage)
- Sidebar with Notes/Projects/Daily/Calendar sections + task counts
- Search panel (Ctrl+K)
- **Dashboard view** (home page) — all projects as cards with active task summaries
  - Click project to navigate, click task to open detail
  - Shows active/backlog/overdue counts per project
- **Split-panel Task view** with:
  - Task list (Active/Backlog/Completed sections)
  - Task detail editor with markdown descriptions
  - Preview toggle for rendered markdown
  - Active/Backlog toggle button
  - **Due date picker** — inline date input to set/clear due dates
  - Due date display with color-coded urgency
  - **Tag system** — add/remove tags with autocomplete from project tags
  - **Tag filter bar** — click tags to filter task list
  - **Subtasks** — expandable subtasks under parent tasks, add subtask inline
  - **Recurrence picker** — set daily/weekly/monthly/yearly recurrence
  - Inline title editing (double-click)
- **Calendar view** — month grid showing tasks by due date
  - Tasks with due dates plotted on calendar cells
  - Daily notes shown as blue dots
  - Color-coded urgency (overdue, today, soon)
  - Month navigation + Today button
  - Click task to navigate to detail, click date to open daily note
- **Split-panel Notes view** (per project)
- **Git panel** — slide-out panel with:
  - Commit history with expandable diffs
  - Working directory changes with line-by-line diff
  - Custom commit messages (Ctrl+Enter to commit)
  - Push/Fetch buttons with ahead/behind indicators
  - File status icons (added/modified/deleted/renamed)
- Git status indicator in sidebar (click to open panel)
- WebSocket real-time updates
- File lock banners (read-only mode when locked)
- Conflict warning banner
- Fullscreen layout (uses all available space)

### API Endpoints
```
GET/POST    /api/notes
GET/PUT/DEL /api/notes/:id
GET/POST    /api/projects
GET/PUT     /api/projects/:id
GET/PUT     /api/projects/:id/content

# Project Notes (file-based)
GET/POST    /api/projects/:id/notes
GET/PUT/DEL /api/projects/:id/notes/:note_id

# Project Tasks (file-based, each task is a .md file)
GET/POST    /api/projects/:id/tasks
GET/PUT/DEL /api/projects/:id/tasks/:task_id
PUT         /api/projects/:id/tasks/:task_id/toggle
PUT         /api/projects/:id/tasks/:task_id/meta

GET         /api/tasks              # All tasks across projects
GET         /api/daily
GET         /api/daily/today
GET/POST    /api/daily/:date
POST        /api/assets/upload
GET         /api/assets/:project/:file
GET         /api/search?q=
GET         /api/git/status
POST        /api/git/commit
POST        /api/git/push
GET         /api/git/conflicts
GET         /api/git/log              # Commit history (limit param)
GET         /api/git/diff             # Working directory diff
GET         /api/git/diff/:commit_id  # Diff for specific commit
GET         /api/git/remote           # Remote repository info
POST        /api/git/fetch            # Fetch from remote
WS          /ws
```

---

## Implemented in Phase 3

- CodeMirror 6 editor with markdown syntax highlighting
- Markdown preview with split view
- Vue Router for navigation (`/`, `/projects/:id`, `/projects/:id/tasks`, `/calendar`, `/daily`)
- Pinia state management (notes, projects, tasks, ui, websocket, git stores)
- Project-specific task view with toggle and add functionality
- File locking via WebSocket (Task View vs Editor)
- Daily notes (`data/daily/`) with templates
- Assets upload API (`/api/assets/upload`, `/api/assets/:project/:file`)
- Git push and conflict detection (`/api/git/push`, `/api/git/conflicts`)

---

## Implemented in Phase 4

- **File-based tasks** — each task is a separate markdown file with YAML frontmatter
  - Supports rich text descriptions with images
  - New fields: `due_date`, `is_active`
  - Task files stored in `projects/{id}/tasks/task-YYYYMMDD-HHMMSS.md`
- **Split-panel task view** — list on left, detail editor on right
- **Markdown preview toggle** — side-by-side raw/rendered view
- **Active/Backlog toggle** — button to move tasks between states
- **Project notes** — separate notes folder per project with split-panel view
- **Stable list sorting** — sorted by created date (not updated)
- **Backend API-only mode** — no frontend serving, no browser auto-open
- **Fullscreen layout** — uses all available browser space

---

## Known Issues / Technical Debt

1. **Project index note ID format**: Project notes use `{slug}-index` as their ID (e.g., `ferrite-index`). Projects created before this fix have incorrect IDs in frontmatter.

2. **Axum nested route limitation**: Path parameters from parent routes are NOT automatically available in nested route handlers. Project task routes use explicit routes instead of `.nest()`.

3. **Some warnings remain**: Unused methods in `locks.rs` and `git.rs` (reserved for future use).

---

## Implemented in Phase 5

- **Dashboard view** — cross-project home page with task summaries per project
- **Tags system** — per-task tags stored in frontmatter, filter bar in task list, autocomplete
- **Subtasks** — tasks with `parent_id`, grouped under parents in list, inline creation
- **Recurring tasks** — daily/weekly/monthly/yearly with auto-creation on completion
- **Calendar view** — month grid with tasks by due date + daily note indicators
- **Due date picker** — inline date input in task detail panel
- **Clickable app title** — "Ironpad" navigates to dashboard

---

## Not Yet Implemented (Phase 6+)

- UI polish and animations
- Responsive sidebar
- Global hotkey (Ctrl+Shift+Space)
- System tray mode
- Backlinks between notes
- Graph view
- Export (PDF / HTML)
- Custom themes
- Tantivy search (if >5000 notes)
- Production packaging (Tauri or similar)
- Task dependencies (blocked by)
- Time estimates on tasks
- Calendar drag-and-drop rescheduling
- Week/day calendar views

---

## Key Technical Decisions

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
| Editor Legacy | CodeMirror 6 (MarkdownEditor.vue, kept for reference) |
| State management | Pinia stores |
| Routing | Vue Router (history mode) |
| File locking | WebSocket-based, per-client locks |
| Project note ID | `{slug}-index` format |
| Task storage | Individual .md files in `tasks/` folder |
| List sorting | By created date (stable, not affected by edits) |
| Backend mode | API-only (no frontend serving) |
| Theme | Dark by default, toggle to light, persists to localStorage |
| Tags | YAML sequence in frontmatter, project-scoped filtering |
| Subtasks | Separate task files with `parent_id` field linking to parent |
| Recurring tasks | On completion, backend auto-creates next instance with advanced due date |
| Calendar | Pure frontend month grid, tasks filtered by `due_date` presence |
| Dashboard | Home route `/`, loads all projects + all tasks for cross-project summary |

---

## Critical: Milkdown Editor Lifecycle

The Milkdown editor requires careful handling when switching between notes/tasks:

**Components:**
- `MilkdownEditor.vue` — Wrapper with `:key` prop for recreation
- `MilkdownEditorCore.vue` — Actual editor using `useEditor` hook from `@milkdown/vue`

**Pattern for switching content:**
```javascript
// Views use a separate editorKey ref (not the noteId/taskId directly)
// Content MUST be set BEFORE updating editorKey

// CORRECT order:
editorContent.value = loadedContent  // Set content first
editorKey.value = noteId             // Then trigger editor recreation

// WRONG order (causes stale content):
editorKey.value = noteId             // Editor recreates with empty/stale defaultValue
editorContent.value = loadedContent  // Too late - editor already initialized
```

**Why:** The editor uses `defaultValue` from props at creation time. If the key changes before content is set, the editor initializes with wrong content and `replaceAll()` updates may fail during the async initialization window.

**State in MilkdownEditorCore must be refs, not module-level variables** — ensures clean state on component recreation.

---

## Development Commands

```bash
# Backend (from backend/)
cargo run              # API server on :3000 (no GUI)

# Frontend (from frontend/)
npm run dev            # Dev server on :5173 (connects to backend on :3000)
npm run build          # Build to dist/

# Development: Run both backend and frontend separately
# Backend is API-only, does not serve frontend
```

---

## Documentation

For detailed information, see:

| Document | Description |
|----------|-------------|
| `/README.md` | Project overview, quick start, installation |
| `/frontend/README.md` | Frontend architecture, component structure, Milkdown editor patterns |
| `/docs/ARCHITECTURE.md` | System design, service layer, data models, security considerations |
| `/docs/API.md` | Complete REST API reference with examples |
| `/HANDOVER.md` | Session handover notes, recent fixes, context for continuing work |
| `/CHECKLIST.md` | Current implementation status and progress |
| `/PRD.md` | Full product requirements document |

---

## Rules for AI

- Prefer incremental, verifiable changes
- File system is source of truth
- No databases, no cloud services
- Windows + PowerShell environment
- Rust 2021 edition
- Check CHECKLIST.md for current status
- Check PRD.md for full requirements
- Check HANDOVER.md for recent session context