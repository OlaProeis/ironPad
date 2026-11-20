# Product Requirements Document — Ironpad v0.4.0

**Version:** 0.4.0  
**Date:** 2026-11-19  
**Status:** Released 2026-11-20.  
**Parent context:** [`PRD.md`](./PRD.md) (original product vision), [`ROADMAP.md`](../../ROADMAP.md), library half in [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md), GitHub issues and pull requests in [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md)

---

## 1. Summary

Release **0.4.0** makes the task list and the dashboard easier to use, and finishes private Git backup inside the app.

Five items are small because the data is already loaded. `allTasks` is fetched on startup, task tags and `estimated_minutes` are already on each task, and the calendar already sums estimates per day. The one heavy item is Git: push, fetch, and ahead/behind already exist, and pull and “add a remote” do not.

The server port work from the previous 0.4 draft is already on main. It stays in this document as shipped behavior, not as new work.

This release does not introduce multi-user real-time editing, a hosted sync service, or a local model.

**Release status:** shipped 2026-11-20. [`CHANGELOG.md`](../../CHANGELOG.md) lists the features under `## [0.4.0] - 2026-11-20`.

**Also in 0.4.0:** the Library (linked files, rendered or raw, collapsible versions, copy bundle), specified in [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md), and a read-only GitHub issues and pull requests link, specified in [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md). This document does not repeat that scope.

---

## 2. Goals

| Goal | Success criteria |
|------|------------------|
| Find the right task faster | From a project task list, the user can sort by created date, due date, or priority, and the choice survives a reload. |
| See overdue work without opening the dashboard | The sidebar shows an overdue count for the active project, using the same calendar-day rule as the dashboard. |
| Slice work by tag across projects | On the dashboard, one task tag filters every project card. |
| See whether today fits | Day view shows estimated minutes against a daily budget and turns amber when the day is over budget. |
| Change many tasks at once | On the open project, the user can complete, move section, or add/remove a tag on several top-level tasks. |
| Back up `data/` without a terminal | The user can set a private `origin` URL, push, and fast-forward pull from the Git panel, and can understand auth and divergence errors. |
| Read repo files beside prompts | Specified in [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md). A project can link a file and show its current contents. |
| See a repo’s open issues and pull requests | Specified in [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md). A project can name `owner/name` and show a read-only snapshot. |

### Non-goals (this document)

The Library is in 0.4.0 and is specified in [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md). The GitHub issues and pull requests link is in 0.4.0 and is specified in [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md). Neither is one of the exclusions below.

- Creating a GitHub, GitLab, or Gitea repo from Ironpad. No token stored for the notes remote, and no OAuth. Login for the issues link is specified in the GitHub PRD.
- Merge commits, rebase, or conflict editing inside Ironpad. Pull is fast-forward only.
- Storing Git passwords or tokens in Ironpad or in the remote URL.
- Extracting `#tags` from note or task body text. Only frontmatter `tags` on tasks.
- Task templates, project blueprints, `waiting_on`, weekly review files.
- Running a prompt against a note or a local model.
- Global capture hotkey, graph view, kanban, PDF/HTML export, custom themes, UI polish.
- A batch task API. Bulk actions call the existing per-task endpoints.
- Binding to a new port or a config-file port pin. The 37891–37990 scan and `IRONPAD_PORT` stay as they are.

---

## 3. Already shipped: server port

Kept here so the release notes stay honest.

1. **Default scan** — **37891–37990** on **127.0.0.1** (`backend/src/main.rs`). IPv6 is not used as a way around port conflicts.
2. **Pinned port** — `IRONPAD_PORT` binds one port and fails if it is busy.
3. **Startup log** — the process logs the chosen URL.

Out of scope: a config file, binding `0.0.0.0`, and binding port `0`.

---

## 4. Feature: Sort the task list

### 4.1 Problem

`list_project_tasks_impl` returns tasks newest-created first. The task view has no other order. Due date and priority are on the task and unused for sorting.

### 4.2 Requirements

1. The project task view (`TasksView`) gets a sort control with three modes:
   - **Created** — newest `created` first. This is the default and matches the API today.
   - **Due date** — earliest `due_date` first. Tasks with no due date go last. Within the same date, keep created-desc as the tie break.
   - **Priority** — `high`, then `medium`, then `low`, then unset. Within the same rank, keep created-desc.
