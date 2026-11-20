# Dashboard tag chips (frontend)

The dashboard shows a row of tag chips derived from **all projects** via `tasksStore.allTasks`. Tags come only from task frontmatter `tags` — no scanning of note or prompt bodies.

## Module

`frontend/src/lib/dashboardTagChips.ts`

| Export | Role |
|--------|------|
| `TagChipSummaryItem` | `{ tag: string; count: number }` for one chip. |
| `buildTagChipSummary(allTasks)` | Pure function: distinct tags among **incomplete** tasks, sorted A–Z, each with how many incomplete tasks carry that tag. |

## Counting rules

- **Completed** tasks are excluded entirely (they do not affect counts).
- **Active and backlog** incomplete tasks both contribute (any `completed: false` task).
- A task with multiple tags increments the count for **each** tag once.
- Tag labels are **case-sensitive** (`Bug` and `bug` are separate chips).
- Chips are sorted with `localeCompare` on the tag string.

## UI (`DashboardView.vue`)

- `tagChipSummary` is a computed value from `buildTagChipSummary(tasksStore.allTasks)`.
- The chip row renders below the dashboard header when `tagChipSummary.length > 0` (no dedicated empty state when there are no tags).
- Each chip shows the tag label and a numeric count badge.
- `selectedDashboardTag` (`ref<string | null>`) tracks one selected chip; clicking the active chip clears selection.
- Card filtering and stats when a chip is selected: see [`dashboard-tag-filter.md`](./dashboard-tag-filter.md).

## Tests

Vitest: `frontend/src/lib/dashboardTagChips.test.ts`

```bash
cd frontend && npm test -- src/lib/dashboardTagChips.test.ts
```

Covers merged duplicate tags, case-sensitive labels, completed exclusion, and multi-tag tasks.

## Related

PRD 0.4.0 empty-state when no tasks match a selected tag is still follow-on; see [`dashboard-tag-filter.md`](./dashboard-tag-filter.md).