<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useNotesStore } from '../stores'

const router = useRouter()
const route = useRoute()
const notesStore = useNotesStore()

const selectedNoteId = computed(() => route.params.id as string | undefined)

function selectNote(id: string) {
  router.push({ name: 'note', params: { id } })
}

function formatDate(dateStr?: string): string {
  if (!dateStr) return ''
  try {
    const date = new Date(dateStr)
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    })
  } catch {
    return dateStr
  }
}
</script>

<template>
  <div v-if="notesStore.loading" class="loading">Loading notes...</div>
  <div v-else-if="notesStore.sortedNotes.length === 0" class="empty">No notes yet</div>
  <ul v-else class="note-list">
    <li
      v-for="note in notesStore.sortedNotes"
      :key="note.id"
      :class="['note-item', { active: note.id === selectedNoteId }]"
      @click="selectNote(note.id)"
    >
      <div class="note-item-title">{{ note.title }}</div>
      <div class="note-item-meta">
        <span class="type-badge">{{ note.note_type }}</span>
        <span v-if="note.updated"> · {{ formatDate(note.updated) }}</span>
      </div>
    </li>
  </ul>
</template>

<style scoped>
.note-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.note-item {
  padding: 10px 16px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background 0.15s, border-color 0.15s;
}

.note-item:hover {
  background: var(--color-border);
}

.note-item.active {
  background: var(--color-border);
  border-left-color: var(--color-primary);
}

.note-item-title {
  font-weight: 500;
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.note-item-meta {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.type-badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  background: var(--color-border);
  color: var(--color-text-secondary);
}

.loading,
.empty {
  padding: 16px;
  color: var(--color-text-secondary);
  text-align: center;
}
</style>