2. Sort **top-level tasks only**. Subtasks stay under their parent in the order they already have.
3. The same mode applies inside Active, Backlog, and Completed.
4. Persist the mode in `localStorage`. A new browser profile starts on Created.
5. No API or file-format change. The backend keeps returning created-desc.

### 4.3 Edge cases

- An unrecognized stored value falls back to Created.
- Priority comparison is case-insensitive. Any other string sorts with unset.
- `due_date` is compared as a calendar date string (`YYYY-MM-DD`), not as a timestamp.

---

## 5. Feature: Overdue count in the sidebar

### 5.1 Problem

Dashboard project cards already show an overdue count. The sidebar quick stats only show active and completed, so the count is invisible while you are inside a project.

### 5.2 Requirements

1. The sidebar quick stats for the **active project** gain an overdue figure.
2. Clicking it opens that project’s task list.
3. Hide the figure when the count is zero.
4. **One date rule**, shared by the sidebar and the dashboard cards:
   - A task is overdue when it is not completed, it has a `due_date`, and that calendar date is **before today in the local timezone**.
   - A task due today is not overdue.
   - Compare dates, not instants. `new Date("YYYY-MM-DD") < new Date()` is not acceptable: it treats the date as UTC midnight and can mark today as overdue.
5. Count the task once, from its stored `due_date`. Do not expand recurrence into extra overdue rows.
6. No new endpoint. Use the tasks already loaded for the project.

### 5.3 Edge cases

- A missing or unparseable `due_date` is not overdue.
- Completed tasks are never overdue, even with an old due date.

---

## 6. Feature: Dashboard filter by task tag

### 6.1 Problem

Tags can be filtered inside one project. The dashboard already has every task in `allTasks` and shows up to two tags on each row, but there is no way to see one tag across projects.

### 6.2 Requirements

1. The dashboard shows a row of chips, one per distinct task tag across all projects, sorted A–Z, each with a count of incomplete tasks that carry it.
2. One selected tag at a time. Clicking the active chip clears the filter.
3. While a tag is selected:
   - Each project card lists only incomplete tasks that have that tag.
   - Active, backlog, and overdue counts on the card are recomputed from that subset.
   - Hide cards with no matching tasks.
4. If no task matches, show an empty state that names the tag and offers a clear control.
5. Tags come from task frontmatter `tags` only. Do not scan note bodies, prompt bodies, or task markdown.
6. No new endpoint.

### 6.3 Edge cases

- Tag match is exact and case-sensitive, consistent with the project task filter.
- A task with several tags appears for each of those chips.
- Completed tasks do not appear in the card list and do not contribute to chip counts.

---

## 7. Feature: Fit the day

### 7.1 Problem

`estimated_minutes` is stored on tasks, and month/week/day cells already show a summed estimate (`getTotalEstimateForDate`). Nothing says whether that sum fits a working day.

### 7.2 Requirements

1. Day view shows **used / budget** and a bar.
2. Default budget is **6 hours** (360 minutes).
3. The user can change the budget from day view, in hours, with a minimum of 1 and a maximum of 24. Persist it in `localStorage`.
4. **Used** is the existing sum: `estimated_minutes` for every task occurrence on that date, including recurring occurrences the calendar already expands.
5. A task with no estimate adds 0 and still appears in the day.
6. When used is greater than the budget, the bar uses the existing overdue/amber treatment. At or under budget, it stays neutral.
7. Month and week views keep the current total label. The bar and the budget editor are day view only.
8. No new drag behavior. Rescheduling stays the drag-and-drop that 0.3 already shipped.

### 7.3 Edge cases

- A budget that fails to parse falls back to 6 hours.
- The bar fills to 100% at the budget and stays full when the day is over. The numeric used/budget label is what shows the overflow.

---

## 8. Feature: Bulk actions

### 8.1 Problem

Every task change is one task at a time. Completing or retagging a set means repeating the same edit.

### 8.2 Requirements

1. On the project task view, each **top-level** task can be selected. Subtasks have no selection checkbox. Actions never cascade to subtasks.
2. A bar appears when the selection is non-empty. It shows the count and these actions:
   - **Mark complete**
   - **Move to Active**
   - **Move to Backlog**
   - **Add tag**
   - **Remove tag**
