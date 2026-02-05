# Product Requirements Document (PRD)
## Ironpad - Personal Project & Knowledge Management System

**Version:** 3.0  
**Date:** 2026-02-02  
**Status:** Active Development  
**Author:** Internal / Personal  
**Changelog:** v3.0 - Addressed concurrency, file watching, git conflicts, port handling, and frontmatter automation

---

## 1. Executive Summary

**Ironpad** is a **local-first, browser-based personal project management and note-taking system** powered by a Rust backend.

The system combines:
- Free-form markdown notes
- Task management per project
- Project organization
- Full-text search
- Git-based versioning
- **Real-time file system watching** for external edits
- **Automatic frontmatter management** for low-ceremony UX

**Core Philosophy:** Simplicity first, power through composition, not rigid workflows.

**Key Innovation:** Single-binary Rust executable that auto-opens your browser - no bundled Chromium, no Node.js runtime required.

---

## 2. Goals

### Primary Goals
- Provide a single place to store thoughts, notes, and tasks
- Enable lightweight project organization without heavy structure
- Support incremental evolution of features
- Keep the system easy to understand, modify, and extend
- Learn Rust through practical application
- **Eliminate manual metadata management** (auto-update timestamps, IDs)
- **Support external editing** (VS Code, Obsidian, etc.)

### Technical Goals
- Zero-dependency data storage (plain files only)
- Fast search (<100ms for 1000 notes)
- Responsive UI (<16ms frame time)
- Startup time <500ms
- Tiny binary size (<15 MB)
- No browser bundling (use system browser)
- **Graceful port conflict handling**
- **Real-time sync between UI and filesystem**
- **Robust git conflict handling**

### Non-Goals (v1)
- Multi-user collaboration
- Cloud sync (Git remote is optional)
- Permissions/roles
- Complex workflow automation
- Enterprise-grade task management
- Mobile app (mobile access not required)

---

## 3. Design Principles

### 1. Local-First
- No external services required
- Works fully offline
- Data stored on local file system
- **File system is the source of truth** (not UI state)

### 2. Notes-First, Tasks-Per-Project
- Notes are the primary unit
- Each project has its own task list
- Tasks don't float independently

### 3. File-Based Storage
- Data stored as Markdown files
- Editable outside the app
- Human-readable formats only
- **External editors fully supported** (VS Code, Obsidian, Vim)

### 4. Low Ceremony
- Minimal configuration
- Minimal UI friction
- No complex setup wizards
- **Automatic metadata management** (no manual timestamps/IDs)

### 5. Future-Proof
- Easy to migrate to other tools
- Git-friendly formats
- No vendor lock-in

### 6. Simplicity Principles
- **No abstraction layers** - Files are the database
- **No build step for data** - Markdown is human-readable
- **No proprietary formats** - Everything is standard
- **No server required** - Runs entirely on localhost
- **No complex workflows** - Write, save, done

---

## 4. Architecture

### Overview

```
User launches executable
         ↓
   Rust Backend (Axum)
   - HTTP server on dynamic port (3000-3010)
   - Serves Vue frontend (static files)
   - REST API for file operations
   - Git operations with conflict handling
   - Full-text search
   - File system watcher (notify crate)
   - WebSocket server for real-time updates
         ↓
   Auto-opens default browser
   → http://localhost:{port}
         ↓
   Vue 3 Frontend (in browser)
   - Markdown editor
   - Task management
   - Project switching
   - WebSocket client (receives file change events)
         ↓
   Bidirectional real-time sync
   ← WebSocket → File System Changes
```

### Why Not Electron?

| **Electron** | **Ironpad (Rust)** |
|-------------|-------------------|
| 150-300 MB bundle | 5-15 MB binary |
| Bundles Chromium | Uses system browser |
| 200-500 MB RAM | 10-50 MB RAM |
| 2-5s startup | <500ms startup |
| Complex distribution | Single executable |

**Rationale:** No need to bundle an entire browser when every user already has one.

### Technology Stack

