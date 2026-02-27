<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useWorkspaceStore, useGitStore } from '../stores'
import { projectsApi } from '../api/client'
import type { ProjectNote, ProjectNoteWithContent } from '../types'
import MilkdownEditor from '../components/MilkdownEditor.vue'
import BacklinksPanel from '../components/BacklinksPanel.vue'

const props = defineProps<{
  id: string
  noteId?: string
}>()

const route = useRoute()
const router = useRouter()
const workspaceStore = useWorkspaceStore()
const gitStore = useGitStore()

const projectId = computed(() => props.id || (route.params.id as string))
const currentNoteId = computed(() => props.noteId || (route.params.noteId as string | undefined))

// Notes list state
const notes = ref<ProjectNote[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

// Editor state
const selectedNote = ref<ProjectNoteWithContent | null>(null)
const editorContent = ref('')
const editorLoading = ref(false)
const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
const backlinksRefreshKey = ref(0)
let saveTimeout: number | null = null

// CRITICAL: Separate key for editor recreation - only update AFTER content is ready
// This prevents the race condition where the editor recreates before content loads
const editorKey = ref<string | null>(null)

// Track the last saved/loaded content to detect actual user changes
// This prevents unnecessary saves when just opening a note
const lastSavedContent = ref<string | null>(null)

// Track which note ID the pending save is for
let pendingSaveNoteId: string | null = null

async function loadNotes() {
  loading.value = true
  error.value = null
  try {
    notes.value = await projectsApi.listNotes(projectId.value)
  } catch (err) {
    error.value = `Failed to load notes: ${err}`
  } finally {
    loading.value = false
  }
}

async function loadNote(noteId: string) {
  editorLoading.value = true
  try {
    selectedNote.value = await projectsApi.getNote(projectId.value, noteId)
    
    // Set content BEFORE updating editorKey so the editor recreates with correct defaultValue
    const newContent = selectedNote.value?.content ?? ''
    editorContent.value = newContent
    lastSavedContent.value = newContent
    editorKey.value = noteId
  } catch {
    selectedNote.value = null
    editorContent.value = ''
    lastSavedContent.value = null
    editorKey.value = null
  } finally {
    editorLoading.value = false
  }
}

async function createNote() {
  const title = prompt('Note title (optional):')
  try {
    const note = await projectsApi.createNote(projectId.value, title || undefined)
    await loadNotes() // Refresh list
    // Select the new note
    const filename = note.path.split('/').pop()?.replace('.md', '')
    if (filename) {
      router.push({ name: 'project-notes', params: { id: projectId.value, noteId: filename } })
    }
  } catch (err) {
    error.value = `Failed to create note: ${err}`
  }
}

function selectNote(note: ProjectNote) {
  const filename = note.path.split('/').pop()?.replace('.md', '')
  if (filename) {
    router.push({ name: 'project-notes', params: { id: projectId.value, noteId: filename } })
  }
}

function clearPendingSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveTimeout = null
  }
  pendingSaveNoteId = null
}

async function saveBeforeSwitch() {
  const noteIdToSave = pendingSaveNoteId
  const contentToSave = editorContent.value
  
  clearPendingSave()
  
  if (!noteIdToSave || !selectedNote.value) return
  if (contentToSave === lastSavedContent.value) return
  
  try {
    await projectsApi.updateNote(projectId.value, noteIdToSave, contentToSave)
    lastSavedContent.value = contentToSave
    gitStore.loadStatus()
  } catch (err) {
    console.error('[ProjectNotesView] Save failed:', err)
  }
}

function scheduleAutoSave() {
  clearPendingSave()
  saveStatus.value = 'idle'
  pendingSaveNoteId = currentNoteId.value || null
  saveTimeout = window.setTimeout(saveNoteContent, 1000)
}

async function saveNoteContent() {
  const noteIdToSave = pendingSaveNoteId
  const contentToSave = editorContent.value
  
  pendingSaveNoteId = null
  saveTimeout = null
  
  if (!noteIdToSave || !selectedNote.value || currentNoteId.value !== noteIdToSave) return
  if (contentToSave === lastSavedContent.value) return
  
  try {
    saveStatus.value = 'saving'
    const savedNote = await projectsApi.updateNote(projectId.value, noteIdToSave, contentToSave)
    
    if (currentNoteId.value === noteIdToSave) {
      selectedNote.value = savedNote
      lastSavedContent.value = contentToSave
      saveStatus.value = 'saved'
      backlinksRefreshKey.value++
      setTimeout(() => {
        if (saveStatus.value === 'saved') saveStatus.value = 'idle'
      }, 2000)
    }
    gitStore.loadStatus()
    await loadNotes()
  } catch (err) {
    if (currentNoteId.value === noteIdToSave) {
      saveStatus.value = 'error'
    }
  }
}

async function saveNote() {
  clearPendingSave()
  if (!selectedNote.value || !currentNoteId.value) return
  
  try {
    saveStatus.value = 'saving'
    selectedNote.value = await projectsApi.updateNote(projectId.value, currentNoteId.value, editorContent.value)
    lastSavedContent.value = editorContent.value
    saveStatus.value = 'saved'
    backlinksRefreshKey.value++
    gitStore.loadStatus()
    await loadNotes()
    setTimeout(() => {
      if (saveStatus.value === 'saved') saveStatus.value = 'idle'
    }, 2000)
  } catch (err) {
    saveStatus.value = 'error'
  }
}

