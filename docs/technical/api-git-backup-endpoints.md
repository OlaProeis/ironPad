# Git backup endpoints (API reference)

Canonical HTTP shapes for Git backup in **[`API.md`](../API.md)** under **Git Operations**:

| Endpoint | Role |
|----------|------|
| `POST /api/git/remote` | Add or replace `origin`; URL validation; post-save fetch with optional `fetch_error` |
| `POST /api/git/pull` | Fetch then fast-forward only; success `commits` count |

**`API.md` includes:** request/response JSON, `validate_remote_url` rules (validation text not passed through `map_git_failure`), pull refusal table (no remote, no upstream, diverged, dirty tree), and pointers to friendly error sentences.

**Implementation detail (not duplicated in API.md):**

| Topic | Doc |
|-------|-----|
| `set_remote_origin`, URL rules | [`git-remote-origin.md`](./git-remote-origin.md) |
| `pull_from_remote` behavior | [`git-pull.md`](./git-pull.md) |
| `map_git_failure` messages | [`git-user-facing-errors.md`](./git-user-facing-errors.md) |
| UI | [`git-remote-ui.md`](./git-remote-ui.md), [`git-pull-ui.md`](./git-pull-ui.md) |

Route structs: `SetRemoteRequest` / `SetRemoteResponse`, `PullResponse` in `backend/src/routes/git.rs`.