# Changelog

All notable changes to Ironpad are documented here.

## [0.3.0] - 2026-02-27

### Added
- **Prompt Manager** with global and project-scoped prompt libraries.
- **Prompt storage model** as markdown + YAML frontmatter:
  - Global prompts in `data/prompts/`
  - Project prompts in `data/projects/{id}/prompts/`
- **Prompt API surface**:
  - `GET/POST /api/prompts`
  - `GET/PUT/DEL /api/prompts/:id`
  - `GET /api/prompts/folders`
  - `GET /api/prompts/search/semantic`
- **Prompt variables** using `{{variable_name}}` placeholders.
- **Copy rendered** action that applies variable substitutions before copying.
- **Prompt-level version history** using Git commit history filtered by prompt file path (`GET /api/git/log/file`).
- **Calendar drag-and-drop** -- reschedule tasks by dragging them to a new date.
- **Week and day calendar views** -- alternative to month view for denser task planning and single-day focus.
- **Backlinks between notes** -- `/link` slash command to insert markdown links to other project notes:
  - Type `/link` in the editor to open a searchable note picker dropdown.
  - Inserts standard markdown `[Note Title](note-id)` links.
  - **BacklinksPanel** below the editor shows forward links ("Links To") and backlinks ("Linked From").
  - Clicking a link in the panel navigates to the target note.
  - Backend link index rebuilt on save and file watcher events.
  - **Backlinks API surface**:
    - `GET /api/backlinks/notes/:id/links` (both directions)
    - `GET /api/backlinks/notes/:id/backlinks`
    - `GET /api/backlinks/notes/:id/forward-links`
    - `POST /api/backlinks/links/rebuild`
    - `GET /api/backlinks/links/stats`
  - **Project notes title endpoints** for link autocomplete:
    - `GET /api/projects/:id/notes-titles`
    - `GET /api/projects/:id/notes-search`

### Changed
- Top navigation now includes **Prompts** for quick global access.
- Project sidebar navigation now includes **Prompts**.
- Prompt library UI now includes collapsible filter controls to save space.
- Prompt input and editor fields now consistently respect dark/light theme styling.
- Calendar view now has view mode toggle (Month/Week/Day) with appropriate navigation for each mode.
- Milkdown editor core now stores a raw (non-reactive) Crepe reference to avoid Vue proxy issues with private fields.
- Editor supports force-remount fallback when programmatic content updates fail.

---

## [0.2.0] - 2026-02-16

### Added
- **Task comments** -- date-stamped comment entries per task, stored as YAML in frontmatter. Last comment shown as summary in task list and dashboard cards. Add/delete via API and UI.
- **Recurring tasks on calendar** -- tasks with daily/weekly/monthly/yearly recurrence now appear on the calendar even without an explicit `due_date`. Occurrences are computed from the anchor date (`due_date` or `created`) and `recurrence_interval`. Recurring entries show with a dashed border and recurrence icon to distinguish from regular due-date tasks.
- **System tray mode** -- production binary runs in the system tray instead of a console window. Tray menu with "Open in Browser" and "Quit". No CMD window on Windows in release builds (`windows_subsystem = "windows"`). Server runs on a background thread with the tray event loop on the main thread for cross-platform safety.
- **App icon and branding** -- Ironpad logo embedded in the Windows executable (Explorer icon + tray icon) via `winresource`. Favicon and logo added to the web UI (browser tab + top bar header).
- **Local build script** -- `build-local.ps1` for building a testable release package locally.

### Changed
- Backend `main.rs` restructured for dual-mode operation: development mode runs the server directly (no tray), production mode runs server on background thread with tray on main thread.
- Calendar view refactored to use `CalendarEntry` interface that merges regular due-date tasks with computed recurring occurrences.

### Dependencies
- Added `tray-item = "0.10"` for cross-platform system tray support.
- Added `windows-sys = "0.52"` (Windows only) for loading the embedded icon resource.
- Added `winresource = "0.1"` (Windows build dependency) for embedding the icon in the .exe.

---

## [0.1.0] - 2025-12-01

### Added
- Initial release of Ironpad -- local-first, file-based project and knowledge management.
- **Backend**: Rust/Axum API server with dynamic port (3000-3010), WebSocket real-time sync, file watcher, Git auto-commit (60s batching), ripgrep search.
- **Frontend**: Vue 3 SPA with Milkdown WYSIWYG editor, dark/light theme, Pinia state management.
- **File-based tasks**: each task stored as a markdown file with YAML frontmatter (title, completed, section, priority, due_date, tags, subtasks, recurrence).
- **Split-panel task view**: task list with active/backlog/completed sections, detail editor with markdown, due date picker, tag system, subtasks, recurrence picker.
- **Calendar view**: month grid showing tasks by due date with color-coded urgency and daily note indicators.
- **Dashboard**: cross-project home page with active task summaries per project.
- **Daily notes**: date-based notes with templates.
- **Git panel**: commit history with diffs, working directory changes, push/fetch with ahead/behind indicators.
- **Project notes**: split-panel notes view per project.
- **Search**: Ctrl+K search panel with ripgrep-powered full-text search.
- Cross-platform builds (Windows, macOS, Linux) via GitHub Actions.
