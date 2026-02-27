# Ironpad Roadmap

## Release 0.3.0 (Current)

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

### High fit (0.3.x)
- [x] **Calendar drag-and-drop** -- reschedule tasks by dragging onto a new date
- [x] **Week / day calendar views** -- alternative to month view for denser task planning
- [ ] **Sort task list by due date / priority** -- alongside current created-date sorting
- [ ] **Overdue indicator** -- clearer overdue badge or count in sidebar and dashboard

### Medium fit (0.4.x)
- **Quick-add task** -- global or dashboard shortcut to create a task without opening a project
- **Bulk actions** -- complete multiple tasks, move section, add/remove tags in one go
- **Task templates** -- create tasks from predefined templates (e.g. "Meeting prep", "Review")
- **Tag extraction and cross-project filtering** -- surface and filter by tags across all projects

### Longer term
- UI polish and subtle animations
- Responsive sidebar / mobile-friendly layout
- Global hotkey (e.g. Ctrl+Shift+Space)
- Graph view of note connections
- Export to PDF / HTML
- Custom themes
- Kanban board view

---

## Version history

| Version | Status   | Date       | Notes                                                               |
|---------|----------|------------|---------------------------------------------------------------------|
| 0.1.0   | Released | 2025-12-01 | First public release, core features in place                        |
| 0.2.0   | Released | 2026-02-16 | Comments, recurring calendar, system tray, app branding             |
| 0.3.0   | Current  | 2026-02-27 | Prompt Manager, calendar improvements, backlinks, Git history |
