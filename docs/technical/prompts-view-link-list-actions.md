# PromptsView — list kind badges and link duplicate/delete

Library list rows show whether each item is a stored prompt or a link pointer. Editor actions use link-specific duplicate and delete copy.

Related: [`prompt-pointer-frontmatter.md`](./prompt-pointer-frontmatter.md), [`prompt-link-create-update.md`](./prompt-link-create-update.md), [`prompts-store-links.md`](./prompts-store-links.md).

## List badges

In `frontend/src/views/PromptsView.vue`, each row in `.prompt-list` renders `.kind-badge` from `PromptSummary.kind` only (list payload / frontmatter; no per-row GET):

- `kind === 'link'` → **Link**
- otherwise → **Prompt**

## Duplicate

- Button label: **Duplicate link** when `selectedPrompt.kind === 'link'`, else **Duplicate**.
- **Prompt:** unchanged — `createPrompt` with copied title suffix `(Copy)`, folder, tags, description, and `content`.
- **Link:** `createPrompt` with the same fields plus `kind: 'link'`, `source`, and `template`; **no `content`**.

## Delete confirm

- **Prompt:** `Delete prompt "<title>"?`
- **Link:** `Delete the link "<title>"? The file on disk stays.`

Deletion still calls `promptsStore.deletePrompt` (removes the pointer file only).

## Tests

`frontend/src/views/PromptsView.test.ts` (Vitest + `@vue/test-utils`, happy-dom):

- Two list items (`kind: 'prompt'` and `kind: 'link'`) show **Prompt** and **Link** badges.
- Duplicate link triggers `createPrompt` with `kind: 'link'`, same `source`, no `content`.
- Mocked `window.confirm` for link delete includes the title and `file on disk stays`.

Run: `npm test -- src/views/PromptsView.test.ts` in `frontend/`.