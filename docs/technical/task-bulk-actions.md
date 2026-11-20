# Task bulk actions (frontend)

When one or more top-level tasks are selected in the project task list, a toolbar runs mark-complete, section move, and tag add/remove as sequential per-task API calls. There is no batch endpoint. Subtasks are not included.

## Module

`frontend/src/lib/bulkTaskActions.ts`

| Export | Role |
|--------|------|
| `tasksForBulkComplete` | Drops already-completed tasks so bulk complete does not toggle them back |
| `runBulkComplete` | Sequential `PUT .../tasks/:id/toggle` (recurrence stays on the toggle path) |
| `runBulkMetaUpdates` | Sequential `PUT .../tasks/:id/meta` |
| `metaForSectionMove` | `{ section }`; if the task is completed, also `{ completed: false }` |
| `normalizeTag` | Trimmed tag, or `null` when empty |
| `tagsAfterAdd` / `tagsAfterRemove` | Full `tags` array with append-once or remove |

Both runners stop on the first thrown error and return `{ ok, applied, failure: { title, error } }`. Earlier requests are left as saved.

## TasksView UI

`frontend/src/views/TasksView.vue` shows `.bulk-bar` when `multiSelectedIds` is non-empty: count, Mark complete, Move to Active, Move to Backlog, Add tag, Remove tag.

- Tag actions use `window.prompt`. An empty tag sets an inline error and does not call the API. Cancel closes the prompt with no requests.
- After the sequence, the view reloads project tasks and the open task. Selection clears only when `ok` is true. On failure the bar shows `title: error` and keeps the selection.
- Buttons are disabled while a run is in progress.

## Tests

Vitest: `frontend/src/lib/bulkTaskActions.test.ts` — skip completed, stop-on-error for toggle and meta, `completed: false` on section move. Fetch is stubbed.

Backend meta `completed: false` is covered by `cargo test meta_completed` (see `task-meta-completed.md`).

## Related

- Selection state: `docs/technical/task-multi-select.md`
- Meta completed flag: `docs/technical/task-meta-completed.md`
