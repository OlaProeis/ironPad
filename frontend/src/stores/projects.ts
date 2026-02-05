import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project } from '../types'
import { projectsApi } from '../api/client'

export const useProjectsStore = defineStore('projects', () => {
  // State
  const projects = ref<Project[]>([])
  const currentProject = ref<Project | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const sortedProjects = computed(() =>
    [...projects.value].sort((a, b) => a.name.localeCompare(b.name))
  )

  const getProjectById = computed(() => (id: string) =>
    projects.value.find(p => p.id === id)
  )

  // Actions
  async function loadProjects() {
    try {
      loading.value = true
      error.value = null
      projects.value = await projectsApi.list()
    } catch (err) {
      error.value = `Failed to load projects: ${err}`
    } finally {
      loading.value = false
    }
  }

  async function loadProject(id: string) {
    try {
      loading.value = true
      error.value = null
      currentProject.value = await projectsApi.get(id)
    } catch (err) {
      error.value = `Failed to load project: ${err}`
      currentProject.value = null
    } finally {
      loading.value = false
    }
  }

  async function createProject(name: string) {
    try {
      error.value = null
      const newProject = await projectsApi.create(name)
      await loadProjects()
      return newProject
    } catch (err) {
      error.value = `Failed to create project: ${err}`
      throw err
    }
  }

  function clearCurrentProject() {
    currentProject.value = null
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    projects,
    currentProject,
    loading,
    error,
    // Getters
    sortedProjects,
    getProjectById,
    // Actions
    loadProjects,
    loadProject,
    createProject,
    clearCurrentProject,
    clearError
  }
})
