# Library copy bundle

The Library list can multi-select items and copy them as one markdown blob. Nothing is written to disk. **Copy** and **Copy Rendered** on the open item are unchanged.

Related: [`prompts-store-links.md`](./prompts-store-links.md), [`library-link-get.md`](./library-link-get.md).

## Blob shape

`frontend/src/lib/copyBundle.ts` exports `buildBundle(items)`. Each item is `{ title, path, text }`. Output, in order, is:

```text
## <title>

<path>

<text>

```

One trailing newline is removed from the whole string. Text is copied as given; `{{name}}` is not substituted.

## UI

In `frontend/src/views/PromptsView.vue`:

- Each list row has a checkbox bound to `selectedIds` (`Set<string>`). The checkbox click does not change the open item.
- Toolbar button **Copy bundle** (`data-testid="copy-bundle"`).
- Ordered ids are the visible list order intersected with `selectedIds`. If that set is empty, the open item id is used.
- Each id is loaded with `promptsApi.get`. In project scope the call passes `getLibraryRoot(activeProjectId)`.
- Prompts contribute `path` (or `data/prompts/<id>.md`) and `content`.
- Links contribute `resolved_path` (else `source`) and `content`. `source_missing` uses the text `(file missing)`.
- The string is written with `navigator.clipboard.writeText`.

## Tests

- `frontend/src/lib/copyBundle.test.ts` — two sections in order; `{{name}}` left intact.
- `frontend/src/views/PromptsView.test.ts` — two checked rows write both titles and leave the open id unchanged; empty selection bundles the open item; a link section includes `resolved_path`.
