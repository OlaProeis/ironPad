# GitHub 0.4.0 release documentation

Pointers for the GitHub half of release **0.4.0** (undated in `CHANGELOG` until tag).

| Artifact | Location | Role |
|----------|----------|------|
| User guide | [`../github-link.md`](../github-link.md) | `owner/name`, read-only, `gh auth login` / `GH_TOKEN`, vs Remote backup, Track as task |
| HTTP reference | [`../API.md`](../API.md) § **GitHub** | Eight-route table (`{id}` paths) plus endpoint detail; all **Added in 0.4.0** |
| Release notes | [`../../CHANGELOG.md`](../../CHANGELOG.md) `## [0.4.0]` → `### GitHub` | Shipped-feature bullets for the GitHub screen |
| Acceptance | [`../ai-workflow/CHECKLIST.md`](../ai-workflow/CHECKLIST.md) Release 0.4.0 → Link / Lists / Login / Read, track, dashboard | Mirrors [`PRD-0.4.0-github.md`](../ai-workflow/PRD-0.4.0-github.md) §12 |
| Product spec | [`../ai-workflow/PRD-0.4.0-github.md`](../ai-workflow/PRD-0.4.0-github.md) | Routes §10, doc list §11, checklist §12 |

In-app link from **GitHub** view to the user guide is task 22 (`GIT_BACKUP_DOC_HREF` pattern in [`git-remote-ui.md`](./git-remote-ui.md)).