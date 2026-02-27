<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { projectsApi } from '../api/client'
import type { NoteTitleEntry } from '../types'

const props = defineProps<{
  projectId: string
  visible: boolean
  anchorPosition?: { x: number; y: number }
}>()

const emit = defineEmits<{
  select: [note: NoteTitleEntry]
  cancel: []
}>()

// State
const notes = ref<NoteTitleEntry[]>([])
const searchQuery = ref('')
const loading = ref(false)
const selectedIndex = ref(0)
const inputRef = ref<HTMLInputElement | null>(null)
const containerRef = ref<HTMLElement | null>(null)

// Filtered notes based on search
const filteredNotes = computed(() => {
  if (!searchQuery.value.trim()) {
    return notes.value
  }
  const query = searchQuery.value.toLowerCase()
  return notes.value.filter((n: NoteTitleEntry) => 
    n.title.toLowerCase().includes(query) || 
    n.id.toLowerCase().includes(query)
  )
})

watch(() => props.visible, async (visible) => {
  if (visible) {
    await loadNotes()
    selectedIndex.value = 0
    searchQuery.value = ''
    nextTick(() => {
      inputRef.value?.focus()
    })
  }
}, { immediate: true })

// Reset selection when search changes
watch(searchQuery, () => {
  selectedIndex.value = 0
})

// Watch for filtered notes changing
watch(filteredNotes, (filtered) => {
  if (selectedIndex.value >= filtered.length) {
    selectedIndex.value = Math.max(0, filtered.length - 1)
  }
})

// Load notes from API
async function loadNotes() {
  if (!props.projectId) return
  
  loading.value = true
  try {
    const response = await projectsApi.getNotesTitles(props.projectId)
    notes.value = response.notes || []
  } catch (err) {
    console.error('[LinkAutocomplete] Failed to load notes:', err)
    notes.value = []
  } finally {
    loading.value = false
  }
}

// Handle keyboard navigation
function handleKeydown(e: KeyboardEvent) {
  const filtered = filteredNotes.value
  
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value + 1) % filtered.length
      scrollToSelected()
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = (selectedIndex.value - 1 + filtered.length) % filtered.length
      scrollToSelected()
      break
    case 'Enter':
      e.preventDefault()
      if (filtered.length > 0 && selectedIndex.value >= 0) {
        const selected = filtered[selectedIndex.value]
        if (selected) {
          selectNote(selected)
        }
      }
      break
    case 'Escape':
      e.preventDefault()
      emit('cancel')
      break
  }
}

// Scroll selected item into view
function scrollToSelected() {
  nextTick(() => {
    const selectedEl = containerRef.value?.querySelector('.link-item.selected')
    selectedEl?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
  })
}

// Select a note
function selectNote(note: NoteTitleEntry) {
  emit('select', note)
}

// Click outside to close
function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (containerRef.value && !containerRef.value.contains(target)) {
    emit('cancel')
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div
    v-if="visible"
    ref="containerRef"
    class="link-autocomplete"
    :style="anchorPosition ? {
      position: 'fixed',
      left: `${anchorPosition.x}px`,
      top: `${anchorPosition.y}px`
    } : {}"
    @keydown="handleKeydown"
  >
    <div class="autocomplete-header">
      <input
        ref="inputRef"
        v-model="searchQuery"
        type="text"
        class="search-input"
        placeholder="Search notes..."
        @keydown="handleKeydown"
      />
    </div>
    
    <div class="autocomplete-list">
      <div v-if="loading" class="loading">Loading notes...</div>
      <div v-else-if="filteredNotes.length === 0" class="empty">
        {{ searchQuery ? 'No notes found' : 'No notes in project' }}
      </div>
      <template v-else>
        <div
          v-for="(note, index) in filteredNotes"
          :key="note.id"
          class="link-item"
          :class="{ selected: index === selectedIndex }"
          @click="selectNote(note)"
          @mouseenter="selectedIndex = index"
        >
          <div class="note-title">{{ note.title }}</div>
          <div class="note-id">{{ note.id }}</div>
        </div>
      </template>
    </div>
    
    <div class="autocomplete-footer">
      <span class="hint">↑↓ to navigate, Enter to select, Esc to cancel</span>
    </div>
  </div>
</template>

<style scoped>
.link-autocomplete {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  width: 320px;
  max-height: 400px;
  display: flex;
  flex-direction: column;
  z-index: 1000;
}

.autocomplete-header {
  padding: 8px;
  border-bottom: 1px solid var(--color-border);
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 14px;
  outline: none;
}

.search-input:focus {
  border-color: var(--color-primary);
}

.autocomplete-list {
  flex: 1;
  overflow-y: auto;
  max-height: 300px;
}

.loading,
.empty {
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.link-item {
  padding: 10px 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--color-border);
  transition: background 0.15s;
}

.link-item:last-child {
  border-bottom: none;
}

.link-item:hover,
.link-item.selected {
  background: var(--color-bg-secondary);
}

.note-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.note-id {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
  font-family: var(--font-mono);
}

.autocomplete-footer {
  padding: 6px 12px;
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  border-radius: 0 0 8px 8px;
}

.hint {
  font-size: 11px;
  color: var(--color-text-secondary);
}
</style>