#### Backend (Rust)
- **Axum** - Web framework (simple, fast, learning-friendly)
- **Tokio** - Async runtime
- **Tower** - Middleware
- **Serde** - JSON serialization
- **markdown-rs** - Markdown parsing with frontmatter (CommonMark compliant)
- **git2** - Git operations with lock handling
- **webbrowser** - Cross-platform browser launching
- **notify** - File system watching for external changes
- **axum-ws** or **tower-websockets** - WebSocket support for real-time updates
- **ripgrep** library or **tantivy** - Fast search

#### Frontend (Vue 3)
- **Vue 3** - UI framework (Composition API)
- **Vite** - Build tool
- **CodeMirror 6** - Markdown editor with syntax highlighting
- **Pinia** - State management (minimal caching, trust filesystem)
- **markdown-it** - Markdown rendering (CommonMark mode for consistency)
- **Native WebSocket API** - Real-time file change notifications

#### Data Storage
- **Markdown files** (.md) on local file system
- **YAML frontmatter** for metadata (auto-managed by backend)
- **Git repository** for versioning
- **No database** - files are the database

---

## 5. User Experience

### Mental Model
- Similar to a notebook system (OneNote / Obsidian)
- A **front page** (index.md) acts as entry point
- Users can create notes and projects
- Projects contain notes + dedicated task list
- Simple navigation via sidebar
- **Edit anywhere** - changes in VS Code/Obsidian auto-sync to UI
- **Conflict-free** - UI prevents simultaneous edits of same file

### Core UI Areas

#### 1. Sidebar (Left)
- **Projects** section
  - List of all projects
  - Quick switch between projects
- **Notes** section
  - List of standalone notes
  - Daily notes
- **Quick actions**
  - New note
  - New project
  - Search
- **Git status indicator** (changes pending commit)

#### 2. Main Area (Center)
- **Editor view**
  - CodeMirror markdown editor
  - Syntax highlighting
  - Auto-save (2s debounce)
  - **File lock indicator** (shows if file open in Task View)
  - **External edit notification** (shows banner if file changed externally)
- **Split view option**
  - Editor on left
  - Preview on right

#### 3. Task View (Separate Page)
- View: `/projects/:id/tasks`
- Shows tasks for currently selected project
- Parse checkboxes from `data/projects/{id}/tasks.md`
- Sections: Active, Completed, Backlog
- Quick checkbox toggle
- **Prevents editor access** - If Task View open, editor shows "Read-Only" mode

---

## 6. Data Model

### File Structure

```
data/
  .git/                   # Git repository
  index.md                # Front page / landing
  inbox.md                # Quick capture
  
  daily/                  # Daily notes (optional)
    2026-02-02.md
    2026-02-03.md
  
  projects/               # Project folders
    ironpad/
      index.md            # Project overview
      tasks.md            # Task list
      notes.md            # Miscellaneous notes (optional)
      assets/             # Images, attachments
        screenshot.png
    homelab/
      index.md
      tasks.md
      assets/
  
  notes/                  # Standalone notes
    ideas.md
    rust-learning.md
    assets/               # Shared assets
      diagram.png
  
  archive/                # Completed/archived items
    old-project/
```

### Frontmatter Schema

#### Standard Fields (All Files) - AUTO-MANAGED
```yaml
---
# Auto-generated by backend (user never edits these manually)
id: ironpad-index        # Derived from filename: {folder}-{filename}
type: note                # Detected from file location
created: 2026-02-02T01:00:00Z   # Set on file creation
updated: 2026-02-02T01:15:00Z   # Auto-updated on every save

# Optional user fields
title: Ironpad Development  # Optional: display title (fallback: filename)
tags: [dev, rust, personal]  # Optional: user-defined tags
status: active               # Optional: draft|active|archived|complete
---
```

**Key Change:** Users never manually write `id`, `created`, or `updated` fields. Backend handles these automatically.

#### ID Generation Strategy
- **Format**: `{parent-folder}-{filename-without-extension}`
- **Examples**:
  - `projects/ironpad/index.md` → `id: ironpad-index`
  - `notes/ideas.md` → `id: notes-ideas`
  - `daily/2026-02-02.md` → `id: daily-2026-02-02`
- **Rationale**: Human-readable, deterministic, no UUIDs cluttering files

