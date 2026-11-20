# Product Requirements Document — Ironpad v0.4.0 GitHub

**Version:** 0.4.0  
**Date:** 2026-11-19  
**Status:** Released 2026-11-20.  
**Parent context:** [`PRD-0.4.0.md`](./PRD-0.4.0.md) (task list, dashboard, Git backup), [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md) (Library), [`ROADMAP.md`](../../ROADMAP.md)

---

## 1. Summary

This is the GitHub part of release **0.4.0**. A project can name one repository on github.com. Ironpad then shows that repository’s open issues and open pull requests, and can turn one of them into a normal local task.

The lists come from the GitHub REST API. They are not files in the Git history, and they are not the private `origin` already specified for backing up `data/`. The Library reads files on disk and is specified separately. This document does not pull `README.md` or `docs/` from the repository.

The snapshot is read-only. Ironpad does not comment, close, label, or edit anything on GitHub. The last successful snapshot remains readable when the network is down.

---

## 2. Goals

| Goal | Success criteria |
|------|------------------|
| Attach a repo to a project | On a project, save `owner/name`. The link survives a reload and a project-page save. Clearing it stops the lists and leaves tasks that were already created. |
| See open work | The project shows open issues and open pull requests, with title, number, labels, and updated time. Pull requests are not listed again as issues. |
| Narrow the list | Issues filter to all open, assigned to the logged-in user, or created by that user. Pull requests filter to all open, authored by that user, or waiting on that user’s review. The choice survives a reload. |
| Stay current | Opening the view, a **Refresh** button, and a 15-minute timer while the view is open update the snapshot. An unchanged list does not spend GitHub’s rate limit. |
| Read an item | Opening a row shows the Markdown body, and up to 30 comments, without writing to GitHub. A pull request also shows its combined commit status. |
| Keep a local task | **Track as task** creates one Active task. Doing it again opens that same task. Completing the task leaves the GitHub item untouched. |
| See it on the dashboard | A project card shows how many open issues are assigned to the user and how many open pull requests request the user’s review, from the last snapshot. |

### Non-goals (GitHub half of v0.4.0)

- Creating a repository, or changing the notes repo’s `origin`. Git backup stays as specified in [`PRD-0.4.0.md`](./PRD-0.4.0.md).
- A token field, a token written into `data/`, OAuth inside Ironpad, or a GitHub App.
- Writing back: comments, closes, labels, assignees, reviews, or merges.
- Two-way sync between a task and the GitHub item it was copied from.
- Webhooks. The app has no public URL.
- More than one repository on a project.
- GitHub Enterprise, GitLab, or Gitea. The host is `api.github.com`.
- Pulling repository files (`README.md`, `docs/`, or any other path). That belongs to the Library, when the file is already on disk.
- Diffs, Actions logs, GitHub Projects, Discussions, and the releases list.
- Search across GitHub. Filters run on the snapshot already downloaded.
- Weekly review files and `waiting_on`. Those stay on the 0.5.0 proposal. A tracked task is an ordinary task and can be used by those later features without a special case.

---

## 3. What stays

1. **Remote backup** still sets `origin` on the `data/` repository, and pull is still fast-forward only. The GitHub screen does not read or write that remote.
2. Tasks, notes, the dashboard, and the Library keep their current storage and routes.
3. Saving a project’s page rewrites the body of `data/projects/{id}/index.md` and keeps existing frontmatter keys, including `github_repo`.
4. Auto-commit still commits `data/` on its 60-second cycle. The GitHub snapshot is not part of that commit.
5. An SSH key or a Git credential that can push `data/` is not treated as a GitHub API login.

---

## 4. Feature: Link one repository

### 4.1 Problem

The Git panel backs up notes. It cannot list issues or pull requests, because those are not commits. A project needs its own pointer at a github.com repository.

### 4.2 The pointer

