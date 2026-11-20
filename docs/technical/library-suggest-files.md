# Suggested repo files panel

After a per-project library root is set, the UI can offer well-known repo files as one-click links, skipping paths already linked and remembering dismissals.

Related: [`library-root.md`](./library-root.md), [`library-browse.md`](./library-browse.md), [`add-link-dialog.md`](./add-link-dialog.md).

## Where it mounts

`frontend/src/views/PromptsView.vue` renders `SuggestFilesPanel` under `LibraryRootBar` when the route is **project** scope (`isProjectScope` + `routeProjectId`). `LibraryRootBar` emits `root-changed` so the panel refreshes when the root is set or cleared.

## `librarySuggest.ts`

`frontend/src/lib/librarySuggest.ts`.

- `SUGGESTED`: `ai-context.md`, `ROADMAP.md`, `docs/index.md` (fixed list; no recursive scan).
- `findSuggestions(root, existingSources, browse)` calls `browse(root)` and `browse(\`${root}/docs\`)`; browse errors are ignored. Root listing matches the first two names by `entry.name` with `linkable`. `docs/index.md` matches `index.md` in the docs listing only.
- Returns relative path strings not present in `existingSources`.
- Dismissal: `ironpad-library-suggest-dismissed:<projectId>` via `isSuggestDismissed` / `dismissSuggest` (`'1'` when dismissed).
- `titleForSuggestedSource(relative)` uses the final path segment for `createPrompt` title (e.g. `docs/index.md` → `index.md`).

## `SuggestFilesPanel.vue`

`frontend/src/components/library/SuggestFilesPanel.vue`.

- Hidden when there is no root, the user dismissed suggestions, or `findSuggestions` returns nothing.
- `existingSources` from `promptsStore.prompts` where `kind === 'link'` and `source` is set.
- All suggestion checkboxes start checked. **Add selected** runs `createPrompt` per checked path (same defaults as Add link: `kind: 'link'`, empty tags, `template: false`, folder omitted when active folder is `root`). Then `refresh()` so newly linked paths drop off the list.
- **Dismiss** calls `dismissSuggest(projectId)` and hides the panel.
- Optional prop `refreshToken` (incremented from `PromptsView` on `root-changed`) forces a rescan.

## Tests

- `frontend/src/lib/__tests__/librarySuggest.test.ts`: mocked browse, filter by `existingSources`, dismiss storage key.
- `frontend/src/components/library/__tests__/SuggestFilesPanel.test.ts`: two suggestions, **Add selected** → two `createPrompt` calls with `kind: 'link'`, **Dismiss** hides panel and sets localStorage.

Run: `npm test -- src/lib/__tests__/librarySuggest.test.ts src/components/library/__tests__/SuggestFilesPanel.test.ts` in `frontend/`.