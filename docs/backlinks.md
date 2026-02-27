# Backlinks — Note Linking System

Ironpad supports linking between project notes using standard markdown links. The backlinks system tracks these connections and provides both forward links (outgoing) and backlinks (incoming) for every note.

## User Guide

### Inserting a Link

1. Open a project note in the editor.
2. Type `/link` anywhere in the note body.
3. A searchable dropdown appears with all notes in the current project.
4. Use arrow keys or type to filter, then press **Enter** (or click) to select.
5. The `/link` text is replaced with a markdown link: `[Note Title](note-id)`.

The link renders immediately in the editor as a clickable hyperlink.

### Viewing Links

Below the editor, the **Linked Notes** panel shows two sections:

- **Links To** — notes that the current note links to (forward links / outgoing).
- **Linked From** — notes that link back to the current note (backlinks / incoming).

Click any entry in the panel to navigate directly to that note.

The panel updates automatically after every save (1-second auto-save debounce).

### Link Format

Links use standard markdown syntax with the note's frontmatter ID as the target:

```markdown
[Meeting Notes](myproject-20260227-153000)
```

This means links are portable and readable in any markdown viewer. The frontmatter ID (e.g., `myproject-20260227-153000`) is used instead of the filename to ensure stable references even if files are renamed.

### Clicking Links in the Editor

Markdown links that point to note IDs (rather than URLs) are clickable in the editor. Clicking navigates to the target note within the same project.

---

## How It Works

### Frontend Components

```
ProjectNotesView.vue
├── MilkdownEditor.vue
│   └── MilkdownEditorCore.vue    ← /link detection, link insertion
│       └── LinkAutocomplete.vue  ← searchable note picker dropdown
└── BacklinksPanel.vue            ← forward links + backlinks display
```

**MilkdownEditorCore** watches for the `/link` pattern at the end of the editor content. When detected, it shows the `LinkAutocomplete` dropdown positioned near the editor. On selection, it replaces `/link` with the markdown link text and updates the editor view.

**BacklinksPanel** fetches links from the backend API whenever the note changes or is saved. It receives the note's frontmatter ID (not the filename) to query the link index.

**milkdown-link-handler.ts** intercepts clicks on links in the ProseMirror editor and routes to the target note via Vue Router instead of performing a browser navigation.

### Backend Link Index

The backend maintains an in-memory link index that maps each note ID to the list of notes linking to it. The index is:

- **Built on startup** by scanning all markdown files in `data/`.
- **Rebuilt on save** when a note is updated via the API.
- **Rebuilt on file change** when the file watcher detects external modifications.

#### Link Extraction

Two link patterns are recognized:

| Pattern | Example | Use Case |
|---------|---------|----------|
| Markdown links | `[Title](note-id)` | Created by `/link` command |
| Wiki-style links | `[[note-id]]` or `[[note-id\|Title]]` | Manual or Obsidian-style |

For markdown links, only targets that look like note IDs are indexed (alphanumeric with dashes/underscores, no URLs, no paths, at least 3 characters).

#### Index Structure

```
LINK_INDEX: HashMap<target_id, Vec<NoteLink>>
NOTE_TITLES: HashMap<note_id, title>
```

- `LINK_INDEX` is keyed by the **target** note ID. Each entry contains all links that point to that target.
- `NOTE_TITLES` maps every note ID to its title for display purposes.

Querying backlinks for note B: look up `LINK_INDEX["B"]`.
Querying forward links for note A: scan all entries for `source_id == "A"`.

### Note ID vs Filename

Project notes have two identifiers:

| Identifier | Example | Used For |
|------------|---------|----------|
| Filename | `20260227-153000` | File paths, route params |
| Frontmatter ID | `ironpad-20260227-153000` | Link targets, backlink index |

The backlinks API and BacklinksPanel always use the **frontmatter ID**. The frontmatter ID is `{project-slug}-{timestamp}` and is stored in the note's YAML frontmatter under `id:`.

---

## API Reference

### Get Links for a Note

Returns both backlinks (incoming) and forward links (outgoing).

```http
GET /api/backlinks/notes/:noteId/links
```

**Response:**

