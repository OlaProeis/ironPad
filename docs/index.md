# Documentation Index

This index provides an overview of all documentation for Ironpad. Attach this file to AI chats when asking questions about documentation structure.

---

## Core Documentation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`/README.md`](../README.md) | Project overview, quick start, installation | First time setup, sharing the project |
| [`/ai-context.md`](../ai-context.md) | Architecture, critical patterns, active work | Every AI chat — attach this first |
| [`/HANDOVER.md`](../HANDOVER.md) | Session handover, recent changes, next tasks | Starting a new session, continuing work |
| [`/ROADMAP.md`](../ROADMAP.md) | Feature roadmap, release planning | Planning features, checking what's next |
| [`/CHANGELOG.md`](../CHANGELOG.md) | Version history, completed features | Understanding what changed when |
| [`technical/changelog-0.4-crosswalk.md`](./technical/changelog-0.4-crosswalk.md) | `CHANGELOG` [0.4.0] vs PRD §11 phrases; `tests/test_changelog_0_4_crosswalk.py` | Editing release notes or acceptance checklist wording |

---

## Technical Documentation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`API.md`](./API.md) | REST API reference; **GitHub** route table (0.4.0); Prompts (Library) documents `kind`/`source`/`template`, `GET /api/library/browse`, `POST /api/library/open`; Git Operations documents `POST /api/git/remote` and `POST /api/git/pull` | Implementing new endpoints, frontend API calls |
| [`technical/api-git-backup-endpoints.md`](./technical/api-git-backup-endpoints.md) | Index of Git backup API sections in `API.md` and links to service/UI docs | Finding request/response docs for remote setup and pull |
| [`technical/api-library-endpoints.md`](./technical/api-library-endpoints.md) | Index of Library sections in `API.md` (`/api/prompts` link fields, `library/browse`, `library/open`) and 0.4 release doc pointers | Finding canonical Library HTTP reference after task 24 |
| [`ARCHITECTURE.md`](./ARCHITECTURE.md) | System design, service layer, data models | Understanding how the system works |
| [`backlinks.md`](./backlinks.md) | Note linking system — `/link` command, BacklinksPanel, link index | Working with backlinks, link API, or the editor |
| [`system-tray-implementation.md`](./system-tray-implementation.md) | System tray mode implementation details | Working with tray functionality |
| [`technical/task-dates.md`](./technical/task-dates.md) | Shared `isTaskOverdue` rule, `countActiveOverdueTasks`, dashboard + sidebar quick stats | Overdue counts, due-today rule, task due-date bugs |
| [`technical/task-sort.md`](./technical/task-sort.md) | `sortTopLevelTasks`, TasksView sort dropdown, `ironpad-task-sort` localStorage | Task list sort UI and ordering rules |
| [`technical/dashboard-tag-chips.md`](./technical/dashboard-tag-chips.md) | `buildTagChipSummary`, dashboard tag chip row from `allTasks`, incomplete-task counts | Dashboard tag chips row and selection |
| [`technical/dashboard-tag-filter.md`](./technical/dashboard-tag-filter.md) | Tag filter on cards, empty state when no matches, clear filter control | Dashboard tag filter, card hiding, and empty state |
| [`technical/day-budget.md`](./technical/day-budget.md) | `barFillRatio`, day-view used/budget bar + hours editor in `CalendarView`, `ironpad-day-budget-hours` (1–24h, default 6) | Calendar day view capacity bar and budget persistence |
| [`technical/task-meta-completed.md`](./technical/task-meta-completed.md) | Meta `completed: false` clears completion; `true` ignored; toggle-only for complete + recurrence | Move-out-of-completed via meta API, backend behavior |
| [`technical/task-multi-select.md`](./technical/task-multi-select.md) | `taskMultiSelect.ts`, TasksView top-level checkboxes, tag-filter prune, clear on project change | Bulk selection UI before action bar |
| [`technical/task-bulk-actions.md`](./technical/task-bulk-actions.md) | Bulk bar: sequential toggle/meta, stop on first error, reload, clear selection on success | Mark complete, move section, add/remove tag for selected tasks |
| [`technical/git-remote-origin.md`](./technical/git-remote-origin.md) | `POST /api/git/remote`, `validate_remote_url`, `set_remote_origin`, fetch-after-save without rollback | Configure or replace private `origin` from the API |
| [`technical/git-remote-url-tests.md`](./technical/git-remote-url-tests.md) | `tests::remote` table cases, temp-repo `origin` add/replace, `cargo test remote::` filter | Regression tests for remote URL validation |
| [`technical/git-credential-helper.md`](./technical/git-credential-helper.md) | HTTPS fetch/push via `credential.helper`; SSH via ssh-agent; `remote_auth_kind` | Git remote authentication by URL scheme |
| [`technical/git-user-facing-errors.md`](./technical/git-user-facing-errors.md) | `map_git_failure` turns auth, non-FF push, diverged pull, dirty tree, missing remote, and unreachable host into PRD sentences | Git JSON error text for push, fetch, pull, and remote setup |
| [`technical/git-pull.md`](./technical/git-pull.md) | `POST /api/git/pull` fetches and fast-forwards only; diverged, dirty, and no-upstream refusals | Fast-forward pull API |
| [`technical/git-push-upstream.md`](./technical/git-push-upstream.md) | `POST /api/git/push`, `push_in`; sets `origin/<branch>` upstream on first successful push | Push API and tracking branch |
| [`technical/project-github-repo.md`](./technical/project-github-repo.md) | `github_repo` frontmatter on `index.md`, optional field on `Project` / `ProjectWithContent` GET; `reqwest` and `dirs` in `Cargo.toml` | GitHub 0.4 project link field on project GET JSON |
| [`technical/project-github-routes.md`](./technical/project-github-routes.md) | `routes/github.rs`: `GET`/`PUT /api/projects/{id}/github`, snapshot read, `github_repo` validation, stale after 15m | Saving or clearing the repo link and reading cached lists |
| [`technical/github-foundations.md`](./technical/github-foundations.md) | `services/github/`: `Snapshot` serde types, `resolve_token` (`gh` then `GH_TOKEN` / `GITHUB_TOKEN`), `github_cache_dir` | GitHub backend module layout and shared types |
| [`technical/github-snapshot-io.md`](./technical/github-snapshot-io.md) | `snapshot_io.rs`: `read_snapshot` / `write_snapshot`, bodies and `CachedComment` JSON under `{cache}/bodies/` | GitHub snapshot file cache before routes and refresh |
| [`technical/github-client.md`](./technical/github-client.md) | `GithubClient` (`get_json`, `get_paginated`), default REST headers, ETag/`If-None-Match`, five-page `Link` pagination, `GithubError` → PRD §5.6; `mockito` tests | GitHub REST calls before snapshot refresh and routes |
| [`technical/github-lists.md`](./technical/github-lists.md) | `fetch_lists`, `fetch_login`, `FetchResult` and `*_304` flags; raw `serde_json::Value` → `GithubIssue`/`GithubPull`; issues page drops `pull_request` keys; team reviewers skipped | PRD §5.2 open issues/PR fetch and normalization before snapshot merge |
| [`technical/github-refresh.md`](./technical/github-refresh.md) | `POST /api/projects/{id}/github/refresh`: token, list merge, 304 `refreshed_at`, closed-item cache delete, 400/503 errors | Refreshing the GitHub snapshot from the API |
| [`technical/github-item-detail.md`](./technical/github-item-detail.md) | `GET` issue and pull detail: cached body, 30 comments, 1 MiB cap, PR `commit_status` | Reading a GitHub issue or pull request body and comments |
| [`technical/github-track.md`](./technical/github-track.md) | `POST` issue/pull `track`: snapshot-only, `create_task_impl`, per-project `github_url` dedup, labels→tags | Track a GitHub item as a local Active task |
| [`technical/github-summary.md`](./technical/github-summary.md) | `GET /api/github/summary`: snapshot-only dashboard counts, `github_summary_entries`, assignee/reviewer case-insensitive match | Dashboard assigned-issue and review-request counts per project |
| [`technical/dashboard-github-summary.md`](./technical/dashboard-github-summary.md) | `DashboardView.vue`: `githubApi.getSummary` on mount, `githubSummaryByProject`, card `router-link` to `project-github`; Vitest `DashboardView.test.ts` | GitHub assigned/review counts on dashboard project cards |
| [`technical/git-remote-ui.md`](./technical/git-remote-ui.md) | Git panel remote backup section, `gitRemoteSetup.ts`, `gitApi.setRemote` JSON on 400 | Add or replace `origin` URL in the UI |
| [`technical/git-pull-ui.md`](./technical/git-pull-ui.md) | Git panel Pull beside Fetch, `gitPull.ts`, `gitApi.pull` JSON on 400, success line + error banner | Fast-forward pull in the UI |
| [`git-backup.md`](./git-backup.md) | Private `origin`, push/pull workflow, fast-forward-only pull, credential helper vs tokens in URL, async exchange not live co-editing | Git panel **Remote backup** and sharing a `data/` mirror |
| [`github-link.md`](./github-link.md) | One `owner/name` repo per project, read-only issues and PRs, `gh auth login` or `GH_TOKEN`, separate from Remote backup, **Track as task** is a local copy | Project **GitHub** sidebar view (release **0.4.0**) |
| [`technical/github-release-docs.md`](./technical/github-release-docs.md) | Index of GitHub 0.4.0 user page, `API.md` route table, `CHANGELOG` / `CHECKLIST` blocks, PRD §10–12 | Finding GitHub release docs after task 21 |
| [`technical/prompt-pointer-frontmatter.md`](./technical/prompt-pointer-frontmatter.md) | `PromptKind`, `PromptPointerFields`, list/summary `kind`+`source`, `prepare_mapping_for_write` for prompts vs links | Library 0.4 pointer frontmatter and prompt save behavior |
| [`technical/prompt-link-create-update.md`](./technical/prompt-link-create-update.md) | `POST`/`PUT` `/api/prompts` with `kind`/`source`/`template`; project vs global validation; kind immutable on update; `is_client_error` → 400 | Creating and editing library link prompts via the API |
| [`technical/library-source-resolution.md`](./technical/library-source-resolution.md) | `Scope`, `resolve_source`, project/global `source` validation, canonical root check, `normalize_relative` | Library link path resolution before read/browse/open |
| [`technical/library-linked-file-read.md`](./technical/library-linked-file-read.md) | `read_linked_file`, `MAX_LINK_BYTES`, `ALLOWED_SUFFIXES`, `suffix_allowed`, `supports_rendered`, strict UTF-8 | Linked-file size/suffix/encoding gate after path resolution |
| [`technical/library-link-get.md`](./technical/library-link-get.md) | `GET /api/prompts/{id}?root=`, `get_prompt_for_read`, `resolve_link_by_id`, `source_missing` / `root_required` / `source_error` | Request-time link file text on single-prompt GET |
| [`technical/library-browse.md`](./technical/library-browse.md) | `GET /api/library/browse?path=`, `browse_directory`, non-recursive listing, `linkable` from suffix and size cap | Add-link directory picker API |
| [`technical/library-open.md`](./technical/library-open.md) | `POST /api/library/open`, `resolve_for_open`, `launch`, `preflight_linked_file`; id-only open via OS default app | Open linked file externally from the API |
| [`technical/prompt-semantic-search.md`](./technical/prompt-semantic-search.md) | `semantic_search`, `prompt_for_semantic_search`, link scoring on `source`/`folder` without `read_linked_file`; list `q` includes link `source` | Library prompt search and semantic endpoint |
| [`technical/frontend-library-api.md`](./technical/frontend-library-api.md) | `PromptKind`, link fields on `Prompt`, `libraryApi.browseLibrary` / `openLibraryItem`, `promptsApi.get(id, root?)`; vitest `client.library.test.ts` | Frontend types and API client before library UI/store work |
| [`technical/github-frontend-api.md`](./technical/github-frontend-api.md) | GitHub types in `types/index.ts`, `githubApi` in `client.ts` (snapshot, repo, refresh, detail, track, summary); vitest `client.github.test.ts` | Frontend typed GitHub HTTP layer before GitHub UI/store work |
| [`technical/github-store.md`](./technical/github-store.md) | `useGithubStore` in `stores/github.ts`: snapshot load/refresh, `ironpad-github-issue-filter` / `ironpad-github-pr-filter`, `filteredIssues` / `filteredPulls`; Vitest `stores/__tests__/github.test.ts` | GitHub list filter state before GitHub panel UI work |
| [`technical/github-view-scaffold.md`](./technical/github-view-scaffold.md) | Route `project-github`, sidebar **★ GitHub**, `GitHubView.vue` loads snapshot and toggles `RepoLinkForm` vs `IssuesList`/`PullsList` stubs; Vitest `Sidebar.test.ts`, `GitHubView.test.ts` | Per-project GitHub screen entry point before link form and lists |
| [`technical/github-view-refresh.md`](./technical/github-view-refresh.md) | `GitHubView.vue`: stale-on-load refresh, header Refresh + spinner, 15-minute interval cleared on unmount, `refreshed_at` local time, dismissable error banner; Vitest fake timers in `GitHubView.test.ts` | GitHub panel refresh UX and while-view-is-open polling |
| [`technical/repo-link-form.md`](./technical/repo-link-form.md) | `RepoLinkForm.vue`: PRD `owner/name` validation, `putRepo` + store `refresh`, remove with confirm; Vitest `RepoLinkForm.test.ts` | Link or unlink the project GitHub repo from the empty-state form |
| [`technical/github-issues-list.md`](./technical/github-issues-list.md) | `IssuesList.vue`: issue filter tabs, store-driven rows, truncation/empty state, `relativeTime.ts`, `select` emit; Vitest `IssuesList.test.ts` | GitHub open-issues panel before issue detail pane |
| [`technical/github-pulls-list.md`](./technical/github-pulls-list.md) | `PullsList.vue`: PR filter tabs, Draft badge, store-driven rows, truncation/empty state, `select` emit; Vitest `PullsList.test.ts` | GitHub open-pull panel before pull detail pane |
| [`technical/github-item-detail-pane.md`](./technical/github-item-detail-pane.md) | `ItemDetailPane.vue`: cached `getIssue`/`getPull`, rendered body and comments, PR commit status, Open on GitHub and Copy link; `GitHubView` `selectedItem`/`selectedKind` | Read-only GitHub issue or pull side pane |
| [`technical/github-track-button.md`](./technical/github-track-button.md) | `TrackButton.vue`: `trackIssue`/`trackPull`, **Track as task** / **Open task**; wired in `IssuesList` rows and `ItemDetailPane`; Vitest `TrackButton.test.ts` | Track GitHub items as local tasks from list or detail |
| [`technical/library-ui-labels.md`](./technical/library-ui-labels.md) | Top bar and sidebar **Library** labels, `Global Library` title, `PromptsView` heading; routes/API unchanged; `Sidebar.test.ts` | Renaming Prompts → Library in navigation only |
| [`technical/library-root.md`](./technical/library-root.md) | `libraryRoot.ts`, `ironpad-library-root:<projectId>`, `LibraryRootBar` on `project-prompts`; `isInsideRoot` / `toRelativeSource` | Set, show, and clear per-project coding-repo root in the Library UI |
| [`technical/prompts-store-links.md`](./technical/prompts-store-links.md) | `prompts.ts`: `getLibraryRoot` on project GET, `reloadOpen`, focus refresh for links only; Vitest `prompts.links.test.ts` | Opening/reloading link content in the Library store |
| [`technical/prompts-view-link-list-actions.md`](./technical/prompts-view-link-list-actions.md) | `PromptsView.vue` list `Prompt`/`Link` badges, duplicate link without `content`, link delete confirm; Vitest `PromptsView.test.ts` | Library list kind labels and link duplicate/delete UX |
| [`technical/link-reading-pane.md`](./technical/link-reading-pane.md) | `LinkPane.vue`: read-only `<pre>`, Copy path, Open externally (`libraryApi`), Relink, root bar, source error, Reload; path actions disabled on link errors; save omits `content` | Open-link reading pane and error states |
| [`technical/add-link-dialog.md`](./technical/add-link-dialog.md) | `AddLinkDialog.vue`: browse, paste path (`isInsideRoot` / `toRelativeSource`), multi-file confirm; **Add link** on the Library toolbar | Creating library links from browse or typed path |
| [`technical/library-suggest-files.md`](./technical/library-suggest-files.md) | `librarySuggest.ts`, `SuggestFilesPanel.vue`, browse `root` + `root/docs` for well-known files, dismiss localStorage, bulk **Add selected** | Suggested `ai-context.md`, `ROADMAP.md`, `docs/index.md` links after a project root is set |
| [`technical/rendered-markdown.md`](./technical/rendered-markdown.md) | `markdownBody.ts` (`stripFrontmatter`, `prepareMarkdownBody`), `RenderedMarkdown.vue` (markdown-it, `html: false`); Vitest `markdownBody.test.ts`, `RenderedMarkdown.test.ts` | Library read-only rendered HTML preview and frontmatter strip |
| [`technical/reading-mode.md`](./technical/reading-mode.md) | `readingMode.ts`, `ironpad-library-mode:<itemId>`, PromptsView Raw/Rendered; Save uses the raw draft; variable substitution before render | Library reading-mode toggle, defaults, and persistence |
| [`technical/library-versions-panel.md`](./technical/library-versions-panel.md) | `ironpad-library-versions-expanded`, `PromptsView` Show/Hide, full-width grid; prompts use `logByPath`, links show outside-repo line | Collapsible Git versions column in the Library |
| [`technical/link-template-toggle.md`](./technical/link-template-toggle.md) | Link **Template** checkbox, `templateVariablesEnabled`, `updatePrompt({ template })` only; Copy verbatim vs Copy Rendered; Vitest `PromptsView.test.ts` | Opting links into `{{variable}}` UI and copy behavior |
| [`technical/library-copy-bundle.md`](./technical/library-copy-bundle.md) | `copyBundle.ts` `buildBundle`; PromptsView checkboxes and **Copy bundle**; raw text, link `resolved_path`, no disk write | Multi-select library items into one markdown clipboard blob |