async function deleteNote() {
  if (!selectedNote.value || !currentNoteId.value) return
  if (!confirm('Are you sure you want to delete this note?')) return
  
  try {
    await projectsApi.deleteNote(projectId.value, currentNoteId.value)
    selectedNote.value = null
    editorContent.value = ''
    router.push({ name: 'project-notes', params: { id: projectId.value } })
    await loadNotes()
  } catch (err) {
    alert(`Failed to delete note: ${err}`)
  }
}

function formatDate(dateStr: string) {
  if (!dateStr) return ''
  try {
    return new Date(dateStr).toLocaleDateString()
  } catch {
    return dateStr
  }
}

function isSelected(note: ProjectNote) {
  const filename = note.path.split('/').pop()?.replace('.md', '')
  return filename === currentNoteId.value
}

watch(editorContent, (newContent) => {
  if (!selectedNote.value) return
  
  if (lastSavedContent.value !== null && newContent !== lastSavedContent.value) {
    scheduleAutoSave()
  }
})

watch(projectId, () => {
  // Clear any pending saves when switching projects
  clearPendingSave()
  editorContent.value = ''
  lastSavedContent.value = null
  selectedNote.value = null
  
  loadNotes()
}, { immediate: true })

watch(currentNoteId, async (noteId, oldNoteId) => {
  if (oldNoteId && pendingSaveNoteId) {
    await saveBeforeSwitch()
  } else {
    clearPendingSave()
  }
  saveStatus.value = 'idle'
  
  if (noteId) {
    await loadNote(noteId)
  } else {
    selectedNote.value = null
    editorContent.value = ''
    lastSavedContent.value = null
    editorKey.value = null
  }
}, { immediate: true })

onMounted(() => {
  if (workspaceStore.activeProjectId !== projectId.value) {
    workspaceStore.setActiveProject(projectId.value)
  }
})
</script>

<template>
  <div class="notes-split-view">
    <!-- Notes List Panel -->
    <div class="notes-list-panel">
      <div class="panel-header">
        <h3>Notes</h3>
        <button class="primary small" @click="createNote">+ New</button>
      </div>

      <div v-if="loading" class="loading-small">Loading...</div>
      
      <div v-else-if="notes.length === 0" class="empty-list">
        <p>No notes yet</p>
      </div>

      <div v-else class="notes-list">
        <div
          v-for="note in notes"
          :key="note.id"
          :class="['note-item', { selected: isSelected(note) }]"
          @click="selectNote(note)"
        >
          <div class="note-title">{{ note.title || 'Untitled' }}</div>
          <div class="note-meta">{{ formatDate(note.updated) }}</div>
        </div>
      </div>
    </div>

    <!-- Editor Panel -->
    <div class="editor-panel">
      <template v-if="currentNoteId && selectedNote && editorKey">
        <div class="editor-header">
          <h3>{{ selectedNote.title || 'Untitled' }}</h3>
          <div class="editor-actions">
            <span :class="['status', saveStatus]">
              <template v-if="saveStatus === 'saving'">Saving...</template>
              <template v-else-if="saveStatus === 'saved'">Saved</template>
              <template v-else-if="saveStatus === 'error'">Error</template>
            </span>
            <button @click="saveNote" :disabled="saveStatus === 'saving'">Save</button>
            <button class="danger" @click="deleteNote">Delete</button>
          </div>
        </div>
        <div class="editor-content">
          <MilkdownEditor
            v-model="editorContent"
            :editor-key="editorKey"
            :project-id="projectId"
            placeholder="Write your note..."
          />
        </div>
        <BacklinksPanel v-if="selectedNote" :note-id="selectedNote.id" :project-id="projectId" :refresh-key="backlinksRefreshKey" />
      </template>

      <template v-else-if="editorLoading">
        <div class="editor-placeholder">
          <p>Loading note...</p>
        </div>
      </template>

      <template v-else>
        <div class="editor-placeholder">
          <h3>Select a note</h3>
          <p>Choose a note from the list or create a new one.</p>
          <button class="primary" @click="createNote">+ New Note</button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.notes-split-view {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-width: 0;
}

/* Notes List Panel */
.notes-list-panel {
  width: 280px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  background: var(--color-bg-secondary);
  overflow: hidden;
}

.panel-header {
  height: 52px;
  min-height: 52px;
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.panel-header h3 {
  font-size: 13px;
  font-weight: 600;
  margin: 0;
}

button.small {
  padding: 4px 10px;
  font-size: 12px;
}

.notes-list {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.note-item {
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  cursor: pointer;
  transition: background 0.15s;
}

.note-item:hover {
  background: var(--color-border);
}

.note-item.selected {
  background: var(--color-primary);
  color: white;
}

.note-item.selected .note-meta {
  color: rgba(255, 255, 255, 0.7);
}

.note-title {
  font-weight: 500;
  font-size: 13px;
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-meta {
  font-size: 11px;
  color: var(--color-text-secondary);
}

.loading-small,
.empty-list {
  padding: 24px 16px;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 13px;
}

/* Editor Panel */
.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.editor-header {
  height: 52px;
  min-height: 52px;
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.editor-header h3 {
  font-size: 14px;
  font-weight: 500;
  margin: 0;
}

.editor-actions {
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

.editor-content {
  flex: 1;
  overflow: hidden;
  display: flex;
}

.editor-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  color: var(--color-text-secondary);
  padding: 32px;
}

.editor-placeholder h3 {
  margin-bottom: 8px;
  color: var(--color-text);
}

.editor-placeholder p {
  margin-bottom: 16px;
}
</style>
