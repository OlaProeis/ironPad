# GitHub Pinia store — snapshot and list filters

`useGithubStore` in `frontend/src/stores/github.ts` holds the project GitHub snapshot from the API, loading/error state, per-project issue/PR filter prefs in localStorage, and computed filtered lists — no Vue component coupling yet.

Related: [`github-frontend-api.md`](./github-frontend-api.md), [`project-github-routes.md`](./project-github-routes.md).

## State and API actions

- `snapshot`, `loading`, `error`
- `loadSnapshot(projectId)` — `initFilters(projectId)` then `githubApi.getSnapshot`
- `refresh(projectId)` — `githubApi.refresh` (does not re-read filter keys)

## Filters

| Ref | Values | localStorage key |
|-----|--------|------------------|
| `issueFilter` | `all`, `assigned`, `created` | `ironpad-github-issue-filter:{projectId}` |
| `prFilter` | `all`, `mine`, `review` | `ironpad-github-pr-filter:{projectId}` |

`initFilters` validates stored strings and falls back to `all`. `setIssueFilter` / `setPrFilter` update the ref and write localStorage.

## Computed lists

- `hasLogin` — `!!snapshot.login`
- `filteredIssues` — If login is missing, always returns all issues (filters ignored). `assigned`: `assignee_logins` includes login (case-insensitive). `created`: `author_login` matches login (case-insensitive).
- `filteredPulls` — If login is missing, returns all pulls. `mine`: author matches login. `review`: `requested_reviewer_logins` includes login (case-insensitive).

## Export

Re-exported from `frontend/src/stores/index.ts` as `useGithubStore`.

## Tests

`frontend/src/stores/__tests__/github.test.ts` (Vitest, happy-dom, mocked `githubApi`):

- `loadSnapshot` stores mocked response.
- Assigned filter with `login` / assignee case mismatch.
- Assigned filter with no login returns full list.
- `setIssueFilter` + `initFilters` round-trip via localStorage.
- Unknown stored filter values → `all`.

Run: `npm test -- github.test` in `frontend/` (also matches `client.github.test.ts` unless filtered by path).