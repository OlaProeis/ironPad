# Link template toggle and variable gating

Links can opt into prompt-style `{{variable}}` substitution without changing prompt behavior. The **Template** checkbox only updates the library pointer (`template` in frontmatter), not linked file content.

Related: [`reading-mode.md`](./reading-mode.md), [`link-reading-pane.md`](./link-reading-pane.md), [`prompt-link-create-update.md`](./prompt-link-create-update.md).

## When variables apply

`templateVariablesEnabled` in `PromptsView.vue` is true when:

- `kind === 'prompt'`, or
- `kind === 'link'` and `template === true`.

When false on a link, `{{…}}` in the linked body is plain text: no **Variables** row, no **Copy Rendered**, and no **Rendered Preview** in the history column (even if the body contains placeholders).

**Copy** for a link with `template: false` writes `draftContent` verbatim (source text). **Copy Rendered** and in-memory substitution for render/copy use the same rules as prompts only when `templateVariablesEnabled` is true.

## UI and API

- **Template** checkbox (`data-testid="link-template"`) appears in the editor sub-row for links only.
- Toggling calls `promptsStore.updatePrompt(id, { template })` — no `content` or other fields.
- `renderedSource` already substituted for prompts and template links; non-template links pass the raw body (see [`reading-mode.md`](./reading-mode.md)).

## Tests

| File | Covers |
|------|--------|
| `frontend/src/views/PromptsView.test.ts` | Template off: no `name` field, Copy copies `Hi {{name}}`; toggle sends `{ template: true }` only; template on: variable field and **Copy Rendered** → `Hi Ada` |

Run in `frontend/`:

```bash
npx vitest run src/views/PromptsView.test.ts
```