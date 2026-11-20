# Ironpad - AI Context

This file is **whole-project** agent memory across all phases and task tags. It is not phase-scoped. Update it additively; never rewrite it for the current phase alone.

## Rules (DO NOT UPDATE)
- **Implementation sessions:** follow **Implementation Phase Rules** below only.
- **Update sessions:** follow **Update Phase Rules** below only when you receive the update handover prompt.
- Only do the task specified; do not start the next task or go over scope.
- **Test policy:** run **only targeted tests** for what you changed (a single test binary, module, or test-name filter), within the handover's **Test & Resource Budget**. The full `python -m pytest` is run by cyclopsctl once after the implementation phase — the handover's Test & Resource Budget block says whether you may run it yourself at all. Never run any suite in the update phase. No VMs, containers, e2e/soak/benchmark suites, or background servers unless the task's Test Strategy names them as the deliverable.
- Follow existing code patterns and conventions.
- Use Context7 MCP to fetch library documentation when needed (resolve library ID first, then fetch docs). Task operations use **`cyclopsctl tasks` CLI only**.

## Implementation Phase Rules
When working from **`current-handover-prompt.md`** (the normal case for every cyclopsctl task cycle):

- **DO:** Implement and test only the current parent task described in the handover.
- **DO:** Meet the task Test Strategy within the handover's **Test & Resource Budget** using targeted tests only; follow the handover's rule about the full `python -m pytest` (by default cyclopsctl runs it for you). Anything under "Manual Verification" is for a human — skip it.
- **DO:** Use Context7 MCP for up-to-date library documentation when implementing unfamiliar APIs or frameworks.
- **DO NOT:** Read `prd.md` during cyclopsctl cycles — task scope, details, and test strategy are already in this handover.
- **DO NOT:** Mark tasks done or change task status.
- **DO NOT:** Run `cyclopsctl tasks next`, rewrite `current-handover-prompt.md`, or edit `ai-context.md`.
- **DO NOT:** Create or update docs in `docs/`, or edit `docs/index.md`.
- **DO NOT:** Edit `update-handover-prompt.md`.

Task completion and all documentation updates happen only in the **update phase** (`update-handover-prompt.md`).

## Update Phase Rules
When `update-handover-prompt.md` is provided (after implementation in the same agent session):

- **DO:** Follow every step in `update-handover-prompt.md`.
- **DO:** Mark the current task done, update docs, and append durable facts with `cyclopsctl memory add --project-root G:\DEV\proman2k`.
- **DO NOT:** Rewrite `current-handover-prompt.md`. Cyclopsctl writes the next handover from `cyclopsctl tasks next` after the update phase.
- **DO NOT:** Store facts in `ai-context.md`. Facts live in `.cyclopsctl/memory/facts.jsonl` via `cyclopsctl memory add`.
- **DO NOT:** Re-run `python -m pytest` or any test suite in the update phase.
- **DO:** Use `cyclopsctl tasks` with `--project-root G:\DEV\proman2k` for all task commands (see Environment in the handover).
- **DO:** Document by feature (e.g., `auth-layer.md`), not by task number; update `docs/index.md` when adding documentation.
- **DO NOT:** Re-implement or extend the task you just finished.
- **DO NOT:** Rewrite `ai-context.md` from scratch or treat it as phase-only context.

## Conventions
- **Documentation:** Feature-based names in `docs/` (e.g., `auth-layer.md`), not `task-1.md`. Update `docs/index.md` in the update phase only.
- **Tasks:** `cyclopsctl tasks` CLI only from agents.

## Handover Files
| File | Who may edit | When |
|------|----------------|------|
| `current-handover-prompt.md` | Cyclopsctl | After the update phase, from `tasks next` |
| `update-handover-prompt.md` | Human / template only | Never edited by agents |
| `ai-context.md` | Update-phase agent only | Only when architecture/conventions change (additive); durable facts go to `cyclopsctl memory add`, not here |

## Tech Stack
![Tech Stack](docs/graphics/tech-stack.png), | Component | Technology |, |-----------|------------|, | Backend | Rust, Axum 0.8, Tokio |, | Frontend | Vue 3, Vite, TypeScript |, | Editor | Milkdown (ProseMirror-based) |, | State | Pinia |, | Routing | Vue Router |, | Data | Markdown + YAML frontmatter |, | Version Control | Git (via git2) |, | Search | ripgrep |

## Architecture & Data Model
See `prd.md` for product architecture. This file captures agent workflow rules and where project artifacts live.

Release **0.4.0** is the current GitHub release (2026-11-20). All three 0.4 PRDs are done: task list, dashboard, and Git backup ([`docs/ai-workflow/PRD-0.4.0.md`](docs/ai-workflow/PRD-0.4.0.md)), Library ([`docs/ai-workflow/PRD-0.4.0-library.md`](docs/ai-workflow/PRD-0.4.0-library.md)), and GitHub issues and pull requests ([`docs/ai-workflow/PRD-0.4.0-github.md`](docs/ai-workflow/PRD-0.4.0-github.md)). The GitHub link is separate from Git backup of `data/`. There is no 0.5 PRD yet; the next scope is the proposal in [`ROADMAP.md`](ROADMAP.md). Notes are the `## [0.4.0] - 2026-11-20` section of [`CHANGELOG.md`](CHANGELOG.md).

## Where Things Live
| Want to... | Look in... |
|------------|------------|
| Product requirements | `prd.md` |
| Release 0.4.0 scope and acceptance | `docs/ai-workflow/PRD-0.4.0.md` (task list, dashboard, Git backup), `docs/ai-workflow/PRD-0.4.0-library.md` (library), and `docs/ai-workflow/PRD-0.4.0-github.md` (GitHub issues and pull requests). Released 2026-11-20. Notes in `CHANGELOG.md`. Next proposal is 0.5.0 in `ROADMAP.md` (no PRD yet) |
| Current implementation handover | `current-handover-prompt.md` |
| Post-task update rules | `update-handover-prompt.md` |
| Documentation map | `docs/index.md` |
| Tasks and complexity | `.cyclopsctl/tasks/tasks.json`, `.cyclopsctl/reports/complexity-report.json` |
| Cyclopsctl config | `cyclopsctl.toml` |
