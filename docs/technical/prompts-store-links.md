# Prompts store — link load, reload, and focus refresh

Pinia `usePromptsStore` passes the per-project library root when opening items, keeps full link GET metadata on `selectedPrompt`, and refreshes open links on explicit reload and window focus — no file watcher.

Related: [`library-root.md`](./library-root.md), [`frontend-library-api.md`](./frontend-library-api.md), [`library-link-get.md`](./library-link-get.md).

## Root on GET

`loadPrompt` and `reloadOpen` call `promptsApi.get(id, root?)` where `root` comes from `getLibraryRoot(activeProjectId)` when `activeScope === 'project'` and `activeProjectId` is set. Global scope omits `root`.

The API response is stored whole on `selectedPrompt` (`kind`, `source`, `template`, `content`, `read_only`, `source_missing`, `resolved_path`, `root_required`, `source_error`, `supports_rendered`, etc.).

## `reloadOpen()`

Refetches the currently open id if `selectedPrompt` is set. Replaces `selectedPrompt` only when the id still matches (avoids races if the user switched items mid-flight). Does not toggle `loadingPrompt`.

## Focus refresh

- `registerFocusRefresh()` / `unregisterFocusRefresh()` attach a single `window` `focus` listener.
- On focus, calls `reloadOpen()` **only** when `selectedPrompt.kind === 'link'`. Open **prompt** items are not refetched (avoids clobbering in-editor drafts; `PromptsView` syncs draft from `selectedPrompt` id changes, not content-only updates).
- `PromptsView.vue` calls register on `onMounted` and unregister on `onUnmounted`.

## Create / update

`createPrompt` and `updatePrompt` forward optional `kind`, `source`, and `template` to the API client (same fields as `promptsApi.create` / `update`).

## Tests

`frontend/src/stores/__tests__/prompts.links.test.ts` (Vitest, mocked `promptsApi`):

- Project `loadPrompt` passes root from localStorage.
- `reloadOpen()` replaces `content`.
- `window.dispatchEvent(new Event('focus'))` refetches an open link, not an open prompt.
- `createPrompt({ kind: 'link', source: 'a.md' })` forwards those fields.

Run: `npm test -- src/stores/__tests__/prompts.links.test.ts` in `frontend/`.