# Dashboard tag filter (frontend)

When a dashboard tag chip is selected, project cards show only **incomplete** tasks that carry that tag. Per-card stats and the active-task list are recomputed from that subset; cards with zero matches are hidden.

## Module

`frontend/src/lib/dashboardTagFilter.ts`

| Export | Role |
|--------|------|
| `filterTasksByTag(tasks, tag)` | Incomplete tasks whose `tags` array includes `tag` (exact, case-sensitive — same rule as `TasksView` `activeTagFilter`). |
| `computeProjectCardStats(projectTasks, todayLocal)` | Active task list, backlog/completed counts, overdue via `countActiveOverdueTasks`, and `totalCount`. |
| `shouldShowTagFilterEmptyState(selectedTag, visibleProjectCardCount)` | `true` when a tag is active and every project card is hidden (`visibleProjectCardCount === 0`). |
| `clearSelectedDashboardTag(_)` | Returns `null` for resetting selection (used by clear control; same effect as toggling the active chip off). |
| `ProjectCardStats` | Return shape for one project card. |

## UI (`DashboardView.vue`)

- `selectedDashboardTag` — one tag at a time; clicking the active chip clears the filter (unchanged from chip row).
- `projectSummaries` — for each project, takes tasks from `tasksStore.allTasks`, applies `filterTasksByTag` when a tag is selected, then `computeProjectCardStats`.
- Cards are dropped when a tag is selected and `totalCount === 0` after filtering.
- With no tag selected, all projects use the full project task list (same stats rules as before this feature).
- **Tag filter empty state** — when `shouldShowTagFilterEmptyState` is true, the grid is replaced by a message naming the tag and a **Clear tag filter** button (`clearDashboardTagFilter` → `selectedDashboardTag = null`). This block uses `v-if` so it can appear while the chip row is still visible (do not chain it as `v-else-if` on the chip row).
- **Welcome empty state** — “Create your first project” only when `projectsStore.sortedProjects.length === 0` and the tag-filter empty state is not showing (avoids mislabeling a zero-match tag filter as a new workspace).

## Related

- Tag chip list and counts: [`dashboard-tag-chips.md`](./dashboard-tag-chips.md)
- Overdue on cards: [`task-dates.md`](./task-dates.md) (`countActiveOverdueTasks`)

## Tests

Vitest: `frontend/src/lib/dashboardTagFilter.test.ts`

```bash
cd frontend && npm test -- src/lib/dashboardTagFilter.test.ts
```
