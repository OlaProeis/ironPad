# Add link dialog

Browse one directory from `GET /api/library/browse` and create one library link per selected file.

Related: [`library-browse.md`](./library-browse.md), [`library-root.md`](./library-root.md), [`prompts-store-links.md`](./prompts-store-links.md).

## Where it mounts

`frontend/src/views/PromptsView.vue` puts **Add link** next to **+ New**. It opens `AddLinkDialog` with `promptsStore.activeScope` and `activeProjectId`.

## `AddLinkDialog.vue`

`frontend/src/components/library/AddLinkDialog.vue`.

- Project scope starts at `getLibraryRoot(projectId)`. No root shows "Set a folder root for this project to open this link" and does not browse.
- Global scope shows an absolute-directory field and **Go** before the first listing.
- `libraryApi.browseLibrary(path)` fills the list. **Up** uses `parent`. A directory click loads that path (one level, no import).
- `linkable` files get a checkbox. Other files stay visible with a disabled checkbox and `not allowed` or `too large` (same suffix set and 1_048_576 byte cap as the backend).
- **Confirm** calls `promptsStore.createPrompt` once per selected file, in order: `kind: 'link'`, `title` = file name, `tags: []`, `template: false`, `folder` omitted when the active folder filter is `root`. Project `source` is `toRelativeSource(root, entry.path)`; global `source` is `entry.path`.
- **Paste path** (`add-link-paste-path`, **Add**): visible when the project has a root or in global mode. One file per add; success closes the dialog like **Confirm**.

### Paste path rules

| Scope | Input | Stored `source` | Refused |
|-------|--------|----------------|---------|
| Project | Absolute under `projectRoot` | `toRelativeSource(root, path)` | Outside root: *This file is outside the project root. Change the root or add it from the global library.* |
| Project | Relative (no `..`) | Forward slashes, no leading `/` | `..` segments: same wording as backend project validation (*must not contain '..'*) |
| Global | Absolute | `normalizeAbsolutePath(path)` | Relative: *Enter an absolute path.* |

Suffix and other library validation run on `POST /api/prompts`; failures surface in `add-link-error` (API error text).

Well-known file suggestions live in [`library-suggest-files.md`](./library-suggest-files.md) (`SuggestFilesPanel`), not in this dialog.

## Tests

`frontend/src/components/library/__tests__/AddLinkDialog.test.ts` (Vitest, happy-dom): directory navigation, disabled non-linkable file, project source `ai-context.md` when root is `/repo`, global source `/repo/ai-context.md`, missing-root prompt, paste-path containment and global absolute rules.

Run: `npm test -- src/components/library/__tests__/AddLinkDialog.test.ts` in `frontend/`.
