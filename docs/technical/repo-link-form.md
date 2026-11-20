# Repo link form — save, validate, remove

`RepoLinkForm.vue` is the empty-state (and edit) UI for attaching a `github.com` repository to a project. It validates `owner/name` on the client, saves via the API, refreshes the Pinia snapshot, and can unlink with confirmation.

Related: [`project-github-routes.md`](./project-github-routes.md), [`github-frontend-api.md`](./github-frontend-api.md), [`github-store.md`](./github-store.md), [`github-view-scaffold.md`](./github-view-scaffold.md).

## Component

Path: `frontend/src/components/github/RepoLinkForm.vue`

| API | Detail |
|-----|--------|
| Prop | `projectId: string` |
| Emits | `saved` (after successful `putRepo` + `refresh`), `removed` (after `putRepo(null)` and snapshot cleared) |

Uses `useGithubStore()` for `snapshot` and `refresh`. **Remove** is shown only when `snapshot?.repo` is set (edit mode). Input is pre-filled from `snapshot.repo` when present.

Helper copy: *A github.com repository, separate from Remote backup. Ironpad does not store a token.*

## Client validation

On **Save**, trim the input and reject when any of:

- empty
- contains `://`
- ends with `.git`
- contains `..`
- slash count ≠ 1
- characters outside `[A-Za-z0-9._-/]`
- does not match `^[A-Za-z0-9._-]+/[A-Za-z0-9._-]+$`

Error text (matches backend `GITHUB_REPO_VALIDATION_MSG`):

`Use owner/name, with no URL and no .git suffix.`

## Flows

**Save:** `githubApi.putRepo(projectId, trimmed)` → `githubStore.refresh(projectId)` → emit `saved`. API errors appear in `localError`.

**Remove:** **Remove** opens inline confirmation: *Stop showing this repository. Tasks you already created stay.* **Confirm** calls `putRepo(projectId, null)`, sets `githubStore.snapshot = null`, emits `removed`.

## GitHubView wiring

`GitHubView.vue` still mounts `<RepoLinkForm v-if="!hasRepo" />` without `projectId`. Pass `:projectId="id"` when wiring the form into the view.

## Tests

`frontend/src/components/github/RepoLinkForm.test.ts` — mock `githubApi.putRepo` / `refresh`; Vitest `data-testid` hooks on input, save, error, remove, and confirm.

Run: `cd frontend && npm run test -- RepoLinkForm`.