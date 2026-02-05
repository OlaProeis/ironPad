# Ironpad — Implementation Checklist

This checklist tracks **what is planned vs what is done**, based on the PRD.
It is the authoritative execution status for the project.

---

## Phase 0 — Preparation ✅ (COMPLETED)

### Repository & Tooling
- [x] Create project root structure
- [x] Initialize Rust backend (`ironpad`, edition 2021)
- [x] Add backend dependencies (Axum, Tokio, notify, git2, etc.)
- [x] Verify backend builds (`cargo check`)

### Backend Scaffolding
- [x] Create `routes/`, `services/`, `models/` modules
- [x] Create placeholder files for all planned backend components
- [x] Prepare WebSocket and file watcher modules

### Data Layer
- [x] Create `data/` directory structure
- [x] Create initial files (`index.md`, `inbox.md`)
- [x] Initialize `data/` as its own Git repository

### Project Meta
- [x] Create `ai-context.md`
- [x] Create implementation checklist

---

## Phase 1 — MVP ✅ (COMPLETED)

### Backend Core
- [x] Implement `main.rs` (Axum server bootstrap)
- [x] Dynamic port selection (3000–3010)
- [x] Auto-open system browser on startup
- [x] Serve static frontend files (production path)

### Notes (CRUD)
- [x] List notes from filesystem
- [x] Read markdown file by ID
- [x] Create new note with auto-generated frontmatter
- [x] Update note with auto-save + timestamp update
- [x] Archive note on delete (move to `data/archive/`)

### Frontmatter Automation
- [x] Parse/serialize frontmatter
- [x] Deterministic ID from path
- [x] Auto-manage `created`/`updated` timestamps
- [x] Preserve user-defined fields

### Frontend (Basic)
- [x] Vue 3 + Vite setup
- [x] Note list sidebar
- [x] Note viewer/editor (textarea)
- [x] Create/archive note actions
- [x] Auto-save on edit

---

## Phase 2 — Core Daily Driver ✅ (COMPLETED)

### Real-Time Sync
- [x] File system watching (`notify` crate)
- [x] WebSocket server for real-time updates
- [x] External edit detection + UI notifications
- [x] Filter out own saves from notifications

### Search
- [x] Full-text search (ripgrep with fallback)
- [x] Search endpoint (`GET /api/search?q=`)
- [x] UI search integration (Ctrl+K)

### Git Integration
- [x] Git status endpoint
- [x] Auto-commit (60-second batching)
- [x] Manual commit button
- [x] Git status indicator in UI

### Projects
- [x] Project creation (folder + `index.md` + `assets/`)
- [x] List projects API
- [x] Project task file creation (`tasks.md`)

### Tasks (Basic)
- [x] Task parsing from markdown checkboxes
- [x] Tasks API (list all tasks)
- [x] Task view in sidebar

---

## Phase 3 — Full PRD Compliance ✅ (COMPLETED)

### Projects & Tasks (Per PRD Section 7.2, 7.3)
- [x] Project-specific task endpoint (`GET /api/projects/:id/tasks`)
- [x] Task toggle endpoint (update checkbox state)
- [x] Add task via UI (append to `tasks.md`)
- [x] Task sections: Active, Completed, Backlog
- [x] Project task view at `/projects/:id/tasks` route

### File Locking (Per PRD Section 7.7)
- [x] Backend tracks open files via WebSocket
- [x] File lock when Task View opens
- [x] Editor shows "Read-Only" if file locked
- [x] Auto-unlock when view closes

### Daily Notes (Per PRD Section 6)
- [x] Create `data/daily/` directory
- [x] Daily note endpoint (create/get today's note)
- [x] Daily note templates
- [x] Daily notes in sidebar

### CodeMirror 6 Editor (Per PRD Section 9.3)
- [x] Install CodeMirror 6 dependencies
- [x] Replace textarea with CodeMirror
- [x] Markdown syntax highlighting
- [x] Line numbers
- [x] Keyboard shortcuts

### Markdown Preview (Per PRD Section 5)
- [x] Split view (editor + preview)
- [x] Markdown-it rendering
- [x] CommonMark consistency

### Assets API (Per PRD Section 8)
- [x] `POST /api/assets/upload` endpoint
- [x] `GET /api/assets/:project/:file` endpoint
- [x] Image upload UI in editor

### Git Advanced (Per PRD Section 7.5)
- [x] Git conflict detection
- [x] Conflict warning banner in UI
- [x] `POST /api/git/push` endpoint
- [x] `GET /api/git/conflicts` endpoint

### Frontend Architecture (Per PRD Section 14)
- [x] Vue Router for navigation
- [x] Pinia state management
- [x] Separate view components (NotesView, TasksView, ProjectView)
- [x] WebSocket composable

---

## Phase 4 — Enhanced Task System ✅ (COMPLETED)

### Dashboard
- [x] Cross-project dashboard as home page (`/`)
- [x] Project cards with active task counts and summaries
- [x] Click-through to project or task detail
- [x] Clickable "Ironpad" title navigates to dashboard

### Tags
- [x] Tags field in task frontmatter (YAML sequence)
- [x] Backend parses/writes tags on task CRUD
- [x] Tag pills displayed on task list items
- [x] Tag filter bar — click to filter tasks by tag
- [x] Tag editor in task detail panel with autocomplete
- [x] `projectTags` computed getter for all unique tags in project

### Subtasks
- [x] `parent_id` field in task frontmatter
- [x] Backend accepts `parent_id` on task creation
- [x] Task list groups subtasks under parent (indented)
- [x] Subtask count badge on parent tasks (completed/total)
- [x] Subtask panel in task detail with inline add
- [x] Subtasks clickable to view/edit

### Recurring Tasks
- [x] `recurrence` and `recurrence_interval` fields in frontmatter
- [x] Backend auto-creates next instance on recurring task completion
- [x] Due date advanced by interval (daily/weekly/monthly/yearly)
- [x] Recurrence picker (dropdown) in task detail panel
- [x] Recurrence indicator on task list items

### Calendar View
- [x] Month grid calendar at `/calendar`
- [x] Tasks with due dates plotted on day cells
- [x] Daily notes shown as blue dots
- [x] Color-coded urgency (overdue/today/soon)
- [x] Month navigation (prev/next) + Today button
- [x] Click task → navigate to detail, click date → daily note
- [x] Calendar link in sidebar navigation

### Due Date
- [x] Inline date picker in task detail panel
- [x] Clear due date button
- [x] Due date display with color-coded urgency on task items

---

## Phase 5 — Polish

- [ ] UI polish and animations
- [ ] Responsive sidebar
- [ ] Better error handling/messages
- [ ] Loading states

---

## Phase 6 — Future / Optional

- [ ] Global hotkey (Ctrl+Shift+Space)
- [ ] System tray mode
- [ ] Backlinks between notes
- [ ] Graph view
- [ ] Export (PDF / HTML)
- [ ] Custom themes
- [ ] Tantivy search (if >5000 notes)
- [ ] Task dependencies (blocked by)
- [ ] Time estimates on tasks
- [ ] Calendar drag-and-drop rescheduling
- [ ] Week/day calendar views

---

## Rules

- No item is marked complete unless it is implemented and verified.
- New features must be added to this checklist before implementation.
- If it's not on this list, it's out of scope.
