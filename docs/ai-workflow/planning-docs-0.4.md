# Planning docs for release 0.4.0

Release **0.4.0** shipped on 2026-11-20. No 0.4 PRD is left to implement. The next scope is the 0.5.0 proposal in [`ROADMAP.md`](../../ROADMAP.md), which has no PRD yet.

Scope lives in three PRDs, all implemented:

- [`PRD-0.4.0.md`](./PRD-0.4.0.md) — task list, dashboard, and Git backup.
- [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md) — Library.
- [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md) — GitHub issues and pull requests. Separate from Git backup.

Top-level planning files point implementers at those PRDs so feature names stay consistent across the repo.

## ROADMAP.md

The **Release 0.4.0 (Released)** section lists all three parts.

Task, dashboard, and Git backup use the same titles as PRD-0.4.0 sections 4–9:

| PRD section | ROADMAP heading |
|-------------|-----------------|
| §4 | Sort the task list |
| §5 | Overdue count in the sidebar |
| §6 | Dashboard filter by task tag |
| §7 | Fit the day |
| §8 | Bulk actions |
| §9 | Private Git remote, finished in the app |

The Library uses the same titles as PRD-0.4.0-library.md:

| PRD section | ROADMAP heading |
|-------------|-----------------|
| §3 | Library label |
| §4 | Linked files |
| §5 | Rendered or raw |
| §6 | Versions (Git) |
| §7 and §4.4 | Copy bundle and suggest files |

The GitHub link uses the same titles as PRD-0.4.0-github.md:

| PRD section | ROADMAP heading |
|-------------|-----------------|
| §4 | Link |
| §5 | Lists |
| §6 and §7 | Read and track |
| §8 | Dashboard |

**Fit the day**, the **Library**, and the **GitHub link** are on 0.4 and are implemented in the working tree. **Task templates** stay unscheduled (called out under 0.4 and in **Suggested features → Still unscheduled**). Server port behavior is under **Server port** in the 0.4.0 section. The tagged 0.3.0 release still scans 3000–3010. The GitHub link is not the private `origin` in PRD-0.4.0 §9.

Runnable prompts, the weekly review, `waiting_on`, the capture strip, and project blueprints stay on the 0.5.0 proposal.

## ai-context.md

Agents and humans use `ai-context.md` for workflow rules and where artifacts live. For 0.4:

- **Architecture & Data Model** says 0.4.0 is the current GitHub release and links all three PRDs.
- **Where Things Live** includes a row for 0.4 scope → all three PRDs.

Long-term product vision remains in root `prd.md`. The three 0.4 PRDs are the spec for the unreleased 0.4.0 tree. The 0.5.0 proposal has no PRD yet.

## PRD artifacts

| Artifact | Role for 0.4 |
|----------|----------------|
| [`CHECKLIST.md`](./CHECKLIST.md) | 0.4 acceptance rows are checked. Task/Git rows mirror PRD-0.4.0 §11. Library rows mirror PRD-0.4.0-library.md §11. GitHub rows mirror PRD-0.4.0-github.md §12. Phase 5 and Phase 6 stay open. See [`checklist-v0.4.md`](./checklist-v0.4.md) |
| `CHANGELOG.md` | Undated `## [0.4.0]` for task, dashboard, Git backup, Library, and GitHub notes. No release date until the Ironpad release is tagged |
| [`docs/index.md`](../index.md) | All three PRDs, plus technical docs as features land |

## Related

- Task/Git acceptance: PRD-0.4.0 §11
- Library acceptance: PRD-0.4.0-library.md §11
- GitHub acceptance: PRD-0.4.0-github.md §12
- Checklist mapping: [`checklist-v0.4.md`](./checklist-v0.4.md)
