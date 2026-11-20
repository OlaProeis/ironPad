# Collapsible Versions (Git) panel

The **Versions (Git)** column in `PromptsView.vue` can be hidden so the reading pane uses full width. State is global (not per item), same pattern as folder filters.

Related: [`reading-mode.md`](./reading-mode.md), [`link-reading-pane.md`](./link-reading-pane.md).

## Persistence

- **Key:** `ironpad-library-versions-expanded`
- **Default:** collapsed (`false` when missing or not `'true'`)
- Toggle **Show** / **Hide** in the `versions-header` row (label **Versions (Git)**) writes the boolean on each change.

## Layout

- Collapsed: `content-grid--full-width` (`grid-template-columns: 1fr`); `versions-panel` is not mounted.
- Expanded: two-column grid (`2fr 1fr`) with `history-panel` as before for prompts.

Collapsing does **not** clear `selectedCommitId` or `selectedCommitDiff`; expanding a prompt again shows the same selected commit.

## Prompt vs link

| Open item | Expanded panel |
|-----------|------------------|
| **Prompt** | `gitApi.logByPath(prompt.path, 30)` on select (not for links); commit list and per-file diff unchanged |
| **Link** | Single line: `This file is outside Ironpad's repository.` — `loadHistoryForPrompt` returns without calling Git |

The rendered-variable side preview (when template variables are visible) still lives inside the versions panel when expanded.

## Tests

`frontend/src/views/PromptsView.test.ts` — describe **collapsible Versions (Git)**:

- Fresh storage → no panel, full-width class
- Expand → log visible, `localStorage` `true`
- Select commit → collapse → expand → same active commit
- Link expanded → outside-repo notice, `logByPath` not called

```bash
npx vitest run src/views/PromptsView.test.ts
```