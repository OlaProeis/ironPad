# Frontend library types and API client

TypeScript contracts and HTTP helpers for library link pointers and library endpoints. Stores and UI for links build on this surface in later tasks.

Related backend docs: [`library-link-get.md`](./library-link-get.md), [`library-browse.md`](./library-browse.md), [`library-open.md`](./library-open.md), [`prompt-link-create-update.md`](./prompt-link-create-update.md).

## Types (`frontend/src/types/index.ts`)

| Type | Purpose |
|------|---------|
| `PromptKind` | `'prompt' \| 'link'` — matches API JSON. |
| `PromptSummary` | List rows include required `kind` and optional `source`. |
| `Prompt` | Full item plus `template`, link GET flags (`read_only`, `source_missing`, optional `resolved_path`, `root_required`, `source_error`, `supports_rendered`). |
| `BrowseEntry` / `BrowseResponse` | Directory listing for the add-link browser. |
| `OpenLibraryResponse` | `{ ok, resolved_path }` from open. |

Create/update responses from the API may omit link GET-only fields; the type matches the flattened `GET /api/prompts/{id}` body.

## API client (`frontend/src/api/client.ts`)

### Prompts

- `promptsApi.get(id, root?)` — when `root` is set, appends `?root=` (encoded) for project-scoped link resolution.
- `promptsApi.create` / `update` payloads accept optional `kind`, `source`, `template`.

### Library

`libraryApi` (same `request` wrapper and error handling as other APIs):

| Method | HTTP |
|--------|------|
| `browseLibrary(path)` | `GET /api/library/browse?path=` |
| `openLibraryItem(id, root?)` | `POST /api/library/open` with JSON `{ id }` or `{ id, root }` |

## Tests

Vitest is configured in `frontend/vite.config.ts` (`include: src/**/*.test.ts`).

`frontend/src/api/__tests__/client.library.test.ts` mocks `fetch` and asserts URL encoding for browse, POST body for open, and root query on `promptsApi.get`.

Run: `npm test` in `frontend/`.