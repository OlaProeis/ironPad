import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { Prompt, PromptFolder, PromptSearchResult, PromptSummary } from '../types'
import { promptsApi } from '../api/client'

export const usePromptsStore = defineStore('prompts', () => {
  const prompts = ref<PromptSummary[]>([])
  const selectedPrompt = ref<Prompt | null>(null)
  const folders = ref<PromptFolder[]>([])
  const semanticResults = ref<PromptSearchResult[]>([])
  const loading = ref(false)
  const loadingPrompt = ref(false)
  const searching = ref(false)
  const error = ref<string | null>(null)
  const activeScope = ref<'global' | 'project'>('global')
  const activeProjectId = ref<string | null>(null)
  const activeFolder = ref<string>('root')
  const activeTag = ref<string | null>(null)
  const query = ref('')
  const semanticMode = ref(true)

  const filteredPrompts = computed(() =>
    prompts.value.filter((p) => {
      if (activeFolder.value !== 'root' && p.folder !== activeFolder.value) return false
      if (activeTag.value && !p.tags.includes(activeTag.value)) return false
      return true
    })
  )

  const visiblePrompts = computed(() => {
    if (query.value.trim() && semanticMode.value && semanticResults.value.length > 0) {
      return semanticResults.value.map((r) => r.prompt)
    }
    return filteredPrompts.value
  })

  const allTags = computed(() => {
    const set = new Set<string>()
    for (const p of prompts.value) {
      for (const t of p.tags) set.add(t)
    }
    return [...set].sort()
  })

  async function loadPrompts() {
    try {
      loading.value = true
      error.value = null
      prompts.value = await promptsApi.list({
        scope: activeScope.value,
        project_id: activeProjectId.value ?? undefined
      })
    } catch (err) {
      error.value = `Failed to load prompts: ${err}`
    } finally {
      loading.value = false
    }
  }

  async function loadFolders() {
    try {
      folders.value = await promptsApi.folders({
        scope: activeScope.value,
        project_id: activeProjectId.value ?? undefined
      })
    } catch {
      folders.value = []
    }
  }

  async function loadPrompt(id: string) {
    try {
      loadingPrompt.value = true
      error.value = null
      selectedPrompt.value = await promptsApi.get(id)
    } catch (err) {
      error.value = `Failed to load prompt: ${err}`
      selectedPrompt.value = null
    } finally {
      loadingPrompt.value = false
    }
  }

  async function createPrompt(payload: {
    title?: string
    folder?: string
    tags?: string[]
    description?: string
    content?: string
  }) {
    const created = await promptsApi.create({
      scope: activeScope.value,
      project_id: activeScope.value === 'project' ? activeProjectId.value ?? undefined : undefined,
      ...payload
    })
    await Promise.all([loadPrompts(), loadFolders()])
    selectedPrompt.value = created
    return created
  }

  async function updatePrompt(id: string, payload: {
    title?: string
    folder?: string
    tags?: string[]
    description?: string
    content?: string
  }) {
    const updated = await promptsApi.update(id, payload)
    selectedPrompt.value = updated
    await Promise.all([loadPrompts(), loadFolders()])
    return updated
  }

  async function deletePrompt(id: string) {
    await promptsApi.delete(id)
    if (selectedPrompt.value?.id === id) {
      selectedPrompt.value = null
    }
    await Promise.all([loadPrompts(), loadFolders()])
  }

  async function runSemanticSearch() {
    const q = query.value.trim()
    if (!q) {
      semanticResults.value = []
      return
    }
    try {
      searching.value = true
      error.value = null
      semanticResults.value = await promptsApi.semanticSearch({
        q,
        scope: activeScope.value,
        project_id: activeScope.value === 'project' ? activeProjectId.value ?? undefined : undefined,
        folder: activeFolder.value === 'root' ? undefined : activeFolder.value,
        tag: activeTag.value ?? undefined,
        limit: 40
      })
    } catch (err) {
      error.value = `Semantic search failed: ${err}`
      semanticResults.value = []
    } finally {
      searching.value = false
    }
  }

  async function setScope(scope: 'global' | 'project', projectId?: string | null) {
    activeScope.value = scope
    activeProjectId.value = scope === 'project' ? (projectId ?? null) : null
    activeFolder.value = 'root'
    activeTag.value = null
    semanticResults.value = []
    await Promise.all([loadPrompts(), loadFolders()])
  }

  function clearError() {
    error.value = null
  }

  return {
    prompts,
    selectedPrompt,
    folders,
    semanticResults,
    loading,
    loadingPrompt,
    searching,
    error,
    activeScope,
    activeProjectId,
    activeFolder,
    activeTag,
    query,
    semanticMode,
    filteredPrompts,
    visiblePrompts,
    allTags,
    loadPrompts,
    loadFolders,
    loadPrompt,
    createPrompt,
    updatePrompt,
    deletePrompt,
    runSemanticSearch,
    setScope,
    clearError
  }
})
