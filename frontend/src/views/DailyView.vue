<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { dailyApi } from '../api/client'
import { useGitStore } from '../stores'
import type { DailyNote } from '../types'
import MilkdownEditor from '../components/MilkdownEditor.vue'

const props = defineProps<{
  date?: string
}>()

const route = useRoute()
const router = useRouter()
const gitStore = useGitStore()

const currentDate = computed((): string => {
  if (props.date) return props.date
  const routeDate = route.params.date
  if (typeof routeDate === 'string') return routeDate
  return getTodayDate()
})

// Note state
const dailyNote = ref<DailyNote | null>(null)
const editorContent = ref('')
const loading = ref(false)
const error = ref<string | null>(null)
const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
const noteExists = ref(false) // Track if the note file actually exists
const loadFailed = ref(false)
const lastSavedContent = ref<string | null>(null)
let saveTimeout: number | null = null
let userEdited = false
let loadedDate: string | null = null

// Default template for daily notes
function getDefaultTemplate(dateStr: string): string {
  // Parse date components directly to avoid timezone issues
  const [year, month, day] = dateStr.split('-').map(Number)
  const date = new Date(year, month - 1, day)
  const formatted = date.toLocaleDateString(undefined, {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
  return `# ${formatted}

## Today's Focus


## Notes


## Tasks

- [ ] 
`
}

function getTodayDate(): string {
  const now = new Date()
  return formatDateToString(now)
}

// Helper to format date to YYYY-MM-DD without timezone issues
function formatDateToString(date: Date): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function formatDateDisplay(dateStr: string): string {
  // Parse date components directly to avoid timezone issues
  const [year, month, day] = dateStr.split('-').map(Number)
  const date = new Date(year, month - 1, day)
  return date.toLocaleDateString(undefined, {
    weekday: 'long',
    year: 'numeric',
    month: 'long',
    day: 'numeric'
  })
}

function goToToday() {
  router.push({ name: 'daily' })
}

function goToPrevDay() {
  // Parse the current date from the route to avoid stale computed values
  const currentDateStr = typeof route.params.date === 'string' 
    ? route.params.date 
    : currentDate.value
  
  const [year, month, day] = currentDateStr.split('-').map(Number)
  const date = new Date(year, month - 1, day) // Use local date components directly
  date.setDate(date.getDate() - 1)
  
  const prevDate = formatDateToString(date)
  router.push({ name: 'daily-note', params: { date: prevDate } })
}

function goToNextDay() {
  // Parse the current date from the route to avoid stale computed values
  const currentDateStr = typeof route.params.date === 'string' 
    ? route.params.date 
    : currentDate.value
  
  const [year, month, day] = currentDateStr.split('-').map(Number)
  const date = new Date(year, month - 1, day) // Use local date components directly
  date.setDate(date.getDate() + 1)
  
  const nextDate = formatDateToString(date)
  router.push({ name: 'daily-note', params: { date: nextDate } })
}

function formatError(err: unknown): string {
  if (err instanceof TypeError && err.message === 'Failed to fetch') {
    return 'Could not reach the server'
  }
  if (err instanceof Error) return err.message
  return String(err)
}

function isMissingNote(err: unknown): boolean {
  const message = formatError(err)
  return /not found/i.test(message) || /\b404\b/.test(message)
}

function clearPendingSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveTimeout = null
  }
}

function scheduleAutoSave() {
  if (loadFailed.value || !userEdited) return
  clearPendingSave()
  saveStatus.value = 'idle'
  saveTimeout = window.setTimeout(() => {
    void saveNote(false)
  }, 1000)
}

function markUserEdited() {
  if (loading.value || loadFailed.value) return
  userEdited = true
}

async function persistNote(date: string, content: string, exists: boolean) {
  if (exists) {
    await dailyApi.update(date, content)
    return
  }
  await dailyApi.create(date, content)
}

async function saveNote(force = false) {
  clearPendingSave()
  if (loading.value || loadFailed.value) return

  const date = currentDate.value
  const content = editorContent.value
  const unchanged = content === lastSavedContent.value
  if (!force && (!userEdited || unchanged)) return
  if (force && unchanged && noteExists.value) {
    saveStatus.value = 'idle'
    return
  }
  if (!content.trim()) {
    saveStatus.value = 'idle'
    return
  }

  try {
    saveStatus.value = 'saving'
    await persistNote(date, content, noteExists.value)
    if (currentDate.value !== date) return
    noteExists.value = true
    lastSavedContent.value = content
    userEdited = false
    saveStatus.value = 'saved'
    gitStore.loadStatus()
    setTimeout(() => {
      if (saveStatus.value === 'saved') saveStatus.value = 'idle'
    }, 2000)
  } catch (err) {
    if (currentDate.value === date) {
      saveStatus.value = 'error'
      error.value = `Failed to save: ${formatError(err)}`
    }
  }
}

