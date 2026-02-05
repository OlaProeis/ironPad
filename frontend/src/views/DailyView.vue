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
let saveTimeout: number | null = null

// Default template for daily notes
function getDefaultTemplate(dateStr: string): string {
  const date = new Date(dateStr + 'T00:00:00')
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
  return new Date().toISOString().split('T')[0] as string
}

function formatDateDisplay(dateStr: string): string {
  const date = new Date(dateStr + 'T00:00:00')
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
  const date = new Date(currentDate.value + 'T00:00:00')
  date.setDate(date.getDate() - 1)
  const prevDate = date.toISOString().split('T')[0]
  router.push({ name: 'daily-note', params: { date: prevDate } })
}

function goToNextDay() {
  const date = new Date(currentDate.value + 'T00:00:00')
  date.setDate(date.getDate() + 1)
  const nextDate = date.toISOString().split('T')[0]
  router.push({ name: 'daily-note', params: { date: nextDate } })
}

function scheduleAutoSave() {
  if (saveTimeout) clearTimeout(saveTimeout)
  saveStatus.value = 'idle'
  saveTimeout = window.setTimeout(saveNote, 1000)
}

async function saveNote() {
  // Don't save if content is just the template or empty
  const template = getDefaultTemplate(currentDate.value)
  const trimmedContent = editorContent.value.trim()
  const trimmedTemplate = template.trim()
  
  if (!trimmedContent || trimmedContent === trimmedTemplate) {
    // Don't save empty/template-only content
    saveStatus.value = 'idle'
    return
  }
  
  try {
    saveStatus.value = 'saving'
    
    if (!noteExists.value) {
      // Create the note first
      dailyNote.value = await dailyApi.create(currentDate.value, editorContent.value)
      noteExists.value = true
    } else {
      // Update existing note using the date
      await dailyApi.update(currentDate.value, editorContent.value)
    }
    
    saveStatus.value = 'saved'
    gitStore.loadStatus()
    setTimeout(() => {
      if (saveStatus.value === 'saved') saveStatus.value = 'idle'
    }, 2000)
  } catch (err) {
    saveStatus.value = 'error'
    error.value = `Failed to save: ${err}`
  }
}

async function loadDailyNote() {
  loading.value = true
  error.value = null
  noteExists.value = false
  
  try {
    // Try to get existing daily note (don't auto-create)
    dailyNote.value = await dailyApi.get(currentDate.value)
    editorContent.value = dailyNote.value?.content ?? ''
    noteExists.value = true
  } catch (err) {
    // Note doesn't exist - that's fine, show template but don't create file
    dailyNote.value = null
    editorContent.value = getDefaultTemplate(currentDate.value)
    noteExists.value = false
  } finally {
    loading.value = false
  }
}

// Watch for content changes - auto-save
watch(editorContent, (newContent, oldContent) => {
  // Only trigger auto-save if content actually changed (not on initial load)
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
        <span v-if="!noteExists" class="note-status">Draft</span>
        <button @click="goToToday" v-if="currentDate !== getTodayDate()">Today</button>
        <button @click="saveNote" :disabled="saveStatus === 'saving'">Save</button>
      </div>
    </div>

    <div v-if="error" class="error-message">
      {{ error }}
      <button @click="error = null">Dismiss</button>
    </div>

    <div v-if="loading" class="loading">Loading daily note...</div>
    
    <div v-else class="view-content">
      <div class="editor-container">
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
