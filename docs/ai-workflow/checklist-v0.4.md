# Release 0.4.0 checklist tracking

The **Release 0.4.0** section in [`CHECKLIST.md`](./CHECKLIST.md) is the execution status for the 0.4 release. Those acceptance rows are checked: all three 0.4 PRDs are implemented in the working tree. The release is not shipped. The GitHub release is still 0.3.0. Phase 5 and Phase 6 in the same file stay open. There is no 0.5 PRD yet.

Through **Git backup**, the checklist mirrors [`PRD-0.4.0.md`](./PRD-0.4.0.md) **§11 (Acceptance checklist)** line for line. The Library sections mirror [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md) **§11** line for line. The GitHub sections mirror [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md) **§12** line for line.

## Structure

| CHECKLIST subsection | Acceptance source | Notes |
|---------------------|-------------------|-------|
| Port (already done) | PRD-0.4.0 §11 Port | Only items shipped before 0.4 feature work; keep checked when verified |
| Task list and dashboard | PRD-0.4.0 §11 Task list and dashboard | Sort, overdue parity, tag filter, day budget, bulk actions |
| Git backup | PRD-0.4.0 §11 Git backup | Remote setup, pull, user-facing errors, ahead/behind, `docs/git-backup.md` link |
| Library label | PRD-0.4.0-library.md §11 Library label | Top bar and sidebar say Library; routes stay |
| Linked files | PRD-0.4.0-library.md §11 Linked files | Pointers, folder root, refresh, save, delete, relink, limits, variables |
| Rendered and raw | PRD-0.4.0-library.md §11 Rendered and raw | Toggle, save writes raw, markdown default, remembered mode |
| Versions | PRD-0.4.0-library.md §11 Versions | Collapse, prompt log, links have no Ironpad history |
| Copy bundle, suggest, open | PRD-0.4.0-library.md §11 Copy bundle, suggest, open | Bundle text, suggested filenames, open and copy path |
| Link | PRD-0.4.0-github.md §12 Link | `owner/name` on the project, validation, remove, `origin` unchanged |
| Lists and refresh | PRD-0.4.0-github.md §12 Lists and refresh | Separate lists, filters, 15-minute refresh, 304, failed refresh, snapshot outside `data/` |
| Login | PRD-0.4.0-github.md §12 Login | `gh auth token`, then `GH_TOKEN` or `GITHUB_TOKEN`, token never stored |
| Read, track, dashboard | PRD-0.4.0-github.md §12 Read, track, dashboard | Read-only body, status word, one local task, dashboard counts |

## Completion rules

- Check an item only after the behavior is **implemented and verified** (same rule as the global checklist footer).
- When a PRD acceptance list changes, update the matching checkbox lines in `CHECKLIST.md`. Do not invent parallel criteria.
- Feature implementation docs live under `docs/technical/`. This page only describes how checklist rows map to the PRDs.

## Related

- Task, dashboard, and Git spec: [`PRD-0.4.0.md`](./PRD-0.4.0.md)
- Library spec: [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md)
- GitHub spec: [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md)
- User-facing Git backup overview: [`../git-backup.md`](../git-backup.md)