#### Task File Example
```yaml
---
id: ironpad-tasks          # Auto-generated from filename
type: tasks                 # Auto-detected
project_id: ironpad        # Parent folder name
created: 2026-02-01T12:00:00Z
updated: 2026-02-02T01:00:00Z  # Auto-updated on every save
---

# Tasks: Ironpad

## Active
- [ ] Set up Rust backend with Axum
- [ ] Create Vue frontend with CodeMirror
- [ ] Implement task parsing

## Completed
- [x] Write PRD
- [x] Review architecture decisions

## Backlog
- [ ] Add full-text search
- [ ] Implement Git auto-commit
```

---

## 7. Functional Requirements

### 7.1 Notes Management
- **Create** new notes via sidebar button
- **Read** note content with markdown rendering
- **Update** notes with auto-save (2s debounce after last edit)
- **Delete** notes (moves to archive/ folder)
- Notes stored as `.md` files in `data/notes/`
- **Auto-update** `updated` timestamp on every save

### 7.2 Project Management
- **Create** new projects (creates `data/projects/{id}/` folder + `assets/` subfolder)
- **View** project overview (`index.md`)
- **Switch** between projects via sidebar
- **Archive** completed projects (moves to `archive/`)
- Projects automatically get `tasks.md` file + `assets/` folder

### 7.3 Task Management
- **View** tasks per project at `/projects/:id/tasks`
- **Toggle** task completion (checkbox state)
- **Add** new tasks via UI (appends to tasks.md)
- **Organize** tasks in sections: Active, Completed, Backlog
- Tasks represented as Markdown checkboxes:
  ```markdown
  - [ ] Incomplete task
  - [x] Completed task
  ```
- **No global task view in v1** - tasks belong to projects
- **Concurrency handling**: If Task View is open for a file, Editor View shows "Read-Only - Open in Task View" banner

### 7.4 Search
- **Full-text search** across all markdown files
- Search triggered from sidebar search box
- Results show: filename, matching line, context
- **Implementation**: Use `ripgrep` as library (faster than naive grep)
- Performance target: <100ms for 1000 notes
- **Future**: Migrate to Tantivy if needed (>5000 notes)

### 7.5 Git Integration
- **Auto-commit** with time-based batching (5 minutes)
- **Manual commit** button available
- **Commit message format**: `Auto-save: {timestamp}` or user-provided message
- **Git history** viewable via external tools (GitKraken, git log, etc.)
- **Remote push** optional (manual via git push or UI button)
- **Lock handling**: If `.git/index.lock` exists, skip auto-commit and retry next cycle
- **Conflict detection**: If `git status` shows conflicts, show warning banner in UI

### 7.6 File System Watching (NEW)
- **Backend watches** `data/` directory using `notify` crate
- **Detects changes** made by external editors (VS Code, Obsidian, Vim)
- **Sends WebSocket message** to frontend: `{ type: "file_changed", path: "notes/ideas.md" }`
- **Frontend response**:
  - If file is currently open in editor → Show banner: "File changed externally. Reload?"
  - If file is not open → Auto-refresh sidebar file list
  - If Task View is open for that file → Auto-reload task list
- **Debouncing**: Batch file changes (100ms) to avoid spamming WebSocket

### 7.7 Concurrency Control (NEW)
- **Single-file lock**: Only one view (Editor OR Task View) can edit a file at a time
- **Implementation**:
  - Backend tracks "open files" via WebSocket connection state
  - When Task View opens `tasks.md`, backend marks file as "locked for task editing"
  - If user tries to open same file in Editor, show "Read-Only" mode
  - When Task View closes, file unlocked automatically
- **Rationale**: Prevents race conditions between checkbox toggles and text edits

---

## 8. API Design

### REST Endpoints

#### Notes
```
GET    /api/notes              # List all notes
GET    /api/notes/:id          # Get note content
POST   /api/notes              # Create new note (auto-generates frontmatter)
PUT    /api/notes/:id          # Update note content (auto-updates 'updated' field)
DELETE /api/notes/:id          # Delete note (archive)
```

#### Projects
```
GET    /api/projects           # List all projects
GET    /api/projects/:id       # Get project details
POST   /api/projects           # Create new project (creates folder + assets/)
PUT    /api/projects/:id       # Update project
DELETE /api/projects/:id       # Archive project
```

