# Task meta: `completed` field (backend)

Clients can clear completion when moving a task out of **Completed** (e.g. to Active or Backlog) via the meta endpoint, without using the toggle route. Completing a task — including spawning the next recurring instance — stays on the toggle endpoint only.

## Endpoint

`PUT /api/projects/:id/tasks/:taskId/meta` — body may include optional `completed`.

## Behavior

| Value | Effect |
|-------|--------|
| `completed: false` | Sets frontmatter `completed` to `false`. Does not auto-change `section`; clients typically send `section` in the same request when moving out of Completed. |
| `completed: true` | **Ignored.** `completed` is not set to `true` and no recurring follow-up task is created. Use `PUT .../toggle` to complete and handle recurrence. |
| Omitted | No change to `completed`. |

## Implementation

- **Request type:** `UpdateTaskMetaRequest` in `backend/src/routes/tasks.rs` — field `completed: Option<bool>`.
- **Handler:** `update_task_meta_impl` applies only `Some(false)` to frontmatter.

Toggle completion and recurrence logic remain in `toggle_task_impl`.

## Tests

`backend/src/routes/tasks.rs` — `#[cfg(test)] mod meta_tests`:

- `meta_completed_false_clears_completion`
- `meta_completed_true_ignored_no_recurrence_spawn`
- `update_task_meta_request_deserializes_completed`

```bash
cd backend && cargo test meta_tests
```

Use `IRONPAD_DATA_DIR` (set automatically in tests) for an isolated temp data directory.