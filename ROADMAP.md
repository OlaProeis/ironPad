# Ironpad Roadmap

## Release 0.2.0 (Current)

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
- **Calendar drag-and-drop** -- reschedule tasks by dragging onto a new date
- **Week / day calendar views** -- alternative to month view for denser task planning
- **Sort task list by due date / priority** -- alongside current created-date sorting
- **Overdue indicator** -- clearer overdue badge or count in sidebar and dashboard

### Medium fit (0.4.x)
- **Quick-add task** -- global or dashboard shortcut to create a task without opening a project
- **Bulk actions** -- complete multiple tasks, move section, add/remove tags in one go
- **Task templates** -- create tasks from predefined templates (e.g. "Meeting prep", "Review")
- **Tag extraction and cross-project filtering** -- surface and filter by tags across all projects

### Longer term
- UI polish and subtle animations
- Responsive sidebar / mobile-friendly layout
- Global hotkey (e.g. Ctrl+Shift+Space)
- Backlinks between notes
- Graph view of note connections
- Export to PDF / HTML
- Custom themes
- Kanban board view

---

## Version history

| Version | Status   | Date       | Notes                                                    |
|---------|----------|------------|----------------------------------------------------------|
| 0.1.0   | Released | 2025-12-01 | First public release, core features in place             |
| 0.2.0   | Current  | 2026-02-16 | Comments, recurring calendar, system tray, app branding  |
