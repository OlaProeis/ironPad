# GitHub API client and types (frontend)

Release 0.4.0 adds a typed `githubApi` on the shared HTTP client so GitHub UI and stores can call backend routes from PRD §10 without ad-hoc `fetch`.

## Types

Module: `frontend/src/types/index.ts`

| Type | Role |
|------|------|
| `GithubLabel`, `GithubIssue`, `GithubPull` | Open list items from snapshot JSON |
| `GithubSnapshotResponse` | `GET`/`POST` refresh project GitHub endpoint body |
| `GithubComment`, `GithubItemDetail` | Issue/pull detail (body, comments, PR `commit_status`) |
| `TrackResult` | `{ task_id, created }` from track endpoints |
| `GithubSummaryEntry` | One row from `GET /api/github/summary` |

Field names match `backend/src/routes/github.rs` and `services/github/snapshot.rs` serde shapes.

## Client

Module: `frontend/src/api/client.ts` — `export const githubApi`:

| Method | HTTP | Path |
|--------|------|------|
| `getSnapshot(projectId)` | GET | `/api/projects/{id}/github` |
| `putRepo(projectId, repo)` | PUT | `/api/projects/{id}/github` — body `{ repo }` (`null` clears link) |
| `refresh(projectId)` | POST | `/api/projects/{id}/github/refresh` |
| `getIssue(projectId, number)` | GET | `/api/projects/{id}/github/issues/{number}` |
| `getPull(projectId, number)` | GET | `/api/projects/{id}/github/pulls/{number}` |
| `trackIssue(projectId, number)` | POST | `/api/projects/{id}/github/issues/{number}/track` |
| `trackPull(projectId, number)` | POST | `/api/projects/{id}/github/pulls/{number}/track` |
| `getSummary()` | GET | `/api/github/summary` |

All methods use the shared `request<T>` helper (`encodeURIComponent` on project id). `putRepo` sends `Content-Type: application/json`; clearing the repo uses `repo: null` and may receive an empty 204 response.

Out of scope for this layer: Pinia store and Vue components (later tasks).

## Tests

`frontend/src/api/__tests__/client.github.test.ts` — Vitest with `vi.stubGlobal('fetch', …)` (same pattern as `client.library.test.ts`):

- `getSnapshot` resolves a stub `GithubSnapshotResponse`
- `putRepo(..., null)` uses PUT with body `{"repo":null}`

Run: `cd frontend && npm run test -- client`

## Related

- Backend project routes: [`project-github-routes.md`](./project-github-routes.md)
- Dashboard summary: [`github-summary.md`](./github-summary.md)
- Item detail and track: [`github-item-detail.md`](./github-item-detail.md), [`github-track.md`](./github-track.md)
- Library client precedent: [`frontend-library-api.md`](./frontend-library-api.md)