# Library Raw / Rendered mode

One segmented control on the open library item in `PromptsView.vue`. Only one pane is in the DOM. Save always sends the raw draft ref, never rendered HTML.

Related: [`rendered-markdown.md`](./rendered-markdown.md), [`link-reading-pane.md`](./link-reading-pane.md).

## `readingMode.ts`

`frontend/src/lib/readingMode.ts`:

- **`ReadingMode`** — `'raw' | 'rendered'`.
- **`defaultReadingMode(item)`** — `'rendered'` when `item.kind === 'link'` and `item.supports_rendered` is true (markdown links). Prompts and non-markdown links (`.txt`, `supports_rendered === false`) default to `'raw'`.
- **`getReadingMode` / `setReadingMode`** — `localStorage` key `ironpad-library-mode:<itemId>`. Unknown or missing values use the fallback. Storage failures are ignored.

## `PromptsView.vue`

- Control sits above the content grid. The **Rendered** button is omitted when the open item is a link with `supports_rendered === false`, and the mode is forced to raw (stored `'rendered'` is not applied).
- **Raw** — existing prompt textarea (`draftContent`) or `LinkPane` for links.
- **Rendered** — `RenderedMarkdown` with `source` equal to substituted text when the item is a prompt or a link with `template: true`. Substitution is the same `{{name}}` replace used by **Copy Rendered** (`{{` without a closing `}}` stays literal). Other links pass the raw body.
- Switching modes does not replace `draftContent`. **Save** still calls `updatePrompt` with that draft (links omit `content`, same as before).

## Tests

| File | Covers |
|------|--------|
| `frontend/src/lib/readingMode.test.ts` | Defaults; set/get under `ironpad-library-mode:<itemId>` |
| `frontend/src/views/PromptsView.test.ts` | Draft survives Raw → Rendered → Raw; Save in Rendered mode sends the raw draft; `.txt` link has no Rendered option; `{{name}}` → value and unclosed `{{` stays literal |

Run in `frontend/`:

```bash
npx vitest run src/lib/readingMode.test.ts src/views/PromptsView.test.ts
```