---

## AI Workflow Documentation

Documentation about the AI-assisted development process used to build Ironpad:

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`ai-workflow/README.md`](./ai-workflow/README.md) | Overview of AI workflow methodology | Understanding the development process |
| [`ai-workflow/PRD.md`](./ai-workflow/PRD.md) | Product requirements document | Understanding original feature specifications |
| [`ai-workflow/PRD-0.4.0.md`](./ai-workflow/PRD-0.4.0.md) | PRD for release 0.4.0 task list, dashboard, and Git backup. Released 2026-11-20 | Task, dashboard, and Git backup behavior |
| [`ai-workflow/PRD-0.4.0-library.md`](./ai-workflow/PRD-0.4.0-library.md) | PRD for the 0.4.0 Library: linked files, rendered or raw, collapsible versions, copy bundle. Released 2026-11-20 | Library behavior, acceptance checklist §11 |
| [`ai-workflow/PRD-0.4.0-github.md`](./ai-workflow/PRD-0.4.0-github.md) | PRD for the 0.4.0 GitHub link: one repo per project, read-only issues and pull requests, track as a local task. Released 2026-11-20 | GitHub link behavior, acceptance checklist §12 |
| [`ai-workflow/method.md`](./ai-workflow/method.md) | Detailed workflow method | Implementing similar AI workflows |
| [`ai-workflow/CHECKLIST.md`](./ai-workflow/CHECKLIST.md) | Implementation checklist; **Release 0.4.0** mirrors PRD-0.4.0 §11, the library PRD acceptance list, and the GitHub PRD acceptance list | Tracking feature completion and 0.4 acceptance |
| [`ai-workflow/checklist-v0.4.md`](./ai-workflow/checklist-v0.4.md) | How CHECKLIST 0.4 sections map to PRD-0.4.0 §11, the library PRD, and the GitHub PRD, and when to mark items done | Updating 0.4 checklist rows or PRD acceptance text |
| [`ai-workflow/planning-docs-0.4.md`](./ai-workflow/planning-docs-0.4.md) | How `ROADMAP.md` and `ai-context.md` point at the three 0.4 PRDs. Released 2026-11-20 | Aligning roadmap or agent context with 0.4 scope |
| [`ai-workflow/tools.md`](./ai-workflow/tools.md) | Tools and setup | Setting up development environment |
| [`ai-workflow/lessons-learned.md`](./ai-workflow/lessons-learned.md) | Development insights | Avoiding past mistakes, understanding decisions |
| [`ai-workflow/HANDOVER.md`](./ai-workflow/HANDOVER.md) | Archived handover notes | Historical session context |

---

## Frontend Documentation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`/frontend/README.md`](../frontend/README.md) | Frontend architecture, component structure | Working on Vue components, Milkdown editor |

---

## Documentation Guidelines

### When Adding New Documentation

1. **Technical docs** → Add to `docs/` folder, update this index
2. **Architecture changes** → Update `ai-context.md` AND this index
3. **API changes** → Update `docs/API.md`
4. **Feature specs** → Add to `docs/ai-workflow/` with PRD format
5. **Workflow/process docs** → Add to `docs/ai-workflow/`

### File Naming Conventions

- Use lowercase with hyphens: `api-reference.md`
- README files for folder overviews
- Match document purpose to filename

---

## Quick Reference

**Starting a new feature:**
1. Read `ROADMAP.md` for planned features
2. Read `HANDOVER.md` for current context
3. Read `ai-context.md` for architecture
4. Check `docs/API.md` if adding endpoints

**Debugging an issue:**
1. Read `ai-context.md` for known issues
2. Read `ARCHITECTURE.md` for system flow
3. Read relevant component docs

**Adding documentation:**
1. Choose the right folder (docs/ or docs/ai-workflow/)
2. Update this index
3. Link from relevant files
