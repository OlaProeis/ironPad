# Task list sort (frontend)

Release 0.4.0 adds client-side ordering for top-level tasks in the project task view. Sort rules live in `frontend/src/lib/taskSort.ts`; `TasksView` applies them per section.

## Module

`frontend/src/lib/taskSort.ts`

| Export | Role |
|--------|------|
| `TaskSortMode` | `'created' \| 'due_date' \| 'priority'` |
| `sortTopLevelTasks(tasks, mode)` | Returns a new sorted array; does not mutate input |
| `readSortMode()` | Reads persisted mode from `localStorage` |
| `writeSortMode(mode)` | Persists mode to `localStorage` |

## Sort rules

All modes apply to **top-level tasks only** (subtask order is unchanged).

| Mode | Order | Tie-break |
|------|--------|-----------|
| **created** | Newest `created` first (ISO string `localeCompare`) | — |
| **due_date** | Earliest valid `YYYY-MM-DD` first | Newest `created` |
| **priority** | `high` → `medium` → `low` → unset | Newest `created` |

**Due date:** Uses `parseDueDateOnly` from `task-dates.md`. Missing `due_date`, empty values, and unparseable strings sort **last** (same bucket; tie-break by `created` desc).

**Priority:** Case-insensitive. Any value other than high/medium/low sorts as **unset**.

## TasksView UI

`frontend/src/views/TasksView.vue`:

- Sort bar (dropdown) in the list panel, below the header and above tag filters.
- Modes: **Created**, **Due date**, **Priority** — maps to `TaskSortMode`.
- On mount: `readSortMode()`; on change: `writeSortMode(mode)`.
- `filteredActiveTasks`, `filteredBacklogTasks`, and `filteredCompletedTasks` call `sortTopLevelTasks` after top-level + tag filtering.
- Subtasks render via `getSubtasks(parentId)` in store order; only parent rows are reordered.
- Backend list order is unchanged (created-desc); sorting is client-side only.

## Persistence

- Key: `ironpad-task-sort`
- Invalid or missing stored values → **`created`** (matches API default)
- `readSortMode` / `writeSortMode` no-op safely when `localStorage` is unavailable

## Tests

Vitest: `frontend/src/lib/taskSort.test.ts` — all three modes, tie-breakers, due-date edge cases, invalid stored mode.

```bash
cd frontend && npm test -- src/lib/taskSort.test.ts
```

## Related

- PRD: `docs/ai-workflow/PRD-0.4.0.md` § Task list sort
- Due-date parsing: `docs/technical/task-dates.md`