# Product Requirements Document — Ironpad v0.4.0 Library

**Version:** 0.4.0  
**Date:** 2026-11-19  
**Status:** Released 2026-11-20.  
**Parent context:** [`PRD-0.4.0.md`](./PRD-0.4.0.md) (task list, dashboard, Git backup), [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md) (GitHub issues and pull requests), [`ROADMAP.md`](../../ROADMAP.md)

---

## 1. Summary

This is the Library part of release **0.4.0**. The task list, dashboard, and Git backup half is already in the working tree and is specified in [`PRD-0.4.0.md`](./PRD-0.4.0.md). GitHub issues and pull requests are specified in [`PRD-0.4.0-github.md`](./PRD-0.4.0-github.md). This document specifies the Library.

The Prompts screen becomes the Library. Every prompt behavior stays. A user can also pin a real file, such as `ai-context.md` in a coding repo. Ironpad stores a pointer and reads the original file. It does not copy that file into `data/`.

The open item can be shown as rendered markdown or as raw source. Versions (Git) can be collapsed. Several items can be copied as one markdown blob for a chat.

This release does not write linked files, run a prompt against a model, or search inside linked file bodies.

---

## 2. Goals

| Goal | Success criteria |
|------|------------------|
| Read a repo file beside prompts | From a project, set a folder on disk, link `ai-context.md`, and see that file’s current text. An edit made outside Ironpad shows up the next time the library loads the link. |
| Keep prompts | Create, edit, save, duplicate, delete, filter, search, `{{variables}}`, Copy, and Copy rendered still work. A prompt file with no `kind` is a prompt. |
| Switch reading mode | One control flips the open item between rendered markdown and raw source. The choice survives a reload. Saving a prompt writes the raw draft. |
| Tuck Git history away | Versions (Git) collapses. The choice survives a reload. The reading pane uses the full width while it is collapsed. |
| Copy files into a chat | Multi-select copies one markdown blob, with a heading and a path for each item. |

### Non-goals (library half of v0.4.0)

- Writing a linked file from Ironpad. Raw view of a link is read-only. Edit the file outside, or use **Open externally**.
- Watching the coding repo. Refresh on open, when the window regains focus, and on **Reload**.
- Full-text search of linked files. Ctrl+K stays on `data/`. Library search matches title, folder, tags, description, and path.
- Renaming `/api/prompts`, moving `data/prompts/` or `data/projects/{id}/prompts/`, or renaming the `prompts` and `project-prompts` routes.
- Filling `{{title}}`, `{{content}}`, or other fields from the open note or task. That stays on the 0.5.0 proposal.
- Posting a prompt to Ollama or any other local or hosted model.
- Weekly review files, `waiting_on`, the capture hotkey, and project blueprints.
- Graph view, kanban, themes, and PDF or HTML export.
- Treating a linked file as a note. The backlink index keeps skipping paths that contain `prompts/`.
- Importing a directory tree, including `node_modules` or a recursive folder pick.

---

## 3. What stays

1. **Global Library** from the top bar, and the project Library from the sidebar. Both labels change from Prompts to **Library**. The top-bar title attribute becomes **Global Library**.
2. Routes stay `prompts` and `project-prompts`. Storage stays `data/prompts/` and `data/projects/{id}/prompts/`. The API stays under `/api/prompts`.
3. Folders, tags, description, list filters, and semantic search stay for prompts.
4. The `{{variable}}` form, **Copy**, and **Copy rendered** stay for prompts.
5. Prompt Git history stays available through the existing per-file log when Versions is expanded.
6. A file with no `kind`, or with `kind: prompt`, is a prompt. Existing prompt files keep working with no rewrite.

---

## 4. Feature: Linked files

### 4.1 Problem

Prompts are files Ironpad owns under `data/`. A coding repo already has `ai-context.md`, `ROADMAP.md`, and other markdown. Copying them into the prompt folders makes a second copy that goes stale.

### 4.2 Pointer

