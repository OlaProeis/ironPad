# Ironpad Roadmap

## Release 0.2.0 (Next)

### Planned Features

#### 1. Task comments & activity summary
- **Comment section** per task with date-stamped entries
- Store comments (e.g. in task file as YAML sequence or dedicated section)
- **Last comment as summary** — show the most recent comment/activity in the main task list and dashboard to indicate last action or status
- Enables quick status updates without editing the full description

#### 2. Recurring tasks on the calendar
- **Bug/feature gap**: Tasks with daily/weekly recurrence but no explicit `due_date` currently do not appear on the calendar (calendar only shows tasks with `task.due_date`)
- **Change**: Expand recurring tasks into the calendar for the visible month:
  - **Daily** — show on every day in the month (or cap at reasonable limit)
  - **Weekly** — show on the matching weekday(s) in the month
  - **Monthly** — show on the day-of-month if set, else treat as “floating”
- Requires frontend logic to compute occurrences from `recurrence`, `recurrence_interval`, and optionally `due_date` / `created`

---

## Suggested features (future releases)

Ideas that fit the current architecture and local-first design:

### High fit (0.3.x)
- **Calendar drag-and-drop** — reschedule tasks by dragging onto a new date (already listed in ai-context)
- **Week / day calendar views** — alternative to month view for denser task planning
- **Sort task list by due date / priority** — alongside current created-date sorting
- **Overdue indicator** — clearer overdue badge or count in sidebar and dashboard

### Medium fit (0.4.x)
- **Quick-add task** — global or dashboard shortcut to create a task without opening a project
- **Bulk actions** — complete multiple tasks, move section, add/remove tags in one go
- **Task templates** — create tasks from predefined templates (e.g. “Meeting prep”, “Review”)
- **Tag extraction and cross-project filtering** — surface and filter by tags across all projects

### Longer term (Phase 6+)
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

| Version | Status  | Notes                                              |
|---------|---------|----------------------------------------------------|
| 0.1.0   | Current | First public release, core features in place       |
| 0.2.0   | Planned | Comments, recurring tasks on calendar              |
