# Ironpad - Chat Handover Document

**Date:** 2026-02-05  
**Context:** See `ai-context.md` for full project overview

---

## Session Summary

This session focused on fixing a critical bug where **notes and tasks displayed stale/wrong content** when switching between items. The issue caused data loss as the wrong content was being saved to the wrong files.

---

## What Was Fixed

### Problem: Stale Content When Switching Notes/Tasks

**Symptoms:**
- Click note A → shows content correctly
- Click note B → still shows note A's content
- Refresh sometimes fixes it, sometimes shows blank
- Auto-save then overwrites note B with note A's content (DATA LOSS)

**Root Cause:**
The Milkdown WYSIWYG editor wasn't properly recreating when switching items. Two issues:

1. **Module-level variables in `MilkdownEditorCore.vue`** - State like `currentContent` was persisting across component recreations because they were `let` variables instead of Vue `ref`s.

2. **Race condition in view components** - The editor key was changing BEFORE content was loaded:
   ```
   noteId changes → editor recreates with empty content → content loads → too late
   ```

**Solution:**
1. Converted module-level `let` variables to `ref`s in `MilkdownEditorCore.vue`
2. Added retry mechanism for applying pending content
3. Introduced separate `editorKey` ref in all view components that only updates AFTER content is loaded
4. Added guards to prevent emitting stale content

**Files Modified:**
- `frontend/src/components/MilkdownEditorCore.vue`
- `frontend/src/components/MilkdownEditor.vue`
- `frontend/src/views/ProjectNotesView.vue`
- `frontend/src/views/TasksView.vue`
- `frontend/src/views/NotesView.vue`

---

## Outstanding Issues

All major issues from this session have been resolved:

1. **Auto-save aggressiveness** - FIXED: Now tracks "last saved content" and only saves when actual changes are made
2. **Documentation** - FIXED: Added README.md, docs/ARCHITECTURE.md, docs/API.md

---

## Technical Context for Future Sessions

### Milkdown Editor Lifecycle (Critical Knowledge)

The Milkdown editor (WYSIWYG markdown) has a complex lifecycle:

1. `MilkdownProvider` provides Vue context
2. `useEditor` hook creates the `Crepe` instance
3. `Crepe.editor` is the actual Milkdown Editor
4. `editor.action(replaceAll(content))` updates content
5. BUT `editor.action` isn't immediately available after `useEditor` returns

**Key Pattern:** Always set content BEFORE changing the editor key:
```javascript
// CORRECT
editorContent.value = newContent
editorKey.value = newId  // Editor recreates with correct defaultValue

// WRONG
editorKey.value = newId  // Editor recreates with stale/empty content
editorContent.value = newContent  // Too late!
```

### Project Structure

```
ironpad/
├── backend/           # Rust Axum server (API only)
├── frontend/          # Vue 3 SPA
│   └── src/
│       ├── components/
│       │   ├── MilkdownEditor.vue      # Wrapper component
│       │   └── MilkdownEditorCore.vue  # Actual editor (key file!)
│       ├── views/
│       │   ├── NotesView.vue           # Standalone notes
│       │   ├── ProjectNotesView.vue    # Project-specific notes
│       │   └── TasksView.vue           # Project tasks
│       └── stores/                     # Pinia state management
└── data/              # Markdown files (source of truth)
```

---

## Recommended Next Steps

1. ~~**Fix auto-save aggressiveness**~~ - DONE: Uses `lastSavedContent` to track actual changes
2. ~~**Create proper README.md**~~ - DONE: See `/README.md`, `/frontend/README.md`
3. ~~**Add developer documentation**~~ - DONE: See `/docs/ARCHITECTURE.md`, `/docs/API.md`
4. **Consider adding tests** - At minimum, test the content switching logic

---

## Commands Reference

```bash
# Backend (from backend/)
cargo run              # API server on :3000

# Frontend (from frontend/)
npm run dev            # Dev server on :5173
npm run build          # Production build
```

---

## Notes

- Windows + PowerShell environment
- Files are the database (no SQL)
- Git auto-commits every 60 seconds
- See `ai-context.md` for full feature list and API endpoints
