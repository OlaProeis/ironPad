# GitHub issues list — filter tabs and rows

`IssuesList.vue` renders open issues from `useGithubStore().filteredIssues` with per-project filter tabs, row metadata, empty state, and truncation notice. Row click emits `select` for a future detail pane.

Related: [`github-store.md`](./github-store.md), [`github-view-scaffold.md`](./github-view-scaffold.md).

## Component

`frontend/src/components/github/IssuesList.vue`

- **Props:** `projectId` — passed to `setIssueFilter` for `ironpad-github-issue-filter:<projectId>` persistence.
- **Store:** `filteredIssues`, `issueFilter`, `hasLogin`, `snapshot`.
- **Tabs:** `all` (All open), `assigned` (Assigned to me), `created` (Created by me). Tabs that require login are hidden when `!hasLogin`; invalid stored values are normalized to `all` in the store.
- **Rows:** `<button>` per issue — `#number title`, label chips with `label.color` (fallback `#e5e7eb`), `author_login`, assignee list, `relativeTime(updated_at)`. Each row includes `TrackButton` (`kind="issue"`). See [`github-track-button.md`](./github-track-button.md).
- **Empty:** `No open issues` when the filtered list is empty.
- **Truncation:** `Showing the 500 most recently updated open items.` when `snapshot.issues_truncated` is true.
- **Emit:** `select(issue: GithubIssue)` on row click.

`GitHubView.vue` mounts `<IssuesList :project-id="id" />` when the project has a linked repo.

## relativeTime

`frontend/src/lib/relativeTime.ts` — pure `Date` arithmetic from an ISO timestamp to a human string (no external library).

## Tests

`frontend/src/components/github/IssuesList.test.ts` — stubbed Pinia store: two rows for `all`, assigned filter with login, hidden assigned/created tabs without login, empty message, truncation notice, `relativeTime` smoke case.