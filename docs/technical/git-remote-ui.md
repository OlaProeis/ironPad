# Git remote URL UI (frontend)

The Git panel Changes tab lets users add or replace `origin` without using the terminal. Validation and post-save fetch failures come from `POST /api/git/remote` (see [`git-remote-origin.md`](./git-remote-origin.md)).

## UI

`frontend/src/components/GitPanel.vue` — **Remote backup** section above the commit form:

| State | UI |
|-------|-----|
| No `origin` | URL field + **Save remote** |
| Configured | Full URL, **Replace** (inline edit with Save / Cancel) |

- **Errors** (`success: false`): server `message` (validation and other 400 text).
- **Warnings**: `fetch_error` when save succeeded but the post-save fetch failed (URL is kept).
- **Success**: short `message` after save.
- **Git backup guide** link uses `GIT_BACKUP_DOC_HREF` (`/docs/git-backup`) with `@click.prevent` until the app shell wires docs navigation (task 22).

Push and Fetch buttons still appear only when `hasRemote` is true.

## Module and API client

`frontend/src/lib/gitRemoteSetup.ts`

| Export | Role |
|--------|------|
| `trimRemoteUrlInput` | Trims whitespace before POST |
| `submitSetRemote` | Calls `gitApi.setRemote` |
| `GIT_BACKUP_DOC_HREF` | In-app path `/docs/git-backup`; user guide at [`../git-backup.md`](../git-backup.md) |

`frontend/src/api/client.ts` — `gitApi.setRemote(url)` posts JSON and **always parses JSON** on 200 and 400 so validation errors are not thrown as raw HTTP text.

`frontend/src/stores/git.ts` — `setRemoteOrigin` calls `submitSetRemote` and reloads remote on success; `settingRemote` drives button disabled state.

Types: `SetRemoteResponse` in `frontend/src/types/index.ts`.

## Tests

Vitest: `frontend/src/lib/gitRemoteSetup.test.ts` — trim, successful POST body, credential-in-URL 400 surfaced without throwing. Fetch is stubbed.

Build check: `npm run build` in `frontend/`.

## Related

- Backend: [`git-remote-origin.md`](./git-remote-origin.md), [`git-user-facing-errors.md`](./git-user-facing-errors.md)
- User guide: [`git-backup.md`](../git-backup.md)
- Product: [`PRD-0.4.0.md`](../ai-workflow/PRD-0.4.0.md) §9 (Git backup)