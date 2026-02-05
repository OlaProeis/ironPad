import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SearchResult } from '../types'
import { searchApi } from '../api/client'

export const useUiStore = defineStore('ui', () => {
  // State
  const showSearch = ref(false)
  const showTasks = ref(false)
  const showPreview = ref(false)
  const searchQuery = ref('')
  const searchResults = ref<SearchResult[]>([])
  const isSearching = ref(false)
  const globalError = ref<string | null>(null)
  const sidebarSection = ref<'notes' | 'projects' | 'daily'>('notes')

  // Getters
  const hasSearchResults = computed(() => searchResults.value.length > 0)

  // Actions
  function openSearch() {
    showSearch.value = true
    showTasks.value = false
  }

  function closeSearch() {
    showSearch.value = false
    searchQuery.value = ''
    searchResults.value = []
  }

  function toggleSearch() {
    if (showSearch.value) {
      closeSearch()
    } else {
      openSearch()
    }
  }

  function openTasks() {
    showTasks.value = true
    showSearch.value = false
  }

  function closeTasks() {
    showTasks.value = false
  }

  function toggleTasks() {
    if (showTasks.value) {
      closeTasks()
    } else {
      openTasks()
    }
  }

  function togglePreview() {
    showPreview.value = !showPreview.value
    // Persist preference
    localStorage.setItem('ironpad-show-preview', String(showPreview.value))
  }

  function loadPreviewPreference() {
    const saved = localStorage.getItem('ironpad-show-preview')
    if (saved !== null) {
      showPreview.value = saved === 'true'
    }
  }

  async function search(query: string) {
    if (query.length < 2) {
      searchResults.value = []
      return
    }

    try {
      isSearching.value = true
      searchQuery.value = query
      searchResults.value = await searchApi.search(query)
    } catch (err) {
      globalError.value = `Search failed: ${err}`
    } finally {
      isSearching.value = false
    }
  }

  function setSidebarSection(section: 'notes' | 'projects' | 'daily') {
    sidebarSection.value = section
  }

  function setGlobalError(message: string | null) {
    globalError.value = message
  }

  function clearGlobalError() {
    globalError.value = null
  }

  return {
    // State
    showSearch,
    showTasks,
    showPreview,
    searchQuery,
    searchResults,
    isSearching,
    globalError,
    sidebarSection,
    // Getters
    hasSearchResults,
    // Actions
    openSearch,
    closeSearch,
    toggleSearch,
    openTasks,
    closeTasks,
    toggleTasks,
    togglePreview,
    loadPreviewPreference,
    search,
    setSidebarSection,
    setGlobalError,
    clearGlobalError
  }
})