#### Tasks
```
GET    /api/projects/:id/tasks    # Get tasks for project
PUT    /api/projects/:id/tasks    # Update tasks for project
POST   /api/tasks/lock/:id        # Lock file for task editing
POST   /api/tasks/unlock/:id      # Unlock file
```

#### Search
```
GET    /api/search?q={query}      # Search all content (ripgrep-powered)
```

#### Git
```
POST   /api/git/commit            # Manual commit with message
GET    /api/git/status            # Get git status (detects conflicts)
POST   /api/git/push              # Push to remote (if configured)
GET    /api/git/conflicts         # Check for merge conflicts
```

#### Assets (NEW)
```
POST   /api/assets/upload         # Upload image/file to project assets/
GET    /api/assets/:project/:file # Retrieve asset file
```

### WebSocket Endpoints (NEW)

```
WS     /ws                        # WebSocket connection for real-time updates

# Messages from backend → frontend:
{ type: "file_changed", path: "notes/ideas.md", timestamp: "2026-02-02T01:00:00Z" }
{ type: "file_deleted", path: "notes/old.md" }
{ type: "file_created", path: "daily/2026-02-03.md" }
{ type: "git_conflict", files: ["notes/ideas.md"] }

# Messages from frontend → backend:
{ type: "subscribe_file", path: "notes/ideas.md" }  # Track active file
{ type: "unsubscribe_file", path: "notes/ideas.md" }
```

---

## 9. Technical Implementation Details

### 9.1 Auto-Save Strategy
- **Decision**: 2-second debounce after last edit
- **Rationale**: Balance between data safety and performance
- **Implementation**: Frontend debounces PUT requests
- **Backend behavior**: On PUT, auto-update `updated` timestamp in frontmatter

### 9.2 Git Commit Strategy
- **Decision**: Time-based batching (5 minutes) + manual commit button
- **Rationale**: Clean history, reduced I/O, user control
- **Implementation**: 
  - Rust background task (tokio::spawn) runs every 5 minutes
  - Checks `git status --porcelain` for changes
  - If `.git/index.lock` exists → Skip and log warning
  - If changes exist → `git add .` → `git commit -m "Auto-save: {timestamp}"`
  - If commit fails (lock, conflict) → Show error in UI via WebSocket

### 9.3 Editor Choice
- **Decision**: CodeMirror 6 with markdown mode
- **Rationale**: Mature, performant, excellent UX out-of-box
- **Features**: Syntax highlighting, line numbers, keyboard shortcuts

### 9.4 Search Implementation (v1)
- **Decision**: Use `ripgrep` as library (via `grep` crate or direct integration)
- **Rationale**: 
  - Faster than naive grep (uses SIMD, optimized algorithms)
  - Handles >1000 files easily
  - Used by VS Code, Sublime Text
- **Future**: Migrate to Tantivy if search becomes slow (>5000 notes)

### 9.5 Browser Launch
- **Decision**: Use `webbrowser` crate to open default browser
- **Rationale**: Cross-platform, simple, no browser bundling
- **Fallback**: Print URL if browser launch fails

### 9.6 Port Conflict Handling (NEW)
- **Decision**: Dynamic port selection (3000-3010 range)
- **Implementation**:
  ```rust
  async fn find_available_port() -> u16 {
      for port in 3000..=3010 {
          if let Ok(listener) = TcpListener::bind(("127.0.0.1", port)).await {
              return port;
          }
      }
      panic!("No available ports in range 3000-3010");
  }
  ```
- **User experience**: Always works, even if other dev tools running
- **Logging**: Print actual port used: `🚀 Ironpad running on http://localhost:3005`

### 9.7 File System Watching (NEW)
- **Decision**: Use `notify` crate with debouncing
- **Implementation**:
  ```rust
  use notify::{Watcher, RecursiveMode, Event};
  
  let (tx, rx) = channel();
  let mut watcher = notify::recommended_watcher(tx)?;
  watcher.watch(Path::new("data/"), RecursiveMode::Recursive)?;
  
  // Debounce: Collect events for 100ms, then broadcast via WebSocket
  ```
