# Changelog

All notable changes to Ironpad are documented here.

## [Unreleased]

## [0.4.0] - 2026-11-20

### Added
- **Task list sort** — Project task view (`TasksView`) can sort top-level tasks by **created** (newest first, default), **due date** (earliest first; no due date last), or **priority** (high → medium → low → unset). The same mode applies in Active, Backlog, and Completed. **Subtasks stay under their parent** in existing order. The choice persists in `localStorage` and survives reload.
- **Sidebar overdue count** — Quick stats on the sidebar show an **overdue** figure for the **active project** when it is greater than zero (click opens that project’s task list). A task due **today** is not overdue. The count uses the same calendar-day rule as dashboard project cards, so **sidebar and dashboard agree** for the same project.
- **Dashboard filter by task tag** — Tag chips across the top list each distinct task tag (A–Z) with incomplete-task counts. One tag at a time filters every project card to incomplete tasks with that tag; clearing the active chip **restores the full dashboard**. Empty filter state names the tag and offers clear.
- **Fit the day (calendar day view)** — Day view shows **used / budget** minutes and a progress bar from task estimates. Default budget is **6 hours**; the user can change it (1–24 hours) in day view and the value is remembered in `localStorage`. The bar turns **amber** when used exceeds budget. Month and week views keep their existing total label only.
- **Bulk task actions** — Multi-select on **top-level** tasks in the project task view (subtasks are not selectable). Actions: mark complete, move to Active, move to Backlog, add tag, remove tag. **Mark complete** skips already-completed tasks and still runs **recurring** spawn behavior via the existing toggle path. **Move** and **tag** updates keep `completed` and `section` consistent (e.g. moving a completed task to Active or Backlog sets `completed` false). Requests run **sequentially**; the first failure stops with task title and error, earlier saves are kept, then the list reloads.
- **Private Git backup in the app**:
  - Save a private **HTTPS or SSH** URL as `origin` from the Git panel (`POST /api/git/remote`). **URLs with embedded credentials are rejected**.
  - **Pull** (`POST /api/git/pull`) fast-forwards from upstream; **diverged histories change no files**. Fetch remains separate.
  - **Push** sets upstream on first success so **ahead/behind** reflect the remote afterward.
  - Actionable UI copy for auth failure, non-fast-forward push, non-fast-forward pull, dirty tree blocking pull, missing remote, and unreachable host (per product error map).
  - [`docs/git-backup.md`](docs/git-backup.md) linked from the Git panel — **async backup**, not live co-editing.
- **Library** (Prompts screen; routes and `/api/prompts` unchanged):
  - UI label **Library** in the top bar and project sidebar; storage stays `data/prompts/` and project prompt folders.
  - **Link** pointers (`kind: link`, `source`, optional `template`) read files on disk at open/reload; Save updates the pointer only.
  - Project links use a folder root in browser `localStorage`; global links use absolute paths.
  - `GET /api/prompts/:id?root=` returns `read_only`, `resolved_path`, `source_missing`, `root_required`, `source_error`, and `supports_rendered` for links.
  - `GET /api/library/browse?path=` for non-recursive directory pick; `POST /api/library/open` opens a link by id with the OS default app.
  - Raw and rendered reading modes, collapsible Versions (Git), copy bundle, suggested `ai-context.md` / `ROADMAP.md` / `docs/index.md`, Add link browse/paste, template links with `{{variables}}`. Spec: [`docs/ai-workflow/PRD-0.4.0-library.md`](docs/ai-workflow/PRD-0.4.0-library.md).
  - A prompt with no `kind` still opens, edits, and saves as before. Adding a link writes a pointer only and does not copy the source into `data/`. Delete removes the pointer and leaves the file. A missing file stays in the list and can be relinked. Files over **1 MiB**, directories, and suffixes outside the allowed list are refused.
  - **Raw / Rendered** is remembered per item. Markdown links open on rendered; other allowed files stay raw. Saving a prompt writes the raw draft, including when the pane is on rendered. `{{variables}}` are substituted only for prompts and for links with **Template** turned on, and only in memory.
  - **Versions (Git)** starts collapsed and remembers that choice. An expanded prompt still shows that file’s Git log. An expanded link does not list Ironpad commits as the linked file’s history.
  - **Copy bundle** puts the selected prompts (raw body) and links (current source text) in one markdown blob, with no variable substitution. **Open externally** and **Copy path** use the resolved source. A missing file does not open.
- **GitHub link** (separate from Remote backup of `data/`):
  - Link **one repository per project** as `owner/name` on github.com. A URL, a `.git` suffix, or an empty value is rejected. Saving the link does not change the notes repo’s `origin`. Remove deletes the link and the snapshot and leaves a task created earlier in place.
  - **GitHub** sidebar view with open **issues** and **pull requests** in separate lists. A pull request is not shown as an issue. Issue filters: **All open** (default), **Assigned to me**, **Created by me**. Pull request filters: **All open**, **Mine**, **Review requested**. The choice survives reload.
  - Refresh when the snapshot is missing or older than 15 minutes, from manual **Refresh**, and on a 15-minute timer while the view is open. An unchanged poll keeps the current rows and updates the shown time. A failed refresh keeps the previous rows. The snapshot directory is outside `data/`.
  - Read item **body** and up to **30 comments** (and PR **commit status**) without writing to GitHub.
  - **Track as task** copies an issue or PR into one local Active task (title, labels as tags, milestone date when present, `github_url`, `github_number`). Tracking the same URL again opens that task. Completing the task does not close the GitHub item and does not call GitHub.
  - Authentication via **`gh auth token`** or **`GH_TOKEN`** / **`GITHUB_TOKEN`**. There is no token field, and no token stored in `data/`, API responses, or logs. With none of the three, the view asks for `gh auth login` or `GH_TOKEN`.
  - **Dashboard** project cards show assigned-issue and review-request counts from the last snapshot, and hide that line when both are zero.
  - User guide: [`docs/github-link.md`](docs/github-link.md). Spec: [`docs/ai-workflow/PRD-0.4.0-github.md`](docs/ai-workflow/PRD-0.4.0-github.md).

### Changed
- **Prompts → Library** — navigation and headings say Library; API paths and prompt file layout are unchanged.
- **Backend listen port** — Default scan range is **37891–37990** instead of 3000–3010, to avoid collisions with Grafana, React dev servers, and other stacks. Optional **`IRONPAD_PORT`** binds a single port (fails fast if busy). Startup logs the URL to open. Vite dev proxy defaults to **37891**; set `IRONPAD_PORT` when running `npm run dev` if the backend uses another port in the range.
- **Task meta API** — `UpdateTaskMetaRequest` accepts optional `completed` so bulk section moves can clear completed without using toggle (toggle remains the path that spawns the next recurring instance).

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
