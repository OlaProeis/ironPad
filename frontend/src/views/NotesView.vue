<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useNotesStore, useWebSocketStore, useGitStore } from '../stores'
import MilkdownEditor from '../components/MilkdownEditor.vue'

const props = defineProps<{
  id?: string
}>()

const route = useRoute()
const router = useRouter()
const notesStore = useNotesStore()
const wsStore = useWebSocketStore()
const gitStore = useGitStore()

const editorContent = ref('')
let saveTimeout: number | null = null

// CRITICAL: Separate key for editor recreation - only update AFTER content is ready
const editorKey = ref<string | null>(null)

// Track the last saved/loaded content to detect actual user changes
// This prevents unnecessary saves when just opening a note
const lastSavedContent = ref<string | null>(null)

// Track which note ID the pending save is for
let pendingSaveNoteId: string | null = null

const noteId = computed(() => props.id || (route.params.id as string))
const selectedNote = computed(() => notesStore.getNoteById(noteId.value))

const isReadOnly = computed(() => {
  if (!notesStore.currentNote) return false
  return wsStore.isFileLockedByOther(notesStore.currentNote.path)
})

function clearPendingSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveTimeout = null
  }
  pendingSaveNoteId = null
}

// Save current content immediately before switching notes
async function saveBeforeSwitch() {
  const noteIdToSave = pendingSaveNoteId
  const contentToSave = editorContent.value
  const currentNoteId = notesStore.currentNote?.id
  
  console.log('[NotesView] saveBeforeSwitch called:', { 
    noteIdToSave, 
    currentNoteId, 
    contentLength: contentToSave?.length,
    hasCurrentNote: !!notesStore.currentNote 
  })
  
  // Clear pending state first
  clearPendingSave()
  
  // Only save if we had a pending save for the current note
  if (!noteIdToSave || !notesStore.currentNote) {
    console.log('[NotesView] Skipping save - no pending save or no current note')
    return
  }
  if (notesStore.currentNote.id !== noteIdToSave) {
    console.log('[NotesView] Skipping save - note ID mismatch:', { currentNoteId: notesStore.currentNote.id, noteIdToSave })
    return
  }
  
  // Only save if content actually changed
  if (contentToSave === lastSavedContent.value) {
    console.log('[NotesView] Skipping save before switch - content unchanged')
    return
  }
  
  console.log('[NotesView] Saving content before switch:', { noteIdToSave, contentLength: contentToSave.length })
  try {
    await notesStore.saveNote(contentToSave)
    lastSavedContent.value = contentToSave
    console.log('[NotesView] Save completed successfully')
  } catch (err) {
    console.error('[NotesView] Save failed:', err)
  }
}

// Auto-save with debounce
function scheduleAutoSave() {
  console.log('[NotesView] scheduleAutoSave called for note:', noteId.value)
  clearPendingSave()
  notesStore.saveStatus = 'idle'
  // Capture the current note ID for this save operation
  pendingSaveNoteId = noteId.value || null
  saveTimeout = window.setTimeout(async () => {
    const noteIdToSave = pendingSaveNoteId
    const contentToSave = editorContent.value
    
    // Clear pending state
    pendingSaveNoteId = null
    saveTimeout = null
    
    // Verify we're still on the same note - critical check to prevent overwrites
    if (!noteIdToSave || !notesStore.currentNote || noteId.value !== noteIdToSave) {
      console.log('[NotesView] Skipping save - note changed:', { noteIdToSave, currentNoteId: noteId.value })
      return
    }
    
    // Double-check the current note ID matches
    if (notesStore.currentNote.id !== noteIdToSave) {
      console.log('[NotesView] Skipping save - currentNote mismatch:', { noteIdToSave, currentNoteId: notesStore.currentNote.id })
      return
    }
    
    // Final check: only save if content actually changed from last saved
    if (contentToSave === lastSavedContent.value) {
      console.log('[NotesView] Skipping save - content unchanged from last save')
      return
    }
    
    try {
      await notesStore.saveNote(contentToSave)
      // Update last saved content on success
      lastSavedContent.value = contentToSave
      gitStore.loadStatus()
    } catch {
      // Error handled in store
    }
  }, 1000)
}

async function saveNote() {
  clearPendingSave()
  if (!notesStore.currentNote) return
  
  try {
    await notesStore.saveNote(editorContent.value)
    // Update last saved content on success
    lastSavedContent.value = editorContent.value
    gitStore.loadStatus()
  } catch {
    // Error handled in store
  }
}

async function deleteNote() {
  if (!confirm('Archive this note?')) return
  try {
    await notesStore.deleteNote()
    router.push({ name: 'home' })
  } catch {
    // Error handled in store
  }
}