```json
{
  "note_id": "ironpad-20260227-153000",
  "backlinks": [
    {
      "source_id": "ironpad-20260226-100000",
      "source_title": "Project Overview",
      "source_path": "projects/ironpad/notes/20260226-100000.md",
      "context": "See [Meeting Notes](ironpad-20260227-153000) for details",
      "line_number": 5
    }
  ],
  "forward_links": [
    {
      "target_id": "ironpad-20260225-090000",
      "target_title": "Architecture Notes",
      "context": "Based on the [Architecture Notes](ironpad-20260225-090000)",
      "line_number": 12
    }
  ]
}
```

### Get Backlinks Only

```http
GET /api/backlinks/notes/:noteId/backlinks
```

### Get Forward Links Only

```http
GET /api/backlinks/notes/:noteId/forward-links
```

### Rebuild Link Index

Force a full rescan of all note files. Useful for debugging or after bulk edits.

```http
POST /api/backlinks/links/rebuild
```

**Response:**

```json
{
  "success": true,
  "indexed_notes": 42,
  "message": "Indexed 42 notes"
}
```

### Get Link Statistics

```http
GET /api/backlinks/links/stats
```

**Response:**

```json
{
  "total_links": 15,
  "unique_targets": 8
}
```

### Get Note Titles (for autocomplete)

Returns all notes in a project with their IDs and titles. Used by the `/link` dropdown.

```http
GET /api/projects/:projectId/notes-titles
```

**Response:**

```json
{
  "notes": [
    { "id": "ironpad-index", "title": "Ironpad", "path": "projects/ironpad/index.md" },
    { "id": "ironpad-20260227-153000", "title": "Meeting Notes", "path": "projects/ironpad/notes/20260227-153000.md" }
  ]
}
```

### Search Notes (for autocomplete)

Filter notes by partial title or ID match within a project.

```http
GET /api/projects/:projectId/notes-search?q=meeting&limit=10
```

**Response:**

```json
{
  "query": "meeting",
  "results": [
    { "id": "ironpad-20260227-153000", "title": "Meeting Notes", "path": "projects/ironpad/notes/20260227-153000.md" }
  ]
}
```

---

## Files

| File | Role |
|------|------|
| `frontend/src/components/MilkdownEditorCore.vue` | `/link` detection, link insertion, raw Crepe reference |
| `frontend/src/components/LinkAutocomplete.vue` | Searchable note picker dropdown |
| `frontend/src/components/BacklinksPanel.vue` | Forward links + backlinks display panel |
| `frontend/src/components/MilkdownEditor.vue` | Editor wrapper with force-remount support |
| `frontend/src/components/milkdown-link-handler.ts` | In-editor link click interception |
| `frontend/src/views/ProjectNotesView.vue` | Integrates editor + backlinks panel |
| `frontend/src/api/client.ts` | `backlinksApi` and `projectsApi` methods |
| `backend/src/services/backlinks.rs` | Link extraction, index, queries |
| `backend/src/routes/backlinks.rs` | Backlinks API endpoints |
| `backend/src/routes/projects.rs` | Notes-titles and notes-search endpoints |
| `backend/src/watcher.rs` | Triggers index rebuild on file changes |

---

## Technical Notes

### Crepe Raw Reference

Milkdown's `Crepe` class uses ES private fields. Vue's reactive proxy breaks getter access to these fields. The editor stores a raw (non-reactive) reference (`crepeRaw`) at creation time and uses it for all programmatic `editor.action()` calls. If the direct update fails, a `force-remount` fallback destroys and recreates the editor with the updated content.

### Index Rebuild Strategy

Currently, every save triggers a full index rebuild (scans all note files). This is simple and correct for small-to-medium datasets. For large datasets (thousands of notes), consider incremental updates that only reprocess the changed file.

### Which Files Are Indexed

The link index scans markdown files in:
- `data/projects/*/notes/*.md` — Project notes
- `data/projects/*/index.md` — Project index pages
- `data/daily/*.md` — Daily notes
- `data/*.md` — Root-level files (inbox, etc.)

It skips: task files, prompt files, archived files, and `.git` contents.
