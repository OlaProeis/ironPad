# GitHub view — snapshot refresh triggers

`GitHubView.vue` loads the cached GitHub snapshot and keeps it fresh while the screen is open: stale-on-load refresh, manual Refresh, and a 15-minute interval aligned with backend snapshot TTL (see [`github-refresh.md`](./github-refresh.md)).

Related: [`github-view-scaffold.md`](./github-view-scaffold.md), [`github-store.md`](./github-store.md).

## Load and stale refresh

On mount and when route prop `id` (project id) changes:

1. `githubStore.loadSnapshot(id)` via `githubApi.getSnapshot`.
2. If `snapshot.stale === true` after load, immediately `githubStore.refresh(id)` (does not wait for user action).

## Manual Refresh

Header control `data-testid="github-refresh-btn"`:

- Calls `githubStore.refresh(props.id)`.
- Local `refreshing` ref disables the button and shows a spinner while that call is in flight.

## 15-minute interval

- `setInterval(() => githubStore.refresh(projectId), 15 * 60 * 1000)` stored in a ref.
- Started on mount and restarted when `props.id` changes (clears the previous interval first).
- `onUnmounted` clears the interval so no refresh runs after leaving the view.

## Updated time

When `snapshot.refreshed_at` is set (UTC ISO from the API), the header shows **Updated** plus `new Date(refreshed_at).toLocaleTimeString()` (`data-testid="github-updated-time"`).

## Error banner

When `githubStore.error` is non-null:

- One-line dismissable banner (`data-testid="github-error-banner"`, `role="alert"`).
- Dismiss only hides the banner; the store error remains until the next successful load/refresh clears it.
- On successful refresh, store `error` is cleared and the dismiss flag resets so a future error shows again.
- Issue/PR lists and the rest of the layout stay visible under the banner (stale snapshot data remains on screen).

Out of scope on this view: issue/pull detail pane and track actions.

## Tests

`frontend/src/views/GitHubView.test.ts` — Vitest + happy-dom, `vi.useFakeTimers()`:

| Case | Assertion |
|------|-----------|
| Mount, `stale: true` | `githubApi.refresh` called once after `getSnapshot` |
| Mount, `stale: false` | No `refresh` on load |
| Click Refresh | `refresh` with project id |
| Advance 15 minutes | Interval `refresh` |
| Unmount then advance 15 minutes | No `refresh` |
| Store `error` set | Error banner in DOM |

Run: `cd frontend && npm run test -- GitHubView`.