3. **Mark complete** calls the existing toggle endpoint only for selected tasks that are not already completed. Already-completed tasks are skipped. That path must keep today’s recurrence behavior: completing a recurring task still creates the next instance.
4. **Move to Active** and **Move to Backlog** set `section` through the meta endpoint. If the task is completed, the same update sets `completed` to false. Moving to Completed is not a separate action; Mark complete is the only way into Completed, so `completed` and `section` stay paired the way toggle already pairs them.
5. The meta request must accept `completed`. Today `UpdateTaskMetaRequest` cannot set it. Add that field. Setting `completed: true` through meta is not required for 0.4 and must not spawn a recurring instance. Only toggle does that.
6. **Add tag** appends one non-empty tag to each selected task’s existing tags, skipping duplicates. **Remove tag** drops that tag where it is present. Both send the full replacement `tags` array the meta endpoint already expects.
7. Run requests sequentially. On the first failure, stop, report the task title and the error, and leave earlier tasks as saved. Then reload the project task list.
8. Clear the selection when the active project changes, and after a successful action.
9. No batch endpoint.

### 8.3 Edge cases

- An empty tag is rejected in the UI and sends no request.
- Remove tag on a task that does not have it is a no-op for that task, not an error.
- Selection does not include tasks that are filtered out by the current project tag filter.

---

## 9. Feature: Private Git remote, finished in the app

This is the heavy item. The `data/` directory is already a Git repo. Push, fetch, remote info, and ahead/behind are in the Git panel. There is no pull route. The only “add remote” path is an error string that tells the user to run `git remote add`. Credential callbacks in `backend/src/services/git.rs` try the SSH agent only, so HTTPS remotes fail even when Git Credential Manager would succeed.

### 9.1 What backup means here

The working copy on disk stays the source of truth. The app works with no network. A private `origin` (GitHub, GitLab, Gitea, or any other Git host) is an optional mirror.

Two people can each run Ironpad on their own clone and exchange work by push and pull. That is ordinary Git. It is not co-editing. The UI and `docs/git-backup.md` say that in plain language.

`data/` stays **one** repo. Per-project remotes are out of scope.

### 9.2 Add or replace `origin`

1. `POST /api/git/remote` with `{ "url": string }`.
2. Accept `https://`, `ssh://`, and `git@host:path` forms.
3. Reject an empty URL, a filesystem path, and any URL that embeds credentials (`user:password@` or a token in the userinfo). The error tells the user to use SSH or a Git credential helper.
4. If `origin` is missing, add it. If it exists, replace its URL and say that the previous URL was replaced.
5. After a valid save, fetch once. Keep the remote if the fetch fails, and show that error. Do not roll the URL back.
6. There is no remove-remote action in 0.4.
7. The Git panel shows a URL field when no remote is configured, and the current remote plus a way to replace the URL when one is.

### 9.3 Pull

1. `POST /api/git/pull`.
2. Pull means fetch, then fast-forward the current branch to its upstream.
3. Fast-forward only. If the histories have diverged, change nothing in the working tree and return a message that Ironpad will not merge or rebase. The user resolves that in Git.
4. If the working tree has changes that the fast-forward would overwrite, change nothing and say so. Auto-commit can still run on its normal 60-second cycle; pull does not invent a commit.
5. If the branch has no upstream yet, say that the first successful push sets it.
6. On success, report either “Already up to date” or how many commits came in.
7. The Git panel gets a **Pull** button next to the existing Fetch button. Fetch stays as it is: update remote-tracking refs, do not move the local branch.
8. A successful pull must let the existing file watcher refresh open views. Do not add a second refresh path if the watcher already emits for those writes.

### 9.4 Push sets upstream

On a successful push, if the current branch has no upstream, set it to `origin/<branch>`. Ahead/behind already depend on `branch.upstream()`, and today a push can succeed while those counts stay at zero.

### 9.5 Credentials and errors

Use the user’s Git credential helper for HTTPS, and the SSH agent for SSH. Do not add a password field.

Map failures to sentences a person can act on. Keep the raw Git error available in the log, not as the only UI text.

