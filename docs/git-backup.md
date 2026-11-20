# Private Git backup

Ironpad keeps your notes and tasks in the local `data/` folder. That working copy is the source of truth. The app runs fully offline.

You can optionally point **`origin`** at a **private** repository on GitHub, GitLab, Gitea, or any other Git host. Ironpad treats that remote as a mirror: push your commits up, pull when the remote has moved ahead. This is **backup and async exchange**, not live co-editing. Two people can each run Ironpad on their own clone and share work with ordinary Git push and pull—they are not editing the same live session.

## Set up

1. On your host, create an **empty private** repository. Ironpad does not create or provision repos for you.
2. In the Git panel, open **Remote backup**, paste the clone URL (HTTPS or SSH), and save. The app adds or replaces `origin` and tries a fetch.
3. Commit any local changes, then **Push**. The first successful push sets the branch’s upstream so ahead/behind counts and pull work as expected.

Use the system **Git credential helper** for HTTPS (for example Git Credential Manager) or an **SSH key loaded in the agent**. Do **not** put a password or personal access token in the URL—Ironpad rejects URLs with embedded credentials.

## Push and pull

- **Push** sends your commits to `origin`. Ironpad never force-pushes.
- **Fetch** updates remote-tracking refs without moving your current branch.
- **Pull** fetches, then **fast-forwards only**: if your branch can move forward to match the remote, it does. If histories have diverged, pull leaves your files unchanged and tells you to resolve the situation outside Ironpad (merge or rebase in Git). Uncommitted changes that would be overwritten block pull until you commit or wait for auto-commit.

If the remote has commits you do not have yet, pull (or fetch and then pull) before pushing again.

## What this is not

- Not hosted sync or OAuth sign-in inside Ironpad.
- Not automatic merge or rebase when branches diverge.
- Not real-time collaboration—think email-with-Git, not Google Docs.

For API and UI details, see the technical notes linked from [`docs/index.md`](./index.md) (`git-remote-origin`, `git-pull`, `git-user-facing-errors`, and related pages).