async function loadDailyNote() {
  const pendingDate = loadedDate
  const pendingContent = editorContent.value
  const pendingExists = noteExists.value
  const shouldFlush = userEdited && pendingDate !== null && pendingContent !== lastSavedContent.value

  clearPendingSave()
  userEdited = false
  loading.value = true
  error.value = null
  loadFailed.value = false
  noteExists.value = false

  if (shouldFlush && pendingDate) {
    try {
      await persistNote(pendingDate, pendingContent, pendingExists)
    } catch (err) {
      error.value = `Failed to save: ${formatError(err)}`
    }
  }

  loadedDate = currentDate.value

  try {
    dailyNote.value = await dailyApi.get(currentDate.value)
    editorContent.value = dailyNote.value?.content ?? ''
    noteExists.value = true
    lastSavedContent.value = editorContent.value
  } catch (err) {
    dailyNote.value = null
    noteExists.value = false
    if (isMissingNote(err)) {
      // Missing file is a new draft. Keep it local until the user edits.
      editorContent.value = getDefaultTemplate(currentDate.value)
      lastSavedContent.value = editorContent.value
    } else {
      editorContent.value = ''
      lastSavedContent.value = null
      loadFailed.value = true
      error.value = `Failed to load daily note: ${formatError(err)}`
    }
  } finally {
    loading.value = false
  }
}

// Editor normalization on open is not a user edit. Only save after the user changes the note.
watch(editorContent, (newContent, oldContent) => {
  if (loading.value || loadFailed.value) return
  if (!userEdited) {
    lastSavedContent.value = newContent
    return
  }
  if (oldContent !== undefined && newContent !== oldContent) {
    scheduleAutoSave()
  }
})

watch(currentDate, () => {
  loadDailyNote()
}, { immediate: true })
</script>

<template>
  <div class="daily-view">
    <div class="view-header">
      <div class="date-nav">
        <button @click="goToPrevDay" title="Previous day">←</button>
        <h2>{{ formatDateDisplay(currentDate) }}</h2>
        <button @click="goToNextDay" title="Next day">→</button>
      </div>
      <div class="button-group">
        <span :class="['status', saveStatus]">
          <template v-if="saveStatus === 'saving'">Saving...</template>
          <template v-else-if="saveStatus === 'saved'">Saved</template>
          <template v-else-if="saveStatus === 'error'">Save failed</template>
        </span>
        <span v-if="!loading && !loadFailed && !noteExists" class="note-status">Draft</span>
        <button @click="goToToday" v-if="currentDate !== getTodayDate()">Today</button>
        <button @click="saveNote(true)" :disabled="saveStatus === 'saving' || loading || loadFailed">Save</button>
      </div>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
      <button @click="error = null">Dismiss</button>
    </div>

    <div v-if="loading" class="loading">Loading daily note...</div>

    <div v-else-if="loadFailed" class="loading">
      <button @click="loadDailyNote">Retry</button>
    </div>
    
    <div v-else class="view-content">
      <div
        class="editor-container"
        @beforeinput="markUserEdited"
        @paste="markUserEdited"
        @drop="markUserEdited"
        @pointerdown="markUserEdited"
      >
        <MilkdownEditor
          v-model="editorContent"
          placeholder="What's on your mind today?"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.daily-view {
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

.date-nav {
  display: flex;
  align-items: center;
  gap: 12px;
}

.date-nav button {
  padding: 4px 8px;
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

.note-status {
  font-size: 11px;
  padding: 2px 8px;
  background: var(--color-bg-secondary);
  border-radius: 4px;
  color: var(--color-text-secondary);
}

.error-message {
  padding: 12px 16px;
  background: var(--color-danger);
  color: white;
  font-size: 13px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.error-message button {
  background: transparent;
  border: 1px solid white;
  color: white;
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

.loading {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  color: var(--color-text-secondary);
  padding: 32px;
}
</style>