| Situation | UI message |
|-----------|------------|
| Auth failed | Could not authenticate. Use an SSH key in the agent, or a Git credential helper for HTTPS. Ironpad does not store tokens. |
| Remote rejected a non-fast-forward push | The remote has commits this copy does not have. Pull first. Ironpad did not force-push. |
| Pull is not a fast-forward | The remote and this copy have diverged. Pull did not change your files. |
| Dirty tree blocks pull | Uncommitted changes would be overwritten. Wait for auto-commit, or commit, then pull again. |
| No remote | No remote yet. Add a private repository URL. |
| Unreachable host | Could not reach the remote. |

Never force-push. Never pass `--no-verify`.

### 9.6 Documentation

Add `docs/git-backup.md`, linked from the Git panel and from `docs/index.md`. One short page:

- Create an empty **private** repo on the host. Ironpad will not create it.
- Paste the URL, then push.
- Pull brings in fast-forwards only.
- This is backup and async exchange, not live collaboration.
- Do not put a token in the URL.

### 9.7 Risks

| Risk | Mitigation |
|------|------------|
| Users expect live co-editing | Panel copy and `docs/git-backup.md` say async, fast-forward, no merge. |
| A token ends up in `origin` | Reject userinfo in the URL. The doc says to use a helper or SSH. |
| Pull overwrites local edits | Refuse when the working tree is dirty in a way the fast-forward would clobber. |
| HTTPS still fails because only the SSH agent is wired | Section 9.5 is part of the feature, not a follow-up. |

---

## 10. Documentation and release artifacts

- [`ROADMAP.md`](../../ROADMAP.md) — 0.4 holds this PRD, [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md), and [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md). Fit the day stays here. Task templates stay unscheduled. The release is not shipped.
- [`CHECKLIST.md`](./CHECKLIST.md) — task, dashboard, and Git items mirror §11. Library items mirror the library PRD’s acceptance list. GitHub items mirror the GitHub PRD’s acceptance list.
- `CHANGELOG.md` — `## [0.4.0]` stays undated until the Ironpad release is tagged. Task, dashboard, Git backup, Library, and GitHub notes are in that section.
- `ai-context.md` — point the 0.4 line at all three PRDs and say the release is not shipped.
- `docs/index.md` — all three PRDs, plus `docs/git-backup.md`.

---

## 11. Acceptance checklist

### Port (already done)

- [x] Default scan is 37891–37990, and `IRONPAD_PORT` pins a port.
- [x] Startup logs the URL to open.

### Task list and dashboard

- [x] Task view sorts by created, due date, or priority, including across reload.
- [x] Subtasks stay under the parent they already belong to.
- [x] Sidebar shows the active project’s overdue count, and a task due today is not in it.
- [x] Dashboard and sidebar agree on that count for the same project.
- [x] Dashboard tag chips filter incomplete tasks across projects, and clearing the chip restores the full dashboard.
- [x] Day view shows used/budget against 6 hours by default, turns amber over budget, and remembers a changed budget.
- [x] Bulk mark-complete skips tasks that are already done and still spawns the next instance of a recurring task.
- [x] Bulk move and bulk tag leave `completed` and `section` consistent, and a mid-list failure does not continue.

### Git backup

- [x] A private HTTPS or SSH URL can be saved as `origin` from the Git panel, and a URL with embedded credentials is rejected.
- [x] Pull fast-forwards and refuses a diverged history without changing files.
- [x] Auth failure, rejected push, and a dirty tree each show the message in section 9.5.
- [x] After the first push, ahead/behind reflect the upstream.
- [x] `docs/git-backup.md` is linked from the panel and does not describe live co-editing.

---

## 12. Decisions already made

1. **Port** — scan 37891–37990, override with `IRONPAD_PORT`. No config file in 0.4.
2. **Create-repo API** — out of this release and out of 0.5 unless a later PRD takes it on.
3. **One repo** — `data/` remains a single Git repository.
4. **Pull** — fast-forward only.
5. **Tags** — dashboard filter uses task frontmatter tags, one tag at a time.
6. **Budget** — 6 hours, stored locally in the browser, day view only.
7. **Sort default** — created, newest first.
8. **Library** — part of 0.4.0, specified in [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md). This document’s acceptance list does not cover it.
9. **GitHub issues and pull requests** — part of 0.4.0, specified in [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md). Read-only, one `owner/name` per project, no token stored. This document’s acceptance list does not cover it. The release is not shipped while that work, or the Library, is open.
