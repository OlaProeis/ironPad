# Rendered markdown (library)

Read-only HTML preview for library **Rendered** mode. Strips YAML frontmatter, uses the same markdown-it options as note preview, and blocks common raw-HTML injection patterns before render. Mode switching lives in [`reading-mode.md`](./reading-mode.md).

Related: [`link-reading-pane.md`](./link-reading-pane.md), [`library-linked-file-read.md`](./library-linked-file-read.md).

## `markdownBody.ts`

`frontend/src/lib/markdownBody.ts`:

- **`stripFrontmatter(text)`** — removes a leading `---\n…\n---\n` block only at offset 0; `---` elsewhere is unchanged.
- **`sanitizeMarkdownSource(text)`** — drops `<script>…</script>` and `onerror=…` from source before markdown-it runs (defense in depth with `html: false`).
- **`prepareMarkdownBody(text)`** — `sanitizeMarkdownSource(stripFrontmatter(text))`.

## `RenderedMarkdown.vue`

`frontend/src/components/library/RenderedMarkdown.vue` accepts prop **`source: string`**.

1. `prepareMarkdownBody(source)` → markdown-it (CommonMark-style: `html: false`, `breaks`, `linkify`, `typographer`, same as `MarkdownPreview.vue`).
2. Output is bound with `v-html` on `.rendered-markdown__body`.
3. Empty `source` renders an empty body without throwing.

Mode switching and `{{var}}` substitution are in [`reading-mode.md`](./reading-mode.md). This component only renders the `source` string it is given.

## Tests

| File | Covers |
|------|--------|
| `frontend/src/lib/markdownBody.test.ts` | Frontmatter strip, mid-doc `---`, no frontmatter |
| `frontend/src/components/library/__tests__/RenderedMarkdown.test.ts` | `# Title` → `<h1>`; no `<script>` or `onerror=` in `wrapper.html()`; empty string |

Run in `frontend/`:

```bash
npm test -- markdownBody.test.ts RenderedMarkdown.test.ts
```