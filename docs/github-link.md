# GitHub link (read-only)

Each Ironpad **project** can name **one** public repository on **github.com** using `owner/name` (for example `acme/widget`). This is **not** a URL and **not** a `.git` clone address.

Open the project’s **GitHub** screen from the sidebar (same level as Tasks, Notes, and Library) to see open **issues** and **pull requests** from that repository.

## Read-only

Ironpad **never** writes to GitHub. It does not comment, close, label, merge, or edit issues or pull requests. Lists, bodies, and comments are a **local snapshot** for reading inside the app. Use **Open on GitHub** when you need the live page.

## Login

The app calls the GitHub API from the **backend** only. Ironpad does not show a token field and does not store credentials in `data/`.

Authenticate on the machine that runs Ironpad:

1. **GitHub CLI** — run `gh auth login`, then ensure `gh auth token` works for the same user.
2. **Environment** — set `GH_TOKEN` (or `GITHUB_TOKEN`) for the Ironpad process if you are not using `gh`.

A Git **SSH key** or credential helper used for **Remote backup** (push/pull of `data/`) is **not** a GitHub API login.

## Not Remote backup

**Remote backup** in the Git panel sets `origin` on your local notes repository and push/pulls the `data/` folder. That is separate from the GitHub link:

| | GitHub link | Remote backup |
|---|-------------|----------------|
| Purpose | Read open issues and PRs on github.com | Mirror `data/` to a private Git remote |
| Value | `owner/name` on the project | HTTPS or SSH clone URL as `origin` |
| Direction | Read-only snapshot | Push and pull commits |

See [`docs/git-backup.md`](./git-backup.md) for backup workflow.

## Track as task

**Track as task** copies one issue or pull request into a normal **Active** task in the project (title, labels as tags, milestone due date when present). The task stores `github_url` and `github_number` so **Track as task** again opens the same task.

**Completing or editing that task is local work only.** It does **not** close or update the GitHub item.

## Refresh and offline

The snapshot updates when you open the view, press **Refresh**, or every 15 minutes while the view stays open. If GitHub is unreachable or rate-limited, the last successful lists and cached bodies remain visible.

For HTTP routes and dashboard counts, see [`docs/API.md`](./API.md) and the technical notes in [`docs/index.md`](./index.md).