1. A link is a markdown file in the same folders as prompts.
2. Frontmatter uses the fields prompts already store (`id`, `title`, `folder`, `tags`, `description`, `scope`, `project_id`) plus:
   - `kind: link`
   - `source`: a path string
   - `template`: optional boolean, default false
3. The pointer body is empty. The source text is not written into the pointer.
4. `GET` of that item reads the source file and returns it as `content`, along with `kind`, `source`, `resolved_path`, `read_only: true`, and `source_missing`.
5. Create and update go through the existing prompt endpoints. The payload gains optional `kind`, `source`, and `template`. On a link, a `content` field is ignored. The source file’s bytes do not change.
6. **Delete** removes the pointer only. The confirm names the title and says the file on disk stays.
7. **Duplicate** creates another pointer with the same `source`. It does not copy the source file. The control label for a link is **Duplicate link**.
8. The list reads pointer frontmatter only. It does not open external files. Each row shows **Prompt** or **Link**.

### 4.3 Project folder root

1. Each Ironpad project may have a folder root: the coding repo on this machine.
2. Store it in `localStorage` under `ironpad-library-root:<projectId>`. It is not written into `data/` and is not committed.
3. A project link’s `source` is relative to that root: forward slashes, no leading slash, no `..` segment, not absolute.
4. A global link’s `source` is an absolute path.
5. The library can set and clear the root. Browsing starts at that root when it is set.
6. If a project link is opened and the root is missing, the pane asks for the root. Ironpad does not guess a directory.

### 4.4 Adding links

1. **Add link** lists a directory from the backend (`GET /api/library/browse?path=`). In a project, the first listing is the folder root. In the global library, the user pastes an absolute directory to start.
2. The user can select several files and confirm once. Each file becomes one pointer. Title defaults to the filename. Folder defaults to the active library folder. Tags start empty.
3. There is no “import this folder” and no recursion.
4. **Paste path** accepts a file path when the listing is not enough.
5. A project paste inside the root is stored relative to the root. A project paste outside the root is refused. The message tells the user to change the root or add the file from the global library.
6. **Suggest files.** When a project root is set, if any of these exist, offer them in one confirm list: `ai-context.md`, `ROADMAP.md`, `docs/index.md`. Nothing is created until the user confirms. Remember a dismissal in `localStorage` (`ironpad-library-suggest-dismissed:<projectId>`). Do not offer a path that is already linked from this project or that was dismissed.

### 4.5 What may be linked

1. Regular files only. A directory cannot be a link.
2. Maximum size is 1 MiB (1,048,576 bytes), checked before the body is returned. A larger file is refused, and the message includes the size.
3. Allowed suffixes, case-insensitive: `md`, `markdown`, `txt`, `rs`, `vue`, `ts`, `js`, `toml`, `json`, `yml`, `yaml`, `py`, `css`, `html`, `mdc`.
4. Any other suffix is refused. Binaries are not linked.
5. `.md` and `.markdown` may use rendered mode. Every other allowed suffix is raw only, and the rendered control is hidden.
6. The file must be valid UTF-8. Otherwise the pane says the file is not UTF-8 text and does not show a lossy decode.

### 4.6 Resolve and refresh

1. A project source joins the stored root and the relative source. A result that escapes the root is an error. A symlink that resolves outside the root is refused.
2. A global source is the absolute path as stored, and it must be a file that passes section 4.5.
3. A missing file stays in the list. The pane shows the stored path and **Relink**. Relink replaces `source` on that pointer.
4. Read the file when the item is opened, when the window regains focus while it is open, and when the user clicks **Reload**.
5. Do not add a file watcher on the coding repo. The existing watcher keeps skipping paths that contain `prompts/`.

### 4.7 Templates

1. `template: false` is the default for a link. `{{variables}}` in the file are ordinary characters. **Copy** copies the source text. There is no variable form and no **Copy rendered**.
2. `template: true` shows the same variable form, **Copy**, and **Copy rendered** as a prompt. Substitution happens in memory for the copy and for the rendered view. The source file is not modified.
3. Turning the flag on or off writes the pointer only.

