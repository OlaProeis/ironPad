import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project } from '../types'
import { projectsApi } from '../api/client'

const STORAGE_KEY = 'ironpad-active-project'

export const useWorkspaceStore = defineStore('workspace', () => {
  // State
  const activeProjectId = ref<string | null>(null)
  const activeProject = ref<Project | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const hasActiveProject = computed(() => activeProjectId.value !== null)

  // Actions
  async function setActiveProject(projectId: string | null) {
    if (projectId === activeProjectId.value) return

    activeProjectId.value = projectId
    
    if (projectId) {
      // Persist to localStorage
      localStorage.setItem(STORAGE_KEY, projectId)
      
      // Load project details
      try {
        loading.value = true
        error.value = null
        activeProject.value = await projectsApi.get(projectId)
      } catch (err) {
        error.value = `Failed to load project: ${err}`
        activeProject.value = null
      } finally {
        loading.value = false
      }
    } else {
      localStorage.removeItem(STORAGE_KEY)
      activeProject.value = null
    }
  }

  async function loadSavedProject() {
    const savedId = localStorage.getItem(STORAGE_KEY)
    if (savedId) {
      await setActiveProject(savedId)
    }
  }

  function clearActiveProject() {
    activeProjectId.value = null
    activeProject.value = null
    localStorage.removeItem(STORAGE_KEY)
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    activeProjectId,
    activeProject,
    loading,
    error,
    // Getters
    hasActiveProject,
    // Actions
    setActiveProject,
    loadSavedProject,
    clearActiveProject,
    clearError
  }
})
