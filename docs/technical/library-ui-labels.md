# Library navigation labels (frontend)

Release 0.4.0 renames user-facing **Prompts** copy to **Library** while routes, stores, filenames, and `/api/prompts` stay unchanged.

## Where labels appear

| Location | User-facing text | `title` / heading | Route (unchanged) |
|----------|------------------|-------------------|-------------------|
| `frontend/src/components/TopBar.vue` | Library | Global Library | `prompts` |
| `frontend/src/components/Sidebar.vue` (project nav) | Library | — | `project-prompts` |
| `frontend/src/views/PromptsView.vue` | — | `<h3>Library</h3>` in the list panel | `prompts` / `project-prompts` |

Navigation handlers remain `goToGlobalPrompts` and `goToProjectPrompts`; only button and heading strings changed.

## Out of scope

Lowercase “prompts” in placeholders, empty states, and API/store names (`usePromptsStore`, `promptsApi`, `PromptsView.vue`) are intentional until a broader rename.

## Tests

`frontend/src/components/Sidebar.test.ts` — mounts `Sidebar` with Pinia, router stub, and stubbed overlays; asserts a `.project-nav .nav-item` renders **Library**.

Vitest uses `@vue/test-utils` and `happy-dom` (`// @vitest-environment happy-dom` on that file).

## Related

- API and types: [`frontend-library-api.md`](./frontend-library-api.md)
- Backend library endpoints: [`library-browse.md`](./library-browse.md), [`library-link-get.md`](./library-link-get.md)