### 4.8 Edge cases

- Store relative sources with forward slashes, including on Windows.
- Two pointers may target the same source.
- Semantic search for a link matches the pointer’s title, description, tags, and `source`. It does not read the external body.
- An oversize file that grows past the cap after it was linked fails on the next read with the same size error. The pointer remains.

---

## 5. Feature: Rendered and raw

### 5.1 Problem

The prompt pane is a textarea. “Rendered preview” is a `<pre>` of variable substitution, and it appears only when the draft contains `{{…}}`. A linked markdown file should be readable as markdown, and the user still needs the source.

### 5.2 Requirements

1. The open item has **Raw** and **Rendered**.
2. **Raw** for a prompt is the editable textarea used today. **Raw** for a link is the source text and is not editable.
3. **Rendered** is read-only markdown. For a prompt, and for a link with `template: true`, substitute variables first, then render, so the preview matches **Copy rendered**.
4. **Save** on a prompt writes the raw draft. It never writes rendered HTML or substituted text.
5. Remember the mode in `localStorage`. Default a prompt to raw. Default a `.md` or `.markdown` link to rendered. Other links have no rendered mode (section 4.5).
6. The renderer must not execute raw HTML as script. Reuse the note renderer in read-only form, or use a lighter renderer that escapes HTML.
7. When a linked file has YAML frontmatter, raw shows the whole file and rendered shows the body. A file with no frontmatter renders the whole text.
8. The variable form stays above the pane for prompts and for template links. It is hidden for other links.
9. One mode is on screen at a time. The reading pane uses the full width while Versions is collapsed.

### 5.3 Edge cases

- Switching between raw and rendered does not drop an unsaved prompt draft.
- An empty body is an empty pane, not an error.
- An unclosed `{{` stays as text.

---

## 6. Feature: Collapse versions

### 6.1 Problem

`PromptsView` always shows **Versions (Git)** in a second column. Folder filters already collapse. Versions does not.

### 6.2 Requirements

1. Versions (Git) has a show/hide control.
2. Remember the state in `localStorage`. The default is collapsed.
3. While collapsed, the commit list and the diff are hidden, and the main pane takes that width.
4. While expanded, a prompt uses the current per-file Git log and diff (`logByPath` on the prompt file).
5. While expanded, a link shows one line: this file is outside Ironpad’s repository. Do not list commits of the pointer as if they were the source file’s history.
6. Collapsing does not clear a loaded commit selection. Expanding a prompt shows it again.

---

## 7. Feature: Copy bundle

1. The list can multi-select items without changing which item is open. A checkbox or an equivalent control is enough. Opening an item still shows it.
2. **Copy bundle** writes one markdown document to the clipboard.
3. Each selected item contributes a heading (the title), a line with the path (the prompt file’s path, or the link’s resolved path), and the text. Prompts contribute the raw body. Links contribute the current source text. Variable substitution is not applied. **Copy rendered** on a single prompt remains the way to copy substituted text.
4. Order follows the list order.
5. With nothing multi-selected, **Copy bundle** uses the open item.
6. The action writes nothing to disk.

---

## 8. Feature: Open externally and copy path

1. A link offers **Open externally** and **Copy path**.
2. **Copy path** copies the resolved absolute path.
3. **Open externally** asks the backend to open that file with the operating system’s default application. The request carries the link id (`POST /api/library/open` with `{ "id" }`). The server resolves the id through the same rules as read. It does not open a client-supplied path that skipped those rules.
4. A missing file does not open and does not copy a path. The pane keeps the missing-file message and **Relink**.

The server already listens on `127.0.0.1`. Browse and read can see files the OS user can read. That is this feature. Project-link resolution still rejects `..` and a symlink that leaves the root.

---

## 9. Documentation and release artifacts