1. Store `github_repo` on the project’s `index.md` frontmatter as `owner/name`. No other new project field.
2. The project sidebar gains **GitHub**, with the Notes, Tasks, and Library entries, shown while a project is active. The route name is `project-github`.
3. With no link, the view has one field and **Save**. The helper text says this is a github.com repository, separate from Remote backup, and that Ironpad does not store a token.
4. **Save** accepts only `owner/name`. Trim whitespace. Reject an empty value, a full URL, a `.git` suffix, a second slash, `..`, and any character outside letters, digits, `.`, `_`, and `-`. The error is: `Use owner/name, with no URL and no .git suffix.`
5. A valid save writes `github_repo` and then refreshes (section 5). If the refresh fails, keep the saved name and show the refresh error.
6. Replacing the name clears that project’s snapshot and loads the new repository.
7. **Remove** asks for confirmation: `Stop showing this repository. Tasks you already created stay.` Confirming deletes `github_repo` and that project’s snapshot directory. It does not delete tasks.

### 4.3 Edge cases

- `Owner/Name` is stored as typed after trim. GitHub treats the lookup as case-insensitive. The displayed name is the saved string.
- Two projects may save the same `owner/name`. Each has its own snapshot.
- A 404 after a successful save means the name stays and the view says: `That repository was not found, or this login cannot see it.`

---

## 5. Feature: Lists, refresh, and the snapshot

### 5.1 Problem

Open issues and open pull requests change outside Ironpad. The view has to show the latest open set, and it has to keep working from the last copy when GitHub cannot be reached.

### 5.2 What is fetched

The backend calls GitHub. The browser never calls `api.github.com` and never receives a token.

1. Issues: `GET /repos/{owner}/{repo}/issues?state=open&sort=updated&direction=desc&per_page=100`.
2. Drop any issue object that contains a `pull_request` key. Those are pull requests and belong only in the pull list.
3. Pull requests: `GET /repos/{owner}/{repo}/pulls?state=open&sort=updated&direction=desc&per_page=100`.
4. Follow `Link` rel=`next` for at most five pages on each list (500 items). When a sixth page exists, keep the five pages and show: `Showing the 500 most recently updated open items.`
5. Send `Accept: application/vnd.github+json`, `Authorization: Bearer`, and `X-GitHub-Api-Version` set to the REST version current when this is implemented. The docs consulted for this PRD used `2026-03-10`.
6. Store each list’s `ETag`. The next refresh sends it as `If-None-Match`, quotes included. A **304** keeps the stored list, refreshes `refreshed_at`, and does not count against the primary rate limit.
7. A successful 200 replaces that list. Items that are no longer in the open set leave the list. Delete their cached bodies and comments.

### 5.3 Snapshot location