- **Events tracked**: Create, Modify, Delete
- **Ignored paths**: `.git/`, `node_modules/`, `.DS_Store`

### 9.8 Frontmatter Automation (NEW)
- **Decision**: Backend owns all frontmatter management
- **Implementation**:
  - On file creation: Generate frontmatter with `id`, `type`, `created`, `updated`
  - On file update: Parse YAML, update `updated` field, rewrite file
  - Use `gray_matter` or `serde_yaml` crates
- **User experience**: Users never manually edit timestamps or IDs

### 9.9 Markdown Consistency (NEW)
- **Decision**: Use CommonMark standard everywhere
- **Backend**: `markdown-rs` (CommonMark compliant)
- **Frontend**: `markdown-it` with CommonMark preset
- **Rationale**: Prevents rendering mismatches between preview and backend parsing

---

## 10. Build & Distribution

### Development Workflow

```bash
# Terminal 1: Frontend dev server with hot reload
cd frontend
npm install
npm run dev  # Runs on localhost:5173

# Terminal 2: Rust backend (proxies frontend)
cd backend
cargo run  # Runs on localhost:3000-3010, opens browser
```

### Production Build

```bash
# Step 1: Build frontend
cd frontend
npm run build  # Outputs to frontend/dist/

# Step 2: Copy frontend to Rust static folder
cp -r frontend/dist/* backend/static/

# Step 3: Build Rust release binary
cd backend
cargo build --release

# Output: backend/target/release/ironpad
# Size: ~5-15 MB
```

### Distribution
- **Single executable** - `ironpad.exe` (Windows), `ironpad` (Mac/Linux)
- **No installer required** - Just run the binary
- **No dependencies** - Statically linked (except libc)
- **User experience**:
  1. User downloads `ironpad.exe`
  2. User double-clicks executable
  3. Browser opens automatically to available port
  4. App is ready to use

---

## 11. Data Safety & Backup

### Git as Primary Backup
- Every save eventually commits to local Git repo
- Repo can be pushed to remote (GitHub, GitLab, self-hosted)
- All history preserved indefinitely

### Recommended Setup
```bash
# Initialize repo (done automatically on first run)
cd data
git init

# Add remote (optional, manual or via UI)
git remote add origin https://github.com/yourusername/ironpad-data.git

# Auto-push (future feature)
# UI button: "Push to Remote"
```

### Disaster Recovery
- Clone repo on new machine
- Point Ironpad to cloned folder
- All history and data preserved
- No proprietary formats to migrate

### Conflict Handling
- If `.git/index.lock` exists → Skip auto-commit, retry next cycle
- If `git status` shows conflicts → Show banner in UI: "Git conflicts detected. Resolve manually."
- Never auto-merge conflicts → User must resolve via git CLI or external tool

---

## 12. Success Criteria

The system is successful if:
- ✅ Used daily for notes and tasks
- ✅ Data remains readable without the app
- ✅ Adding new features does not require migrations
- ✅ System feels flexible rather than restrictive
- ✅ Binary size stays under 15 MB
- ✅ Startup time under 500ms
- ✅ Search completes in <100ms
- ✅ **External edits sync instantly** (<500ms latency)
- ✅ **No manual frontmatter editing required**
- ✅ **Never crashes due to port conflicts**

### Usage Metrics (Personal Tracking)
- Daily notes created per week (target: 5+)
- Tasks completed per week (target: 10+)
- Projects with active tasks (target: 2-3)
- Average note length (target: 200+ words)
- External edits per week (VS Code usage, target: tracked but not enforced)

### Performance Metrics
- App startup time (target: <500ms)
- Search response time (target: <100ms)
- Save operation time (target: <50ms)
- Binary size (target: <15 MB)
- File change notification latency (target: <500ms)
- WebSocket message latency (target: <100ms)

---

## 13. Implementation Phases

### Phase 1: MVP (Week 1-2)
**Goal**: Basic note CRUD + auto-open browser + dynamic port

