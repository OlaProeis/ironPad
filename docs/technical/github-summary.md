# Dashboard GitHub summary

`GET /api/github/summary` returns per-project counts derived from on-disk snapshots only. It does not call GitHub and does not read project frontmatter beyond checking that each project directory contains `index.md`.

Handler: `github_summary` in `backend/src/routes/github.rs`, registered in `main.rs` as `/api/github/summary`.

## Eligible projects

For each subdirectory of `data/projects/` that has `index.md`:

1. `read_snapshot(project_id)` must succeed.
2. Snapshot `login` must be present and non-empty.
3. Snapshot `repo` must be non-empty (after trim).

Projects missing any of the above are omitted from the response.

## Counting rules

Matches the **Assigned to me** and **Review requested** filters (case-insensitive ASCII):

| Field | Rule |
|-------|------|
| `assigned_issues` | Issues where `assignee_logins` contains the snapshot `login` |
| `review_pulls` | Pulls where `requested_reviewer_logins` contains the snapshot `login` |

Entries are included even when both counts are zero. The frontend hides lines when both counts are zero (PRD §8).

## Response shape

JSON array of `GithubSummaryEntry`:

```json
[
  {
    "project_id": "ferrite",
    "repo": "acme/ferrite",
    "assigned_issues": 1,
    "review_pulls": 0,
    "refreshed_at": "2026-02-05T12:00:00Z"
  }
]
```

## Tests

`cargo test -- routes::github::summary` — two projects with snapshots; project B has `login: null`; project A asserts counts and case-insensitive assignee match.