1. Write the snapshot outside `data_dir()`, in the OS cache directory:
   - Windows: `%LOCALAPPDATA%\Ironpad\github\{project-id}\`
   - macOS: `~/Library/Caches/Ironpad/github/{project-id}/`
   - Linux: `$XDG_CACHE_HOME/ironpad/github/{project-id}/`, or `~/.cache/ironpad/github/{project-id}/` when `XDG_CACHE_HOME` is unset
2. `snapshot.json` holds `repo`, `login`, `refreshed_at`, both ETags, and the normalized issues and pull requests.
3. Each list item stores: `number`, `title`, `html_url`, `updated_at`, author login, label `name` and `color`, `comments` count, and milestone due date as `YYYY-MM-DD` when `due_on` begins with that date. Pull items also store `draft`, `requested_reviewers` logins, and `head_sha`.
4. Bodies live in `bodies/issue-{number}.md` and `bodies/pull-{number}.md`. Comments for an opened item live beside the body. These files are written when the item is opened (section 6), not during the list refresh.
5. Nothing under the cache directory is committed by auto-commit.

### 5.4 When to refresh

1. `GET /api/projects/{id}/github` reads the snapshot and does not call GitHub. It returns the saved repo, login, `refreshed_at`, both lists, and `stale: true` when `refreshed_at` is missing or older than 15 minutes.
2. `POST /api/projects/{id}/github/refresh` performs section 5.2.
3. The view calls refresh when it opens and the snapshot is missing or stale, when the user clicks **Refresh**, and every 15 minutes while that view stays open.
4. A 304 or a 200 clears the error line and updates the “Updated …” time from `refreshed_at`.
5. On a network failure, 401, 403, or 404, leave the previous snapshot on screen and show one error line. A missing snapshot shows the error and empty lists.

### 5.5 Filters

1. Filters apply to the snapshot in the browser. Changing a filter does not call GitHub.
2. Issue filters: **All open** (default), **Assigned to me**, **Created by me**.
3. Pull request filters: **All open** (default), **Mine**, **Review requested**.
4. “Me” is the `login` from `GET /user`, stored on the snapshot during refresh. Compare logins without case sensitivity.
5. **Assigned to me** matches an issue whose assignees include that login. **Created by me** matches the author. **Mine** matches the pull request author. **Review requested** matches a pull request whose `requested_reviewers` include that login. A request to a team does not match.
6. Remember each filter in `localStorage` (`ironpad-github-issue-filter:<projectId>`, `ironpad-github-pr-filter:<projectId>`). An unknown stored value falls back to All open.
7. When `login` is missing, the “me” filters are hidden and the lists stay on All open.
8. A row shows number, title, labels, author, and relative `updated_at`. An issue row also shows assignees when it has them. A draft pull request shows **Draft**.
9. An empty filtered list says `No open issues` or `No open pull requests`.

### 5.6 Errors

Use these sentences. Do not append the token, the response body, or a stack trace.

| Condition | Sentence |
|-----------|----------|
| No usable login | `GitHub needs a login. Run gh auth login, or set GH_TOKEN for this app. Ironpad does not store a token.` |
| 401, or 403 that is not a rate limit | `GitHub refused the login. Run gh auth login again, or replace GH_TOKEN. A git SSH key is not a GitHub API login.` |
| 404 | `That repository was not found, or this login cannot see it.` |
| Rate limit (403 or 429 with a rate-limit signal) | `GitHub rate limit reached. The last snapshot is still shown.` When `X-RateLimit-Reset` is present, add the local time it names. |
| Network or other HTTP failure | `Could not reach GitHub. The last snapshot is still shown.` |
| Bad repo string | `Use owner/name, with no URL and no .git suffix.` |

### 5.7 Edge cases

- The issues endpoint returning only pull requests yields an empty issue list. That is a successful refresh.
- A draft pull request stays in the open pull list.
- Labels and colors come from the payload. A missing color still shows the label name.
- `refreshed_at` is UTC. The view formats it in the local timezone.

---

## 6. Feature: Read an item

### 6.1 Problem

The list is a title. The body and the conversation are Markdown the user needs to read before deciding to track the item.

### 6.2 Requirements

1. Opening a row shows a read-only pane. There is no Save.
2. If a cached body exists, show it immediately. Then fetch the current item.
3. An issue body comes from `GET /repos/{owner}/{repo}/issues/{number}`. A pull request body comes from `GET /repos/{owner}/{repo}/pulls/{number}`.
4. Render the body as Markdown. The renderer must not execute raw HTML as script. Reuse the note renderer in read-only form, or use a lighter renderer that escapes HTML.
5. A body larger than 1 MiB (1,048,576 bytes) is not stored and not rendered. The pane says the body is too large and still offers **Open on GitHub**.
6. Comments come from `GET /repos/{owner}/{repo}/issues/{number}/comments?per_page=100`, for both issues and pull requests. Show the 30 newest, newest first, each with author, time, and rendered Markdown. When more than 30 exist, the pane says the rest are on GitHub.
7. This comment list is the issue thread. It does not include pull-request review comments on lines of a diff.
8. For a pull request, `GET /repos/{owner}/{repo}/commits/{head_sha}/status` adds one line: `success`, `pending`, `failure`, `error`, or `No status`. A failure of this call omits the line and still shows the body.
9. **Open on GitHub** is a link to the stored `html_url`. The pane also offers **Copy link**.
10. A failed body fetch with a cached body keeps the cached body and shows the section 5.6 sentence. With no cache, the pane shows that sentence and **Open on GitHub** when `html_url` is already on the list item.
11. After a refresh removes the item from the open lists, the pane says it is no longer open and keeps **Open on GitHub** when the URL is known.

---

## 7. Feature: Track as task

### 7.1 Problem

An issue or pull request is useful as a reminder inside Ironpad: a due date, tags, and the calendar. Copying it once is enough. Keeping the two copies identical is a different product.

### 7.2 Requirements

1. **Track as task** creates one task in the current project through the existing task write path.
2. Title is the item title. Section is Active. Tags are the label names, in order, unmodified. `due_date` is the milestone `YYYY-MM-DD` when the snapshot has one, otherwise unset. Priority is left at the normal create default.
3. The body is the cached Markdown body, then a blank line, then `html_url`. When the body was not cached, the body is the URL only.
4. Frontmatter gains `github_url` (the `html_url`) and `github_number`. Later task edits must preserve both keys. The current meta update already rewrites only the keys it was given, so those two keys survive a title, tag, or due-date edit.
5. If this project already has a task whose `github_url` equals that URL, create nothing. The control is **Open task** and navigates to the existing task.
6. The new task is an ordinary task. Completing it, commenting on it, or changing its due date does not call GitHub.
7. The button is available from the list row and from the open pane. Tracking does not require the body fetch to have succeeded.

### 7.3 Edge cases

- A second project linked to the same repository can track the same URL. The duplicate check is per project.
- Removing the GitHub link does not delete or rewrite the task.
- A label named like a priority (`high`, `bug`) is stored as a tag. It is not copied into the priority field.

---

## 8. Feature: Dashboard counts

### 8.1 Problem

Reviews and assigned issues are easy to miss when the GitHub view is closed. The dashboard already summarizes each project.

### 8.2 Requirements

1. `GET /api/github/summary` reads snapshots only. It does not call GitHub.
2. For each project that has `github_repo` and a snapshot with a `login`, return `project_id`, `repo`, `assigned_issues`, `review_pulls`, and `refreshed_at`. Counts use the same matching rules as the **Assigned to me** and **Review requested** filters.
3. The project card shows `N assigned · N reviews` when at least one count is greater than zero, and the text links to that project’s GitHub view.
4. Hide the line when both counts are zero, when the project has no snapshot, or when `login` is missing.
5. The line describes the last snapshot, including a stale one. Opening the GitHub view is what refreshes it.

---

## 9. Login

1. Resolve a token in the backend process only, in this order: `gh auth token` when it exits 0 within 5 seconds, then the `GH_TOKEN` environment variable, then `GITHUB_TOKEN`.
2. Hold the token in memory for the request. Do not write it to `data/`, to the snapshot, to a log line, or to any JSON response.
3. When `gh auth token` returns a token, use it even if an environment variable is also set. A later 401 uses the refused-login sentence. Do not silently switch to the environment variable.
4. When `gh` is missing, times out, or exits non-zero, try the environment variables. When none of the three yields a token, refresh fails with the “needs a login” sentence and does not offer a password or token input.
5. `GET /user` during a refresh stores `login` on the snapshot. A list refresh that cannot learn the login still stores the lists. The “me” filters stay hidden until a later refresh stores `login`.

---

## 10. API

All routes are on the existing local server. Responses never include a token.

| Method and path | Behavior |
|-----------------|----------|
| `GET /api/projects/{id}/github` | Saved link and snapshot. No GitHub call. |
| `PUT /api/projects/{id}/github` | Body `{ "repo": "owner/name" }` saves, or `{ "repo": null }` removes. A string is validated before write. Save does not itself call GitHub; the client then calls refresh. Remove deletes the snapshot directory. |
| `POST /api/projects/{id}/github/refresh` | Section 5.2. Returns the same shape as GET. |
| `GET /api/projects/{id}/github/issues/{number}` | Cached or freshly fetched issue body, comments, and `html_url`. |
| `GET /api/projects/{id}/github/pulls/{number}` | Same for a pull request, plus the status word when the status call succeeds. |
| `POST /api/projects/{id}/github/issues/{number}/track` | Section 7. Returns the task id and `created: true`, or the existing task id and `created: false`. |
| `POST /api/projects/{id}/github/pulls/{number}/track` | Same for a pull request. |
| `GET /api/github/summary` | Section 8. No GitHub call. |

A request for an item number that is absent from the snapshot returns 404 with `That item is not in the open snapshot.`

---

## 11. Documentation and release artifacts

- [`ROADMAP.md`](../../ROADMAP.md) — 0.4.0 includes this PRD and is not released.
- [`CHECKLIST.md`](./CHECKLIST.md) — the GitHub block mirrors section 12 below.
- [`docs/index.md`](../index.md) — one line for this PRD.
- [`docs/API.md`](../API.md) — the routes in section 10.
- `CHANGELOG.md` — GitHub notes are under the undated `## [0.4.0]` section. Do not put a release date on that heading until the Ironpad release is tagged.
- A short user page `docs/github-link.md`, linked from the GitHub view: one repo per project, read-only, `gh auth login` or `GH_TOKEN`, separate from Remote backup, tracking a task does not close the issue.