// Watch for note changes
watch(noteId, async (newId, oldId) => {
  console.log('[NotesView] noteId changed:', { oldId, newId, pendingSaveNoteId })
  
  // Save any pending content from the previous note BEFORE switching
  if (oldId && pendingSaveNoteId) {
    console.log('[NotesView] Has pending save, calling saveBeforeSwitch')
    await saveBeforeSwitch()
  } else {
    console.log('[NotesView] No pending save, clearing')
    clearPendingSave()
  }
  notesStore.saveStatus = 'idle'
  
  if (newId) {
    console.log('[NotesView] Loading note:', newId)
    await notesStore.loadNote(newId)
    
    // CRITICAL: Set content BEFORE updating editorKey
    const loadedContent = notesStore.currentNote?.content ?? ''
    console.log('[NotesView] Setting editor content, length:', loadedContent.length)
    editorContent.value = loadedContent
    
    // Track this as the "original" content - only save if user makes changes
    lastSavedContent.value = loadedContent
    
    // NOW update the editor key - this triggers editor recreation with correct content
    editorKey.value = newId
    console.log('[NotesView] Updated editorKey to:', newId)
  } else {
    notesStore.clearCurrentNote()
    editorContent.value = ''
    lastSavedContent.value = null
    editorKey.value = null
  }
}, { immediate: true })

// Watch for content changes - ONLY save when content differs from last saved
watch(editorContent, (newContent) => {
  // Skip if no note loaded or read-only
  if (!notesStore.currentNote || isReadOnly.value) {
    return
  }
  
  // CRITICAL: Only schedule auto-save if content actually differs from last saved/loaded
  // This prevents unnecessary saves when just opening a note
  if (lastSavedContent.value !== null && newContent !== lastSavedContent.value) {
    console.log('[NotesView] Content changed from last saved, scheduling auto-save')
    scheduleAutoSave()
  }
})

// Milkdown is WYSIWYG - no separate preview needed
</script>

<template>
  <div class="notes-view">
    <template v-if="notesStore.currentNote && editorKey">
      <div class="view-header">
        <h2>{{ selectedNote?.title ?? notesStore.currentNote.id }}</h2>
        <div class="button-group">
          <span :class="['status', notesStore.saveStatus]">
            <template v-if="notesStore.saveStatus === 'saving'">Saving...</template>
            <template v-else-if="notesStore.saveStatus === 'saved'">Saved</template>
            <template v-else-if="notesStore.saveStatus === 'error'">Save failed</template>
          </span>
          <span v-if="wsStore.connected" class="ws-status connected" title="Real-time sync active">●</span>
          <span v-else class="ws-status" title="Connecting...">○</span>
          <button @click="saveNote" :disabled="notesStore.saveStatus === 'saving'">Save</button>
          <button class="danger" @click="deleteNote">Archive</button>
        </div>
      </div>

      <div v-if="isReadOnly" class="read-only-banner">
        🔒 This file is being edited elsewhere. Read-only mode.
      </div>

      <div class="view-content">
        <div v-if="notesStore.loadingNote" class="loading">Loading note...</div>
        <div v-else class="editor-container">
          <MilkdownEditor
            v-model="editorContent"
            :editor-key="editorKey"
            :readonly="isReadOnly"
            placeholder="Start writing..."
          />
        </div>
      </div>
    </template>

    <div v-else class="empty-state">
      <h2>No note selected</h2>
      <p>Select a note from the sidebar or create a new one.</p>
      <p class="shortcuts">
        <kbd>Ctrl+K</kbd> Search · <kbd>Ctrl+S</kbd> Save
      </p>
      <button class="primary" @click="notesStore.createNote().then(n => router.push({ name: 'note', params: { id: n.id } }))" style="margin-top: 16px">
        Create Note
      </button>
    </div>
  </div>
</template>

<style scoped>
.notes-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.view-header {
  height: var(--header-height);
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.view-header h2 {
  font-size: 14px;
  font-weight: 500;
  margin: 0;
}

.button-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.status {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.status.saving { color: var(--color-primary); }
.status.saved { color: var(--color-success); }
.status.error { color: var(--color-danger); }

.ws-status {
  font-size: 10px;
  color: var(--color-text-secondary);
}

.ws-status.connected {
  color: var(--color-success);
}

.read-only-banner {
  padding: 8px 16px;
  background: var(--color-primary);
  color: white;
  font-size: 13px;
}

.view-content {
  flex: 1;
  overflow: hidden;
  display: flex;
}

.editor-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  color: var(--color-text-secondary);
  padding: 32px;
}

.empty-state h2 {
  margin-bottom: 8px;
  font-weight: 500;
}

.shortcuts {
  margin-top: 16px;
  font-size: 12px;
}

.shortcuts kbd {
  background: var(--color-border);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.loading {
  padding: 16px;
  color: var(--color-text-secondary);
  text-align: center;
}
</style>
