# GitHub TrackButton (frontend)

Release 0.4.0 adds a small control to create or reopen a local Ironpad task from a GitHub issue or pull request number.

## Component

Module: `frontend/src/components/github/TrackButton.vue`

| Prop | Type | Meaning |
|------|------|---------|
| `projectId` | `string` | Active project |
| `number` | `number` | GitHub issue or PR number |
| `kind` | `'issue' \| 'pull'` | Chooses `trackIssue` vs `trackPull` |

Local state:

- `tracking` — spinner while the POST is in flight
- `tracked` — after a successful track response (`created` true or false)
- `existingTaskId` — `TrackResult.task_id` from the API

### UI

- Before track: **Track as task** (`data-testid="track-as-task"`)
- While tracking: disabled button with spinner (`data-testid="track-spinner"`)
- After track: **Open task** (`data-testid="open-task"`)

Both `created: true` (new task) and `created: false` (already tracked) show **Open task**.

### Track flow

Calls `githubApi.trackIssue` or `githubApi.trackPull` (see [`github-frontend-api.md`](./github-frontend-api.md)). Does not refresh the GitHub snapshot.

### Open task navigation

```ts
router.push({
  name: 'project-tasks',
  params: { id: projectId },
  query: { task: existingTaskId },
})
```

`TasksView` primarily selects tasks via the optional `taskId` route param (`/projects/:id/tasks/:taskId?`). The query shape above matches the task handover and Vitest expectations; align with `params.taskId` if “Open task” should focus a row the same way as the dashboard.

## Wiring

| Location | Placement |
|----------|-----------|
| `IssuesList.vue` | One `TrackButton` per issue row (`kind="issue"`), beside the row button; `@click.stop` on the track control |
| `ItemDetailPane.vue` | Below the rendered body (inside the loaded `detail` block); `kind` from the pane prop |

`PullsList.vue` is unchanged in this task (pull rows can adopt the same pattern later).

## Tests

`frontend/src/components/github/TrackButton.test.ts` — Vitest with mocked `githubApi` and `useRouter`:

1. Issue kind calls `trackIssue`
2. `created: true` → **Open task**
3. `created: false` → **Open task**
4. **Open task** calls `router.push` with `project-tasks` and `query.task`
5. Pull kind calls `trackPull`

Run: `cd frontend && npm run test -- TrackButton`

## Related

- Backend track endpoints: [`github-track.md`](./github-track.md)
- Issue list: [`github-issues-list.md`](./github-issues-list.md)
- Detail pane: [`github-item-detail-pane.md`](./github-item-detail-pane.md)