---

## 12. Acceptance checklist

### Link

- [x] A project can save `owner/name`, and that value is still on `index.md` after the project page is saved.
- [x] A URL, a `.git` suffix, or an empty value is rejected with the sentence in section 5.6.
- [x] Remove deletes the link and the snapshot, and leaves a task created earlier in place.
- [x] Saving a repository does not change the notes repo’s `origin`.

### Lists and refresh

- [x] Open issues and open pull requests appear in separate lists. A pull request returned by the issues endpoint is not shown as an issue.
- [x] Assigned, created-by-me, mine, and review-requested filters match the logged-in login and survive a reload. The default is All open.
- [x] Refresh on open when the snapshot is older than 15 minutes, from the button, and on the 15-minute timer while the view is open.
- [x] A 304 keeps the current rows and updates the shown time.
- [x] A failed refresh keeps the previous rows and shows one of the sentences in section 5.6.
- [x] The snapshot directory is outside `data/`, and auto-commit does not pick it up.

### Login

- [x] A working `gh auth token` is used without a token field in the UI.
- [x] With no `gh` login, `GH_TOKEN` or `GITHUB_TOKEN` is used. With none of the three, the view shows the needs-a-login sentence.
- [x] No API response, snapshot file, or log line contains the token.

### Read, track, dashboard

