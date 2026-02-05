<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useNotesStore, useUiStore } from '../stores'

const router = useRouter()
const notesStore = useNotesStore()
const uiStore = useUiStore()

const localQuery = ref('')
let searchTimeout: number | null = null

watch(localQuery, (query) => {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = window.setTimeout(() => {
    uiStore.search(query)
  }, 300)
})

function openSearchResult(result: { path: string }) {
  // Find note by path
  const note = notesStore.notes.find(n => n.path === result.path)
  if (note) {
    router.push({ name: 'note', params: { id: note.id } })
    uiStore.closeSearch()
  }
}
</script>

<template>
  <div class="search-panel">
    <input
      v-model="localQuery"
      type="text"
      placeholder="Search notes..."
      class="search-input"
      autofocus
    />
    <div v-if="uiStore.isSearching" class="loading">Searching...</div>
    <div v-else-if="uiStore.searchResults.length" class="search-results">
      <div
        v-for="result in uiStore.searchResults"
        :key="result.path"
        class="search-result"
        @click="openSearchResult(result)"
      >
        <div class="search-result-title">{{ result.title }}</div>
        <div 
          v-for="match in result.matches.slice(0, 2)" 
          :key="match.line_number" 
          class="search-result-match"
        >
          <span class="line-num">{{ match.line_number }}:</span>
          {{ match.line_content.slice(0, 80) }}
        </div>
      </div>
    </div>
    <div v-else-if="localQuery.length >= 2" class="no-results">No results found</div>
  </div>
</template>

<style scoped>
.search-panel {
  padding: 12px;
  border-bottom: 1px solid var(--color-border);
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.search-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 14px;
  outline: none;
}

.search-input:focus {
  border-color: var(--color-primary);
}

.search-results {
  margin-top: 12px;
  flex: 1;
  overflow-y: auto;
}

.search-result {
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
}

.search-result:hover {
  background: var(--color-border);
}

.search-result-title {
  font-weight: 500;
  margin-bottom: 4px;
}

.search-result-match {
  font-size: 12px;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.line-num {
  color: var(--color-primary);
  margin-right: 4px;
}

.no-results,
.loading {
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary);
}
</style>
