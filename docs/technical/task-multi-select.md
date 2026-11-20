# Task bulk multi-select (frontend)

Release 0.4.0 adds checkboxes on **top-level** tasks in the project task list so a later bulk-action bar can operate on a selection. Subtasks are not selectable. Bulk API calls and the action bar are separate tasks.

## Module

`frontend/src/lib/taskMultiSelect.ts`

| Export | Role |
|--------|------|
| `toggleMultiSelect(selectedIds, taskId)` | Returns a new `Set` with the id toggled |
| `clearMultiSelect()` | Empty selection set |
| `pruneMultiSelect(selectedIds, visibleTopLevelIds)` | Drops ids not currently visible (tag filter) |
| `isMultiSelected(selectedIds, taskId)` | Membership check |

## TasksView UI

`frontend/src/views/TasksView.vue`:

- State: `multiSelectedIds` (`ref<Set<string>>`).
- **Checkbox** (`.multi-select-checkbox`) on each top-level row in Active, Backlog, and Completed — left of the existing complete toggle.
- **No checkbox** on indented subtask rows in the list or in the detail subtask panel.
- Row click still opens the task in the editor; checkbox uses `@click.stop` / `@change` so it does not navigate.
- **Tag filter:** `visibleTopLevelTaskIds` is the union of filtered top-level tasks in all three sections; `watch(visibleTopLevelTaskIds)` runs `pruneMultiSelect` so hidden tasks are removed from selection (not shown as checked).
- **Project change:** `clearMultiSelect` on `projectId` (route) and on `workspaceStore.activeProjectId`.

Completion checkboxes and detail-panel behavior are unchanged.

## Tests

Vitest: `frontend/src/lib/taskMultiSelect.test.ts` — toggle immutability, clear, prune, membership.

```bash
cd frontend && npm test -- src/lib/taskMultiSelect.test.ts
```

Build: `npm run build` in `frontend/`.

## Related

- PRD: `docs/ai-workflow/PRD-0.4.0.md` § Bulk actions (selection only in this slice)
- Tag filter in list: same `activeTagFilter` as sort sections in `task-sort.md`