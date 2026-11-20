# Ironpad Roadmap

## Release 0.3.0 (Released)

### Features

#### 1. Prompt Manager (global + project)
- Dedicated **Prompts** section in project navigation
- Top-right quick access button for **Global Prompts**
- File-based storage:
  - `data/prompts/` for global prompts
  - `data/projects/{id}/prompts/` for project prompts
- Prompt metadata via frontmatter (folder, tags, scope, description)

#### 2. Prompt library UX
- Folder and tag filtering in the library panel
- Collapsible filters to reduce sidebar vertical space
- Prompt CRUD + duplicate actions
- Prompt-level Git history and diff browsing

#### 3. Prompt variables and rendered copy
- Placeholder variables via `{{variable_name}}`
- Auto-detected variable input form
- **Copy raw** and **Copy rendered** actions
- Live rendered preview panel

#### 4. Calendar improvements
- **Drag-and-drop rescheduling** -- drag tasks to new dates to change due dates
- **Week view** -- 7-day horizontal layout for denser task planning
- **Day view** -- single date focus with expanded task list
- View mode toggle (Month/Week/Day) with appropriate navigation for each mode

#### 5. Backlinks between notes
- `/link` slash command triggers searchable note picker dropdown
- Inserts standard markdown links: `[Note Title](note-id)`
- **BacklinksPanel** displays forward links and incoming backlinks per note
- Click-to-navigate from link panel to target note
- Backend link index (in-memory, rebuilt on save and file watcher events)
- Backlinks API (`/api/backlinks/notes/:id/links`, rebuild, stats)
- Project-scoped note title endpoints for autocomplete

#### 6. Quick-add task
- Global floating action to create a task from any view (**Ctrl+Shift+A**)
- Creates a task without navigating into a project first (per product behavior in app)

---

## Release 0.4.0 (Released)

Released 2026-11-20.

Product requirements:

- Task list, dashboard, and Git backup: [`docs/ai-workflow/PRD-0.4.0.md`](docs/ai-workflow/PRD-0.4.0.md)
- Library: [`docs/ai-workflow/PRD-0.4.0-library.md`](docs/ai-workflow/PRD-0.4.0-library.md)
- GitHub issues and pull requests: [`docs/ai-workflow/PRD-0.4.0-github.md`](docs/ai-workflow/PRD-0.4.0-github.md)

### Server port

- **Server port** — default scan **37891–37990**, optional **`IRONPAD_PORT`**. No config-file pin in this release.

### Features

1. **Sort the task list** — created (default), due date, or priority. Top-level tasks only. Choice kept in the browser.
2. **Overdue count in the sidebar** — active project, same local calendar-day rule as the dashboard. Due today is not overdue.
3. **Dashboard filter by task tag** — one task-frontmatter tag across projects. Note-body tags are out.
4. **Fit the day** — day view capacity bar from `estimated_minutes`, default budget 6 hours, amber when over.
5. **Bulk actions** — on the open project: complete, move to Active or Backlog, add or remove a tag. Existing per-task endpoints. Subtasks are not selected.
6. **Private Git remote, finished in the app** — save a private `origin` URL, fast-forward pull, push sets upstream, readable errors for auth and divergence. Short doc at `docs/git-backup.md`. No hosted “create repo”, no merge, no tokens stored.
7. **Library** — Prompts screen renamed in the UI. Routes and `/api/prompts` stay. Spec: [`docs/ai-workflow/PRD-0.4.0-library.md`](docs/ai-workflow/PRD-0.4.0-library.md).
   - **Library label** — top bar and project sidebar say Library.
   - **Linked files** — `kind: link` pointers under the existing prompt folders. A project link is relative to a folder root kept in the browser. The pane is read-only; Save updates the pointer. A missing file stays in the list and can be relinked.
   - **Rendered or raw** — remembered per item. Markdown can render. Other allowed files stay raw. `{{variables}}` apply to prompts and to links with Template turned on.
   - **Versions (Git)** — collapsed by default. An expanded prompt still shows that file’s history. An expanded link does not list Ironpad commits as that file’s history.
   - **Copy bundle and suggest files** — one clipboard blob for the selection, with no variable substitution. With a project root set, offer `ai-context.md`, `ROADMAP.md`, and `docs/index.md`. **Open externally** and **Copy path** use the resolved file.
8. **GitHub link** — separate from Remote backup of `data/`. Spec: [`docs/ai-workflow/PRD-0.4.0-github.md`](docs/ai-workflow/PRD-0.4.0-github.md).
   - **Link** — one `owner/name` per project on github.com. Login is `gh auth login` or `GH_TOKEN`. Ironpad does not store a token. Saving the link does not change `origin`.
   - **Lists** — open issues and pull requests, with filters remembered in the browser. Refresh on open when the snapshot is stale, from **Refresh**, and every 15 minutes while the view is open.
   - **Read and track** — body, up to 30 comments, and pull-request commit status, read-only. **Track as task** copies the item into a local Active task (labels as tags, milestone as due date). Tracking the same URL again opens that task. Completing it does not call GitHub.
   - **Dashboard** — project cards show assigned issues and review requests from the last snapshot, and hide the line when both are zero.

