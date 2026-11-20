# GitHub view scaffold — route, sidebar, and list vs link form

Wires the per-project GitHub screen: Vue Router entry, sidebar navigation, and `GitHubView.vue` that loads the Pinia snapshot and switches between the repo-link form and list placeholders.

Related: [`github-store.md`](./github-store.md), [`github-frontend-api.md`](./github-frontend-api.md).

## Route

In `frontend/src/router/index.ts`, after other project routes:

- Path: `/projects/:id/github`
- Name: `project-github`
- Component: lazy `GitHubView.vue`, `props: true` (route param `id`)

## Sidebar

`Sidebar.vue`:

- `goToProjectGithub()` — same pattern as `goToProjectNotes` / `goToProjectPrompts`; navigates to `project-github` when `activeProjectId` is set.
- Nav button **★ GitHub** after **Library**, visible when `activeProjectId` is truthy; active when `route.name === 'project-github'`.

## GitHubView

`frontend/src/views/GitHubView.vue`:

- Prop: `id` (project id).
- On mount and when `id` changes: `useGithubStore().loadSnapshot(id)`; auto-refresh when stale, header Refresh, 15-minute interval, error banner — see [`github-view-refresh.md`](./github-view-refresh.md).
- `hasRepo` — `Boolean(snapshot?.repo)` after load.
- While `githubStore.loading`: loading line.
- `v-if="!hasRepo"`: `<RepoLinkForm />` (pass `:projectId="id"` when wired; see [`repo-link-form.md`](./repo-link-form.md)).
- `v-else`: `.github-panels` with `<IssuesList>` and `<PullsList>` (both `:project-id="id"`, `@select` → placeholder detail aside).

| Component | Path | Status |
|-----------|------|--------|
| `RepoLinkForm` | `frontend/src/components/github/RepoLinkForm.vue` | implemented |
| `IssuesList` | `frontend/src/components/github/IssuesList.vue` | implemented — see [`github-issues-list.md`](./github-issues-list.md) |
| `PullsList` | `frontend/src/components/github/PullsList.vue` | implemented — see [`github-pulls-list.md`](./github-pulls-list.md) |

List components expose `data-testid` for Vitest.

## Types

`Project` in `frontend/src/types/index.ts` includes optional `github_repo?: string` (matches backend project GET JSON).

## Tests

- `frontend/src/components/Sidebar.test.ts` — GitHub nav when a project is active.
- `frontend/src/views/GitHubView.test.ts` — mocked `githubApi.getSnapshot`: no `repo` → repo-link form; with `repo` → `IssuesList` / `PullsList`, no form.

Run: `cd frontend && npm run test -- Sidebar GitHubView`.