- [x] Opening a row shows the Markdown body and up to 30 comments, read-only, and does not change the item on GitHub.
- [x] An opened pull request shows the combined status word when GitHub returns one.
- [x] Track as task creates one Active task with the title, labels as tags, milestone date when present, `github_url`, and `github_number`.
- [x] Tracking the same URL again opens the existing task and does not create a second one.
- [x] Completing that task does not call GitHub.
- [x] A project card shows assigned and review counts from the last snapshot, and hides the line when both are zero.

---

## 13. Risks

| Risk | Mitigation |
|------|------------|
| The link is mistaken for Remote backup | The empty state and `docs/github-link.md` say it is a separate, read-only github.com repository. Save does not write `origin`. |
| A token is committed with the notes | There is no token field. The token stays in process memory. The snapshot lives outside `data/`. |
| Issue text contains a secret and then lands in a task | The snapshot is outside the notes repo. **Track as task** is an explicit copy into `data/`, which auto-commit will then include. The button label stays **Track as task**, and the user page says the task is a local copy. |
| Users expect a completed task to close the issue | The task is ordinary local work. Nothing in the complete path calls GitHub. |
| A large repository blows the rate limit | Open items only, at most 500 per list, ETag so an unchanged poll is a 304, and bodies load only when a row is opened. |
| `gh` prints a token into a log | The backend reads stdout and must not trace that process output. |
| Rendered Markdown runs HTML from an issue | The renderer does not execute script. HTML is escaped unless the note renderer’s existing sanitization is reused. |

---

## 14. Decisions already made

1. **One repo** — `github_repo: owner/name` on the project `index.md`. github.com only.
2. **Read-only** — lists, bodies, and comments. No write-back in this release.
3. **Not Git backup** — `origin` on `data/` is unchanged.
4. **Not the Library** — repository files on disk stay in [`PRD-0.4.0-library.md`](./PRD-0.4.0-library.md). This PRD does not fetch file paths.
5. **Login** — `gh auth token`, then `GH_TOKEN`, then `GITHUB_TOKEN`. No token stored.
6. **Snapshot** — OS cache directory, outside `data/`. List refresh replaces the open set. Bodies load on open.
7. **Filters** — local, default All open. “Review requested” means the user is in `requested_reviewers`, not a team.
8. **Task** — one-way copy. Identity is `github_url` inside that project. Completing the task is local.
9. **Dashboard** — counts from the last snapshot only. No GitHub call from the dashboard.
10. **Later** — comment or close from Ironpad, a second repository, and any host other than github.com need a later PRD. Weekly review and `waiting_on` stay on the 0.5.0 proposal.
