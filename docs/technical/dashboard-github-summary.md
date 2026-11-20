# Dashboard GitHub summary on project cards

On dashboard load, the UI fetches snapshot-based GitHub counts from `GET /api/github/summary` and shows a per-card link when either count is positive. Counts reflect the last cached snapshot only; the dashboard does not trigger GitHub refresh.

## Implementation

**View:** `frontend/src/views/DashboardView.vue`

- Local ref `githubSummary` holds `GithubSummaryEntry[]` from `githubApi.getSummary()`.
- Called in `onMounted` after `loadProjects` and `loadAllTasks` complete.
- Failures are swallowed so GitHub errors do not block the dashboard.
- Computed `githubSummaryByProject` maps `project_id` → entry.

**Template:** Inside each project card header, when an entry exists and `assigned_issues + review_pulls > 0`:

```text
{assigned_issues} assigned · {review_pulls} reviews
```

The line is a `router-link` to `project-github` for that project (`@click.stop` so it does not activate the card header navigation). When the entry is missing or both counts are zero, nothing is rendered.

**Types / API:** `GithubSummaryEntry` in `frontend/src/types/index.ts`; `githubApi.getSummary()` in `frontend/src/api/client.ts`.

Backend counting rules and response shape: [`github-summary.md`](./github-summary.md).

## Tests

`frontend/src/views/DashboardView.test.ts` (Vitest, happy-dom):

- Mock `getSummary` with project `alpha`: `2 assigned · 1 reviews` link present.
- Both counts zero: no `a.github-summary-link`.
- `getSummary` rejects: dashboard grid still renders, no summary link.

```bash
cd frontend && npm run test -- DashboardView
```