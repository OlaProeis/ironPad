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
- [x] Dynamic port selection (37891–37990, optional `IRONPAD_PORT`)
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

## Release 0.4.0

Released 2026-11-20. The acceptance rows below are checked. Phase 5 and Phase 6 are still open. There is no further 0.4 PRD.

Spec: [`PRD-0.4.0.md`](./PRD-0.4.0.md) §11 for the blocks through Git backup, [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md) §11 for the Library sections, and [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md) §12 for the GitHub sections.

### Port (already done)

- [x] Default scan is 37891–37990, and `IRONPAD_PORT` pins a port.
- [x] Startup logs the URL to open.

### Task list and dashboard

- [x] Task view sorts by created, due date, or priority, including across reload.
- [x] Subtasks stay under the parent they already belong to.
- [x] Sidebar shows the active project’s overdue count, and a task due today is not in it.
- [x] Dashboard and sidebar agree on that count for the same project.
- [x] Dashboard tag chips filter incomplete tasks across projects, and clearing the chip restores the full dashboard.
- [x] Day view shows used/budget against 6 hours by default, turns amber over budget, and remembers a changed budget.
- [x] Bulk mark-complete skips tasks that are already done and still spawns the next instance of a recurring task.
- [x] Bulk move and bulk tag leave `completed` and `section` consistent, and a mid-list failure does not continue.

### Git backup

- [x] A private HTTPS or SSH URL can be saved as `origin` from the Git panel, and a URL with embedded credentials is rejected.
- [x] Pull fast-forwards and refuses a diverged history without changing files.
- [x] Auth failure, rejected push, and a dirty tree each show the message in section 9.5.
- [x] After the first push, ahead/behind reflect the upstream.
- [x] `docs/git-backup.md` is linked from the panel and does not describe live co-editing.

### Library label

- [x] The top bar and the project sidebar say Library. Routes remain `prompts` and `project-prompts`.

### Linked files

- [x] A prompt with no `kind` still opens, edits, and saves as it does today.
- [x] Adding a link writes a pointer under the prompt folders and does not copy the source file into `data/`.
- [x] A project link’s `source` is relative to the folder root in `localStorage`. That root is not stored under `data/`.
- [x] Opening a link shows the source file’s current text. An outside edit appears on Reload and when the window regains focus.
- [x] Save on a link can change title, folder, tags, description, source, or template, and leaves the source file’s bytes unchanged.
- [x] Delete removes the pointer and leaves the source file in place.
- [x] A missing source stays in the list and can be relinked.
- [x] A file over 1 MiB, a directory, or a disallowed suffix is refused.
- [x] `{{variables}}` are substituted only for prompts and for links with template turned on.

### Rendered and raw

- [x] The open item switches between raw source and rendered markdown.
- [x] Saving a prompt writes the raw draft, including when the pane was on rendered.
- [x] A markdown link opens on rendered. A non-markdown link stays on raw, and rendered is unavailable.
- [x] The mode is remembered across reload.

### Versions

- [x] Versions (Git) collapses, defaults to collapsed, and remembers the choice.
- [x] An expanded prompt still shows that file’s Git log.
- [x] An expanded link does not list Ironpad commits as that file’s history.

### Copy bundle, suggest, open

- [x] Copy bundle puts the selected items in one markdown blob: prompts as raw body, links as current source text, with no variable substitution.
- [x] With a project root set, the library offers `ai-context.md`, `ROADMAP.md`, and `docs/index.md` when those files exist, and creates a link only after confirmation.
- [x] Open externally and Copy path use the resolved source. A missing file does not open.

### Link

- [x] A project can save `owner/name`, and that value is still on `index.md` after the project page is saved.
- [x] A URL, a `.git` suffix, or an empty value is rejected with the sentence in section 5.6.
- [x] Remove deletes the link and the snapshot, and leaves a task created earlier in place.
- [x] Saving a repository does not change the notes repo’s `origin`.

### Lists and refresh

- [x] Open issues and open pull requests appear in separate lists. A pull request returned by the issues endpoint is not shown as an issue.
- [x] Assigned, created-by-me, mine, and review-requested filters match the logged-in login and survive a reload. The default is All open.
- [x] Refresh on open when the snapshot is older than 15 minutes, from the button, and on the 15-minute timer while the view is open.
- [x] A 304 keeps the current rows and updates the shown time.
- [x] A failed refresh keeps the previous rows and shows one of the sentences in section 5.6.
- [x] The snapshot directory is outside `data/`, and auto-commit does not pick it up.

### Login

- [x] A working `gh auth token` is used without a token field in the UI.
- [x] With no `gh` login, `GH_TOKEN` or `GITHUB_TOKEN` is used. With none of the three, the view shows the needs-a-login sentence.
- [x] No API response, snapshot file, or log line contains the token.

### Read, track, dashboard

- [x] Opening a row shows the Markdown body and up to 30 comments, read-only, and does not change the item on GitHub.
- [x] An opened pull request shows the combined status word when GitHub returns one.
- [x] Track as task creates one Active task with the title, labels as tags, milestone date when present, `github_url`, and `github_number`.
- [x] Tracking the same URL again opens the existing task and does not create a second one.
- [x] Completing that task does not call GitHub.
- [x] A project card shows assigned and review counts from the last snapshot, and hides the line when both are zero.

---

## Phase 6 — Future / Optional

Shipped earlier, kept here so this list matches the roadmap:

- [x] System tray mode (0.2.0)
- [x] Backlinks between notes (0.3.0)
- [x] Time estimates on tasks, `estimated_minutes` (0.3.0)
- [x] Calendar drag-and-drop rescheduling (0.3.0)
- [x] Week/day calendar views (0.3.0)

Still open:

- [ ] Graph view (reads the 0.3 backlink index)
- [ ] Export (PDF / HTML)
- [ ] Custom themes
- [ ] Tantivy search (if >5000 notes)
- [ ] Kanban board view
- [ ] Note-body tag extraction (task-frontmatter dashboard filter is in the 0.4.0 working tree)
- [ ] Task templates

### Proposed for 0.5.0

- [ ] Run a prompt on a note or task, with optional localhost model output
- [ ] Weekly review file in `data/reviews/`
- [ ] Waiting across projects (`waiting_on`: a person or a task id)
- [ ] Capture strip on Ctrl+Shift+Space (app stays in the background; replaces "hotkey brings the app forward")
- [ ] Project blueprints

---

## Rules

- No item is marked complete unless it is implemented and verified.
- New features must be added to this checklist before implementation.
- If it's not on this list, it's out of scope.