- [ ] Rust backend with Axum
- [ ] Dynamic port selection (3000-3010)
- [ ] File CRUD operations (read/write .md files)
- [ ] Automatic frontmatter generation (id, created, updated)
- [ ] Auto-open browser on launch
- [ ] Vue frontend scaffold
- [ ] Basic markdown editor (textarea)
- [ ] File list sidebar
- [ ] Auto-save with debounce

### Phase 2: Core Features (Week 3-4)
**Goal**: Usable daily driver with real-time sync

- [ ] CodeMirror 6 integration
- [ ] Project creation/switching (with assets/ folders)
- [ ] Task parsing (checkboxes in markdown)
- [ ] Task view per project
- [ ] File locking (Task View vs Editor)
- [ ] Git init + auto-commit with lock handling
- [ ] Ripgrep-based search
- [ ] **File system watching** (notify crate)
- [ ] **WebSocket server** for real-time updates
- [ ] External edit notifications in UI

### Phase 3: Polish (Week 5-6)
**Goal**: Production-ready personal tool

- [ ] Split view (editor + preview)
- [ ] Manual commit with message
- [ ] Git status/history viewer
- [ ] Git conflict detection and UI warnings
- [ ] Tag extraction and filtering
- [ ] Daily note templates
- [ ] UI polish and keyboard shortcuts
- [ ] Asset upload for images

### Phase 4: Advanced (Future)
- [ ] Global hotkey (Ctrl+Shift+Space) using `global-hotkey` crate
- [ ] System tray icon (stays running in background)
- [ ] Tantivy full-text search (if ripgrep becomes slow)
- [ ] Backlinks between notes
- [ ] Remote Git push/pull from UI
- [ ] Export to PDF/HTML
- [ ] Custom themes

---

## 14. Project Structure

```
ironpad/
├── README.md
├── LICENSE
│
├── backend/                    # Rust backend
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   │   ├── main.rs            # Server startup + router + port detection
│   │   ├── websocket.rs       # WebSocket handler for real-time updates
│   │   ├── watcher.rs         # File system watcher (notify integration)
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── notes.rs       # /api/notes endpoints
│   │   │   ├── projects.rs    # /api/projects endpoints
│   │   │   ├── tasks.rs       # /api/projects/:id/tasks
│   │   │   ├── search.rs      # /api/search (ripgrep integration)
│   │   │   ├── git.rs         # /api/git endpoints
│   │   │   └── assets.rs      # /api/assets endpoints
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── filesystem.rs  # File read/write logic
│   │   │   ├── git.rs         # Git operations (git2) with lock handling
│   │   │   ├── search.rs      # Ripgrep search implementation
│   │   │   ├── markdown.rs    # Markdown parsing (CommonMark)
│   │   │   ├── frontmatter.rs # Auto-manage YAML frontmatter
│   │   │   └── locks.rs       # File lock management for concurrency
│   │   └── models/
│   │       ├── mod.rs
│   │       ├── note.rs        # Note struct
│   │       ├── project.rs     # Project struct
│   │       └── task.rs        # Task struct
│   └── static/                # Vue build output (in production)
│       └── (frontend dist files)
│
├── frontend/                   # Vue 3 frontend
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   ├── src/
│   │   ├── main.ts
│   │   ├── App.vue
│   │   ├── composables/
│   │   │   └── useWebSocket.ts  # WebSocket client composable
│   │   ├── components/
│   │   │   ├── Sidebar.vue
│   │   │   ├── ProjectList.vue
│   │   │   ├── NoteList.vue
│   │   │   ├── Editor.vue
│   │   │   ├── MarkdownPreview.vue
│   │   │   ├── TaskList.vue
│   │   │   ├── SearchBar.vue
│   │   │   ├── GitStatus.vue
│   │   │   ├── ExternalEditBanner.vue  # Shows when file changed externally
│   │   │   └── ReadOnlyBanner.vue      # Shows when file locked
│   │   ├── views/
│   │   │   ├── NotesView.vue      # Main notes editor
│   │   │   ├── TasksView.vue      # Project tasks view
│   │   │   └── ProjectView.vue    # Project overview
│   │   ├── stores/
│   │   │   ├── notes.ts           # Pinia store (minimal caching)
│   │   │   ├── projects.ts        # Pinia store for projects
│   │   │   ├── tasks.ts           # Pinia store for tasks
│   │   │   └── websocket.ts       # WebSocket state management
│   │   ├── api/
│   │   │   └── client.ts          # API client (fetch wrappers)
│   │   └── types/
│   │       └── index.ts           # TypeScript types
│   └── dist/                      # Build output (gitignored)
│
└── data/                          # User data (separate repo)
    ├── .git/
    ├── .gitignore
    ├── index.md
    ├── inbox.md
    ├── projects/
    │   └── ironpad/
    │       ├── index.md
    │       ├── tasks.md
    │       └── assets/            # Project-specific images
    ├── notes/
    │   ├── ideas.md
    │   └── assets/                # Shared note assets
    ├── daily/
    └── archive/
```

