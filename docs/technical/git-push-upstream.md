# Push and upstream tracking

`POST /api/git/push` pushes the current branch to `origin` with the same credential behavior as fetch. After a **successful** push, if the branch has no upstream yet, the backend sets tracking to `origin/<branch>` so `GET /api/git/remote` ahead/behind counts work.

## Behavior

`push_to_remote` in `backend/src/services/git.rs` (route in `backend/src/routes/git.rs`):

1. Requires a configured `origin` URL; failures use `map_git_failure` with `GitOp::Push`.
2. Pushes `refs/heads/<branch>:refs/heads/<branch>` (no force-push, no hook bypass).
3. If `branch.upstream()` is missing, calls `set_upstream(Some("origin/<branch>"))` on the local branch.
4. If upstream already exists, it is left unchanged.

Path-specific logic lives in `push_in(path)` so tests can run against temporary repos (same pattern as `pull_in`).

Success JSON: `{ "success": true, "message": "Successfully pushed to remote" }`.

## Relation to pull

`POST /api/git/pull` requires an upstream before it will fetch and fast-forward. Until the first successful push, pull returns the no-upstream sentence. See [`git-pull.md`](./git-pull.md).

## Tests

`cargo test push_sets_upstream` in `backend/` uses a local bare `origin` and asserts upstream name and zero ahead/behind after the first push.