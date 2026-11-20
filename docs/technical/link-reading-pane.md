# Link reading pane

When the open library item is a link, the editor shows read-only file text and link error states. Title, folder, tags, and description stay on the existing form.

Related: [`prompts-store-links.md`](./prompts-store-links.md), [`library-link-get.md`](./library-link-get.md), [`library-root.md`](./library-root.md), [`library-open.md`](./library-open.md).

## Where it mounts

`frontend/src/views/PromptsView.vue` renders `LinkPane` when `selectedPrompt.kind === 'link'` in place of the content textarea. Saving a link sends title, folder, description, and tags only — not `content`.

## `LinkPane.vue`

`frontend/src/components/library/LinkPane.vue` takes the open `Prompt`.

- Header shows `source` and `resolved_path`, plus **Copy path**, **Open externally**, and **Reload**.
- **Copy path** writes `resolved_path` with `navigator.clipboard.writeText`.
- **Open externally** calls `libraryApi.openLibraryItem(id, getLibraryRoot(project_id) ?? undefined)`; failures show inline as `link-open-error` (backend error text).
- **Copy path** and **Open externally** are disabled when `source_missing`, `root_required`, or `source_error` is set (missing-file Relink and other error UI stay visible).
- **Reload** calls `promptsStore.reloadOpen()`.
- Otherwise the raw file text is a `<pre>` (nothing from this pane is written back).
- `source_missing`: stored `source`, "File not found", and **Relink**. The input submits `updatePrompt(id, { source })` (relative path for project scope, absolute for global).
- `root_required`: "Set a folder root for this project to open this link", plus `LibraryRootBar` when `project_id` is set.
- `source_error`: the backend string as returned (includes the byte size when the file is over the cap).

Rendered mode and template toggle live on `PromptsView`, not in this pane.

## Tests

`frontend/src/components/library/__tests__/LinkPane.test.ts` (Vitest, happy-dom): missing-file Relink payload, root prompt text, verbatim source error, read-only `<pre>` content, Reload → `reloadOpen`, Copy path clipboard, Open externally → `openLibraryItem`, disabled actions when `source_missing`.

Run: `npm test -- src/components/library/__tests__/LinkPane.test.ts` in `frontend/`.