- [`ROADMAP.md`](../../ROADMAP.md) — 0.4.0 includes this PRD and is not released.
- [`CHECKLIST.md`](./CHECKLIST.md) — the Library block mirrors section 11 below.
- [`docs/index.md`](../index.md) — one line for this PRD.
- [`docs/API.md`](../API.md) — `kind`, `source`, and `template` on the prompt payloads, `GET /api/library/browse`, and `POST /api/library/open`.
- `CHANGELOG.md` — Library notes are under the undated `## [0.4.0]` section. Do not put a release date on that heading until the Ironpad release is tagged. The GitHub release is still 0.3.0.

---

## 10. Risks

| Risk | Mitigation |
|------|------------|
| A duplicate is mistaken for a snapshot of the file | **Duplicate link** copies the pointer. The source stays a single file. |
| A global absolute path fails on another machine | The link stays in the list, shows missing, and offers **Relink**. Project links store a relative `source`. The root stays in the browser. |
| The user expects to edit `ai-context.md` in the library | Raw is read-only. **Open externally** is the edit path. Save does not write the source. |
| Rendered markdown runs HTML from a linked file | The renderer does not execute script. HTML is escaped unless the note renderer’s existing sanitization is reused. |
| Browse walks the whole disk | The project listing starts at the folder root. A path outside that root is an explicit paste, and a project link still cannot store it. |
| Prompt variables rewrite a linked document | Substitution runs only for prompts and for `template: true`, and only in memory. |

---

## 11. Acceptance checklist

### Library label

- [x] The top bar and the project sidebar say Library. Routes remain `prompts` and `project-prompts`.

### Linked files

- [x] A prompt with no `kind` still opens, edits, and saves as it does today.
- [x] Adding a link writes a pointer under the prompt folders and does not copy the source file into `data/`.
- [x] A project link’s `source` is relative to the folder root in `localStorage`. That root is not stored under `data/`.
- [x] Opening a link shows the source file’s current text. An outside edit appears on Reload and when the window regains focus.
- [x] Save on a link can change title, folder, tags, description, source, or template, and leaves the source file’s bytes unchanged.
- [x] Delete removes the pointer and leaves the source file in place.
- [x] A missing source stays in the list and can be relinked.
- [x] A file over 1 MiB, a directory, or a disallowed suffix is refused.
- [x] `{{variables}}` are substituted only for prompts and for links with template turned on.

### Rendered and raw

- [x] The open item switches between raw source and rendered markdown.
- [x] Saving a prompt writes the raw draft, including when the pane was on rendered.
- [x] A markdown link opens on rendered. A non-markdown link stays on raw, and rendered is unavailable.
- [x] The mode is remembered across reload.

### Versions

- [x] Versions (Git) collapses, defaults to collapsed, and remembers the choice.
- [x] An expanded prompt still shows that file’s Git log.
- [x] An expanded link does not list Ironpad commits as that file’s history.

### Copy bundle, suggest, open

- [x] Copy bundle puts the selected items in one markdown blob: prompts as raw body, links as current source text, with no variable substitution.
- [x] With a project root set, the library offers `ai-context.md`, `ROADMAP.md`, and `docs/index.md` when those files exist, and creates a link only after confirmation.
- [x] Open externally and Copy path use the resolved source. A missing file does not open.

---

## 12. Decisions already made

1. **Name** — the UI says Library. Routes, prompt directories, and `/api/prompts` stay.
2. **Pointer** — `kind: link` in the existing prompt folders. The source file is not copied.
3. **Root** — one folder per Ironpad project, in `localStorage`, not in Git.
4. **Writes** — a link is read-only. Save updates the pointer.
5. **Cap** — 1 MiB, UTF-8, and the suffix list in section 4.5.
6. **Versions** — collapsed by default. Links have no Ironpad history.
7. **Reading default** — prompts open raw. Markdown links open rendered.
8. **Bundle** — no variable substitution. **Copy rendered** stays on one prompt or one template link.
9. **Later** — fill-from-note, a local model, weekly review, `waiting_on`, the capture hotkey, and blueprints stay on the 0.5.0 proposal.
