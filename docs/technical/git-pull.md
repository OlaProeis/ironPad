# Fast-forward pull

`POST /api/git/pull` fetches `origin`, then fast-forwards the current branch to its upstream. It does not merge, rebase, or open a conflict UI. Open views refresh through the existing file watcher.

## Behavior

`pull_from_remote` in `backend/src/services/git.rs` (route in `backend/src/routes/git.rs`):

1. No `origin` returns the no-remote sentence and does not fetch.
2. A branch with no upstream returns `No upstream yet. The first successful push sets it.` and does not fetch. The first successful push sets upstream; see [`git-push-upstream.md`](./git-push-upstream.md).
3. Otherwise it fetches with the same credential callback as `fetch_from_remote`.
4. If `merge_analysis` is up to date, the response is `Already up to date.` with `commits: 0`.
5. If the histories are not a fast-forward, the response is the diverged sentence and the branch and worktree stay put.
6. A fast-forward checks out the upstream tree with a safe checkout (local edits that would be overwritten are refused with the dirty-tree sentence), then moves `refs/heads/<branch>` to the upstream commit.

Success JSON: `{ "success": true, "message": "Pulled N commit(s).", "commits": N }`. Refusals are HTTP 400 with `success: false` and `commits: 0`. Fetch and checkout failures use `map_git_failure` with `GitOp::Pull`. The no-upstream sentence is returned directly so it is not rewritten as a missing remote.

Full HTTP request/response and error table: [`API.md`](../API.md) (**Pull (fast-forward only)**). UI: [`git-pull-ui.md`](./git-pull-ui.md).

## Tests

`cargo test pull_` in `backend/` uses temporary local clones: fast-forward, diverged history with an unchanged tree, and no upstream.