Task templates stay unscheduled. There is no 0.4 PRD left to implement.

---

## Release 0.5.0 (Proposed)

Next release. No PRD has been written for it yet.

Use what 0.3 already stores — prompts, Git history, and daily notes — before adding another generic view. Fit the day, the Library, and the GitHub link are part of 0.4.0 and are already in the working tree.

Backlinks shipped in 0.3.0. Graph view stays a later follow-on and should read the existing link index.

### Features

#### 1. Run a prompt on a note or task
- From a note or a task, pick a prompt and fill `{{title}}`, `{{due_date}}`, `{{content}}`, `{{last_comment}}`, and other frontmatter fields from that file
- **Copy rendered** stays as it works today
- Optional **Run locally** posts to a user-configured localhost endpoint (Ollama or any other local server) and writes the reply back as a task comment or a new note
- No API key and no cloud call. If nothing is listening locally, Run locally is unavailable and copy rendered still works

#### 2. Weekly review file
- **Write this week** creates `data/reviews/YYYY-Www.md`
- Built from Git history plus tasks completed, notes touched, prompts changed, projects that went quiet, and overdue tasks still open
- The review is a normal Markdown file and commits with everything else
- Each review links to the previous one

#### 3. Waiting across projects
- `waiting_on` on a task: a person's name, or the id of another task
- Dashboard line for work that is blocked, including across projects
- Completing a blocker task surfaces the tasks that were waiting on it

#### 4. Capture without opening the app
- **Ctrl+Shift+Space** opens a one-line capture strip
- Enter appends a timestamped bullet to today's daily note, or to `inbox.md` when today has no daily note
- The browser stays in the background
- This replaces the earlier "hotkey brings Ironpad forward" idea

#### 5. Project blueprints
- A blueprint is a folder of notes, tasks, and prompts
- **New project from blueprint** copies that folder and mints fresh ids
- Same file model, no new storage

---

## Release 0.2.0 (Released)

### Features

#### 1. Task comments & activity summary
- **Comment section** per task with date-stamped entries
- Store comments as YAML sequence in task frontmatter
- **Last comment as summary** -- most recent comment shown in task list and dashboard cards
- Add/delete comments via API and UI, newest-first display with relative timestamps

#### 2. Recurring tasks on the calendar
- Tasks with daily/weekly recurrence now appear on the calendar (previously required explicit `due_date`)
- Recurring tasks expanded into the visible month grid (daily/weekly/monthly/yearly)
- Anchor date: `due_date` if set, otherwise `created`; respects `recurrence_interval`
- Recurring occurrences shown with dashed border and recurrence icon

#### 3. System tray mode
- System tray icon replaces CMD window (Windows, macOS, Linux)
- Tray menu: **Open in Browser** | **Quit**
- No console window on Windows in release builds
- Server runs on background thread; tray event loop on main thread (cross-platform safe)

#### 4. App branding
- Ironpad logo as system tray icon and Windows exe icon
- Favicon and logo in the web UI (browser tab + header)

---

## Suggested features (future releases)

Ideas that fit the current architecture and local-first design:

### Shipped in 0.4.0

- **Sort the task list** by created, due date, or priority
- **Overdue count in the sidebar**
- **Dashboard filter by task tag** across projects
- **Fit the day** (day view capacity bar)
- **Bulk actions** on the open project
- **Private Git remote, finished in the app** (URL, fast-forward pull, readable errors)
- **Library** — prompts plus read-only links to real files, rendered or raw, collapsible Git versions, copy bundle, suggested project files, open externally ([`PRD-0.4.0-library.md`](docs/ai-workflow/PRD-0.4.0-library.md))
- **GitHub link** — one `owner/name` per project, read-only open issues and pull requests, track as a local task, dashboard assigned and review counts, no token stored ([`PRD-0.4.0-github.md`](docs/ai-workflow/PRD-0.4.0-github.md))

### Still unscheduled
- **Task templates** -- create tasks from predefined templates (e.g. "Meeting prep", "Review")
- **Note-body tag extraction** -- the 0.4 filter uses task frontmatter tags only

### Longer term
- UI polish and subtle animations
- Responsive sidebar / mobile-friendly layout
- Graph view of note connections (reads the backlink index shipped in 0.3.0)
- Export to PDF / HTML
- Custom themes
- Kanban board view

---

## Version history

| Version | Status   | Date       | Notes                                                               |
|---------|----------|------------|---------------------------------------------------------------------|
| 0.1.0   | Released | 2025-12-01 | First public release, core features in place                        |
| 0.2.0   | Released | 2026-02-16 | Comments, recurring calendar, system tray, app branding             |
| 0.3.0   | Released | 2026-02-27 | Prompt Manager, calendar improvements, backlinks, quick-add, Git history |
| 0.4.0   | Released | 2026-11-20 | Task list, dashboard, Git backup, Library, and GitHub link |
| 0.5.0   | Proposed | —          | No PRD yet. Runnable prompts, weekly review, waiting, capture, blueprints |
