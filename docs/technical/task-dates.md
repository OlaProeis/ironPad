# Task due dates and overdue logic (frontend)

Ironpad stores task due dates as calendar strings (`YYYY-MM-DD` in task frontmatter). Overdue UI must compare **local calendar days**, not UTC instants from `new Date("YYYY-MM-DD")` (which can mark “due today” as overdue).

## Module

`frontend/src/lib/taskDates.ts`

| Export | Role |
|--------|------|
| `parseDueDateOnly(dueDate)` | Validates and normalizes a `YYYY-MM-DD` string; returns `null` if missing or invalid. |
| `formatLocalCalendarDate(date)` | Builds `YYYY-MM-DD` from a local `Date` (year/month/day parts). |
| `isTaskOverdue(task, todayLocal)` | Shared overdue rule for incomplete tasks (see below). |

## Overdue rule

`isTaskOverdue` returns `true` only when all of the following hold:

- Task is not `completed`
- `due_date` is present and parses as a valid calendar date
- Due calendar date is **strictly before** the calendar day of `todayLocal` (local timezone)

Tasks due **today** are not overdue. Completed tasks and bad/missing dates are never overdue.

Comparison uses normalized `YYYY-MM-DD` strings (lexicographic order), not `Date` parsing of the due string alone.

Pass an explicit `todayLocal` in tests (and eventually in UI) so behavior is stable across time-of-day and timezone.

## Dashboard project cards

`frontend/src/lib/dashboardProjectStats.ts` exports `countActiveOverdueTasks(tasks, todayLocal)`:

- Considers only **active** (`is_active`) tasks that are not completed
- Counts tasks where `isTaskOverdue` is true (stored `due_date` only; recurrence is not expanded into extra overdue rows)

`DashboardView.vue` uses `countActiveOverdueTasks(projectTasks, new Date())` for each card’s `overdueCount` stat.

## Sidebar quick stats

`frontend/src/components/Sidebar.vue` shows an **Overdue** quick-stat tile for the active project when `overdueCount > 0`.

- Source: `countActiveOverdueTasks(tasksStore.tasks, new Date())` — same helper and rules as the dashboard (active, incomplete tasks with a stored `due_date`).
- Tasks load via the existing `watch(activeProjectId)` → `tasksStore.loadProjectTasks` path; no extra API calls.
- The tile is hidden when the count is zero.
- Click navigates with `goToProjectTasks` (`project-tasks` route), same as the Active Tasks / Completed stats.

## Tests

Vitest unit tests:

| File | Coverage |
|------|----------|
| `frontend/src/lib/taskDates.test.ts` | `isTaskOverdue` calendar rules |
| `frontend/src/lib/dashboardProjectStats.test.ts` | Dashboard overdue count aggregation |

From `frontend/`:

```bash
npm run test:unit -- src/lib/taskDates.test.ts src/lib/dashboardProjectStats.test.ts
```

Vitest is configured in `frontend/vite.config.ts` (`test.environment: node`, `src/**/*.test.ts`).

## Wiring status (v0.4.0)

| Surface | Uses shared rule? |
|---------|------------------|
| Dashboard project cards | Yes — `countActiveOverdueTasks` |
| Sidebar quick stats (active project) | Yes — `countActiveOverdueTasks` on `tasksStore.tasks` |
| Task list / calendar inline labels | Not yet — still use legacy `Date` diff helpers in views |

## Related product spec

See [PRD 0.4.0 — Overdue count](../ai-workflow/PRD-0.4.0.md) (section 5) for the single date rule shared by dashboard cards and sidebar.