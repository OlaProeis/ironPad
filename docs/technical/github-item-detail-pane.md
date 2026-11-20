# GitHub item detail pane

`ItemDetailPane.vue` is the read-only side pane for one open issue or pull request. `GitHubView.vue` shows it when a list row is selected.

Related: [`github-item-detail.md`](./github-item-detail.md), [`github-issues-list.md`](./github-issues-list.md), [`github-pulls-list.md`](./github-pulls-list.md), [`rendered-markdown.md`](./rendered-markdown.md).

## View wiring

`frontend/src/views/GitHubView.vue`

- `selectedItem` (`GithubIssue | GithubPull | null`) and `selectedKind` (`'issue' | 'pull'`).
- Issues and pulls lists `@select` set both refs. `close` clears `selectedItem`.
- `removed` is true when the selected number is missing from the current snapshot list for that kind. The pane still shows **Open on GitHub**.

## Pane

`frontend/src/components/github/ItemDetailPane.vue`

- **Props:** `projectId`, `item`, `kind`, optional `removed`. **Emit:** `close`.
- On `projectId`, `kind`, or `item.number` change, calls `githubApi.getIssue` or `getPull`. The previous `detail` stays visible until the new response arrives. A failed request leaves the cached detail in place.
- **Body:** `<RenderedMarkdown :source="detail.body" />` (markdown-it `html: false`). When `body_too_large` is true, the pane shows "The body is too large to display" and does not render the body.
- **Comments:** `detail.comments` (backend already caps at 30, newest first). Each row shows `author_login`, `relativeTime(created_at)`, and rendered Markdown. `more_comments` shows "Earlier comments are on GitHub".
- **PR status:** when `kind === 'pull'` and `commit_status` is set, one line `Status: {commit_status}` with a dot: green `success`, yellow `pending`, red `failure` or `error`, gray otherwise (including `No status`).
- **Actions:** `<a href="item.html_url" target="_blank" rel="noreferrer">Open on GitHub</a>` and **Copy link** (`navigator.clipboard.writeText`). `removed` shows "This item is no longer open".
- **Error:** `detail.error` uses the same red banner as `GitHubView` (`role="alert"`).

**Track as task:** `TrackButton` below the body when `detail` is loaded. See [`github-track-button.md`](./github-track-button.md).

## Tests

`frontend/src/components/github/ItemDetailPane.test.ts` — mocked `githubApi`: getIssue on mount, body too large, more comments, pull failure status, Open on GitHub href, `detail.error`, and no `<script>` element when the body contains a script tag.
