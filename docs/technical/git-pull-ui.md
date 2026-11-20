# Git pull button (frontend)

The Git panel exposes **Pull** beside **Fetch** when `origin` is configured. Fetch still updates remote-tracking refs only; Pull calls `POST /api/git/pull` (fast-forward upstream). Backend behavior: [`git-pull.md`](./git-pull.md).

## UI

`frontend/src/components/GitPanel.vue` — commit action row when `hasRemote`:

| Control | Behavior |
|---------|----------|
| **↓ Fetch** | Unchanged: `gitStore.fetchRemote()` → `POST /api/git/fetch`, refresh remote metadata |
| **⇣ Pull** | `gitStore.pull()` → `POST /api/git/pull` |

- **Errors**: server `message` in the panel error banner (`gitStore.error`).
- **Success**: green status line under the buttons (`gitStore.pullMessage`) with API text such as `Already up to date.` or `Pulled N commit(s).`.
- No extra file-watcher or refresh channel beyond what pull + store reload already trigger.

## Module and API client

`frontend/src/lib/gitPull.ts`

| Export | Role |
|--------|------|
| `displayPullResult` | Maps `{ success, message }` for UI |
| `submitPull` | Calls `gitApi.pull` |

`frontend/src/api/client.ts` — `gitApi.pull()` posts with no body and **always parses JSON** (including HTTP 400) so PRD sentences are not lost as thrown HTTP text.

`frontend/src/stores/git.ts` — `pull` uses `submitPull`; on success reloads status, remote, history, and working diff; `pulling` disables the button.

## Tests

Vitest: `frontend/src/lib/gitPull.test.ts` — display mapping, mocked fetch success and 400 error bodies. Build check: `npm run build` in `frontend/`.

## Related

- Backend: [`git-pull.md`](./git-pull.md), [`git-user-facing-errors.md`](./git-user-facing-errors.md)
- Remote setup UI: [`git-remote-ui.md`](./git-remote-ui.md)