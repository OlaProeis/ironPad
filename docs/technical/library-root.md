# Per-project library folder root (frontend)

Release 0.4.0 stores each project’s coding-repo directory on the client so project-scoped library links can resolve without writing paths into `data/`.

## Storage

| Key | Value |
|-----|--------|
| `ironpad-library-root:<projectId>` | Absolute directory path (opaque string; no server sync) |

Module: `frontend/src/lib/libraryRoot.ts`

| Export | Role |
|--------|------|
| `getLibraryRoot(projectId)` | Read stored path or `null` |
| `setLibraryRoot(projectId, path)` | `localStorage.setItem` |
| `clearLibraryRoot(projectId)` | `removeItem` |
| `normalizeAbsolutePath(path)` | `\` → `/`, lowercase Windows drive letter, trim trailing slashes |
| `isInsideRoot(root, absPath)` | Prefix check with boundary (`/`) so `C:/repo` does not match `C:/repo2/...` |
| `toRelativeSource(root, absPath)` | Relative path with `/` separators, no leading slash |

Downstream link UI will pass `root` from this value into `promptsApi.get(id, root?)` and browse/open calls; this task does not wire those flows.

## UI

`frontend/src/components/library/LibraryRootBar.vue` — **Folder root** label, current path or **No folder root**, path input, **Set** / **Clear**.

Rendered in `frontend/src/views/PromptsView.vue` only on the project library route (`isProjectScope` / `project-prompts`), directly above the prompt list.

## Tests

`frontend/src/lib/__tests__/libraryRoot.test.ts` — storage round-trip, `isInsideRoot` / `toRelativeSource` cases, `LibraryRootBar` Set/Clear (`data-testid` hooks). Vitest with `happy-dom` (no `jsdom` dependency in the frontend package).

## Related

- Backend project `source` validation: [`library-source-resolution.md`](./library-source-resolution.md)
- Link GET with optional `root`: [`library-link-get.md`](./library-link-get.md)
- Frontend client `promptsApi.get(id, root?)`: [`frontend-library-api.md`](./frontend-library-api.md)