---

## 15. Dependencies

### Backend (Cargo.toml)
```toml
[package]
name = "ironpad"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.8", features = ["ws"] }  # WebSocket support
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["fs", "cors"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"  # Frontmatter parsing

# Markdown parsing (CommonMark)
markdown = "1.0.0-alpha.22"  # Frontmatter support

# Git operations
git2 = "0.19"

# Browser opening
webbrowser = "1.0"

# File system watching
notify = "6.1"
notify-debouncer-full = "0.3"  # Debounced file events

# Search (ripgrep as library)
grep = "0.3"  # ripgrep internals
walkdir = "2.4"

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Frontend (package.json)
```json
{
  "name": "ironpad-frontend",
  "version": "0.1.0",
  "dependencies": {
    "vue": "^3.5.0",
    "vue-router": "^4.5.0",
    "pinia": "^2.3.0",
    "codemirror": "^6.0.1",
    "@codemirror/lang-markdown": "^6.3.2",
    "markdown-it": "^14.1.0"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^5.2.1",
    "vite": "^6.0.5",
    "typescript": "^5.7.2"
  }
}
```

---

## 16. Open Questions & Decisions Tracking

| Question | Decision | Rationale | Date |
|----------|----------|-----------|------|
| Electron vs Rust backend? | **Rust backend** | Smaller binary, no bundled browser | 2026-02-02 |
| Auto-save strategy? | **2s debounce** | Balance safety and performance | 2026-02-02 |
| Git commit strategy? | **5min batch + manual** | Clean history, user control | 2026-02-02 |
| Editor library? | **CodeMirror 6** | Mature, performant, good UX | 2026-02-02 |
| Search in v1? | **Yes (ripgrep-based)** | Fast, proven, <100ms target | 2026-02-02 |
| Tasks per project or global? | **Per project** | Cleaner mental model | 2026-02-02 |
| Mobile access? | **Not v1 priority** | Desktop-first | 2026-02-02 |
| Port conflict handling? | **Dynamic 3000-3010** | Always works, graceful fallback | 2026-02-02 |
| External edit support? | **Yes (notify + WebSocket)** | True local-first philosophy | 2026-02-02 |
| Frontmatter management? | **Auto-managed by backend** | Low ceremony, no manual IDs | 2026-02-02 |
| Task View vs Editor conflict? | **File locking** | Prevent race conditions | 2026-02-02 |
| Markdown standard? | **CommonMark** | Consistency backend/frontend | 2026-02-02 |

---

## 17. Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Rust learning curve | High | Medium | Start simple, use Axum (easier than Actix) |
| Git conflicts (concurrent edits) | Low | Medium | Detect `.git/index.lock`, show UI warnings |
| Search performance at scale | Low | Medium | Ripgrep handles 5000+ files easily |
| Browser doesn't auto-open | Low | Low | Print URL as fallback |
| File corruption | Low | High | Git versioning protects against data loss |
| **WebSocket connection drops** | Medium | Medium | Auto-reconnect in frontend with exponential backoff |
| **File watcher overhead** | Low | Low | Debounce events (100ms), ignore .git/ |
| **Port conflicts** | Low | Low | Dynamic port selection 3000-3010 |
| **Race condition (Task View + Editor)** | Medium | High | File locking prevents simultaneous edits |
| **Markdown rendering mismatch** | Low | Medium | Use CommonMark everywhere |

---

## 18. Future Enhancements (Out of Scope)

### Potential v2+ Features
- **Global hotkey** - Ctrl+Shift+Space to bring app to front (using `global-hotkey` crate)
- **System tray icon** - Keep app running in background (using `tray-icon` crate)
- **Backlinks** - Automatic link detection between notes
- **Graph view** - Visual representation of note connections
- **Rich editor** - WYSIWYG markdown editor
- **Templates** - Note templates (daily, meeting, project)
- **Plugins** - Extension system for custom functionality
- **Sync** - Optional cloud sync via Git remote
- **Themes** - Dark mode, custom color schemes
- **Export** - PDF, HTML, DOCX export
- **Mobile web UI** - Responsive design for mobile browsers
- **Kanban view** - Visual task board per project
- **Time tracking** - Track time spent on tasks
- **Voice notes** - Audio recording integration
- **OCR** - Extract text from images

---

## 19. Addressing Gemini's Feedback

### ✅ 1. Task Syncing Race Conditions
**Issue**: Task View checkboxes vs Editor text edits conflict  
**Solution**: File locking system
- Task View locks `tasks.md` when open
- Editor shows "Read-Only" banner if file locked
- Only one view can edit at a time

### ✅ 2. File System Watching
**Issue**: External edits (VS Code, Obsidian) don't sync  
**Solution**: `notify` crate + WebSocket
- Backend watches `data/` directory
- Sends real-time updates to frontend
- UI shows "File changed externally. Reload?" banner

### ✅ 3. Git Conflict Handling
**Issue**: `.git/index.lock` can cause crashes  
**Solution**: Graceful lock detection
- Check for lock file before committing
- Skip auto-commit if locked, retry next cycle
- Show UI warning if git conflicts detected

### ✅ 4. Frontmatter Management
**Issue**: Manual timestamp/ID editing is high-friction  
**Solution**: Backend owns all frontmatter
- Auto-generate `id` from filename
- Auto-update `updated` on every save
- Users never manually edit metadata

### ✅ 5. Port Conflicts
**Issue**: Hardcoded :3000 breaks if port busy  
**Solution**: Dynamic port selection
- Try ports 3000-3010
- Bind to first available
- Log actual port used

### ✅ 6. Search Performance
**Issue**: Naive grep slow at >500 files  
**Solution**: Use `ripgrep` library
- Battle-tested, used by VS Code
- Handles 5000+ files easily
- <100ms target achieved

### ✅ 7. Markdown Consistency
**Issue**: Backend parsing vs frontend rendering mismatch  
**Solution**: CommonMark everywhere
- Backend: `markdown-rs` (CommonMark mode)
- Frontend: `markdown-it` (CommonMark preset)
- Guaranteed consistency

### ✅ 8. State Management
**Issue**: Pinia caching vs file system truth  
**Solution**: Minimal caching philosophy
- File system is source of truth
- Pinia only caches current view
- WebSocket invalidates cache on external changes

### ✅ 9. Asset Management
**Issue**: No image/file storage  
**Solution**: `assets/` folders
- Each project gets `assets/` subfolder
- Global `notes/assets/` for shared files
- Upload via `/api/assets/upload`

---

## 20. Conclusion

**Ironpad v3.0** represents a robust, production-ready architecture:
- **Local-first** with true external editor support
- **Lightweight** Rust backend (no browser bundling)
- **Real-time** sync via WebSocket
- **Conflict-free** via file locking
- **Low-ceremony** via automatic frontmatter management
- **Resilient** via git lock handling and dynamic ports

The system is designed to be:
- ✅ Easy to use daily
- ✅ Easy to understand and modify
- ✅ Easy to back up and migrate
- ✅ Fast and responsive
- ✅ A practical Rust learning project
- ✅ **Robust against real-world edge cases**

**Next Step**: Begin Phase 1 implementation - Rust backend with dynamic port selection + automatic frontmatter.

---

**Document Version History**
- v1.0 (2026-02-01): Initial draft with general architecture
- v2.0 (2026-02-02): Complete rewrite with Rust backend, browser-based frontend, detailed technical decisions
- v3.0 (2026-02-02): Addressed concurrency, file watching, git conflicts, port handling, frontmatter automation, and Gemini's architectural feedback

**Contact**: Internal project - personal use