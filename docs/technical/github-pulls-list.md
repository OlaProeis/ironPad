# GitHub pulls list — filter tabs and rows

`PullsList.vue` renders open pull requests from `useGithubStore().filteredPulls` with per-project filter tabs, row metadata (including draft badge), empty state, and truncation notice. Row click emits `select` for the detail pane (placeholder in `GitHubView.vue` until the real pane lands).

Related: [`github-store.md`](./github-store.md), [`github-view-scaffold.md`](./github-view-scaffold.md), [`github-issues-list.md`](./github-issues-list.md).

## Component

`frontend/src/components/github/PullsList.vue`

- **Props:** `projectId` — passed to `setPrFilter` for `ironpad-github-pr-filter:<projectId>` persistence.
- **Store:** `filteredPulls`, `prFilter`, `hasLogin`, `snapshot`.
- **Tabs:** `all` (All open), `mine` (Mine), `review` (Review requested). Login-only tabs hidden when `!hasLogin`.
- **Rows:** `<button>` per PR — `#number title`, gray **Draft** badge when `draft === true`, label chips (same `labelBackground` pattern as issues), `author_login`, `relativeTime(updated_at)` — see [`github-issues-list.md`](./github-issues-list.md).
- **Empty:** `No open pull requests` when the filtered list is empty.
- **Truncation:** Same copy as issues when `snapshot.pulls_truncated` is true.
- **Emit:** `select(pull: GithubPull)` on row click.

## GitHubView wiring

`GitHubView.vue` mounts both lists when the project has a linked repo:

- `<IssuesList :project-id="id" @select="onSelectIssue" />`
- `<PullsList :project-id="id" @select="onSelectPull" />`

Selecting an issue or pull clears the other selection and shows `github-detail-placeholder` (stub sidebar) until the real detail pane is implemented.

## Tests

`frontend/src/components/github/PullsList.test.ts` — stubbed Pinia store: draft badge, `review` filter with login, hidden `mine`/`review` tabs without login, empty message, truncation notice.

Run: `cd frontend && npm run test -- PullsList`.