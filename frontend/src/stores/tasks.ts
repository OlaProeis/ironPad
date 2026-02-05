import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Task, TaskWithContent } from '../types'
import { tasksApi } from '../api/client'

export const useTasksStore = defineStore('tasks', () => {
  // State
  const tasks = ref<Task[]>([])
  const allTasks = ref<Task[]>([])
  const currentProjectId = ref<string | null>(null)
  const selectedTask = ref<TaskWithContent | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const activeTasks = computed(() => 
    tasks.value.filter(t => !t.completed && t.section !== 'Backlog')
  )

  const completedTasks = computed(() =>
    tasks.value.filter(t => t.completed || t.section === 'Completed')
  )

  const backlogTasks = computed(() =>
    tasks.value.filter(t => !t.completed && t.section === 'Backlog')
  )

  const pendingTasks = computed(() =>
    allTasks.value.filter(t => !t.completed)
  )

  const tasksByProject = computed(() => (projectId: string) =>
    allTasks.value.filter(t => t.project_id === projectId)
  )

  /** All unique tags used across tasks in the current project */
  const projectTags = computed(() => {
    const tagSet = new Set<string>()
    for (const task of tasks.value) {
      if (task.tags) {
        for (const tag of task.tags) {
          tagSet.add(tag)
        }
      }
    }
    return [...tagSet].sort()
  })

  // Actions
  async function loadAllTasks() {
    try {
      loading.value = true
      error.value = null
      allTasks.value = await tasksApi.listAll()
    } catch (err) {
      error.value = `Failed to load tasks: ${err}`
    } finally {
      loading.value = false
    }
  }

  async function loadProjectTasks(projectId: string) {
    try {
      loading.value = true
      error.value = null
      currentProjectId.value = projectId
      tasks.value = await tasksApi.list(projectId)
    } catch (err) {
      error.value = `Failed to load project tasks: ${err}`
    } finally {
      loading.value = false
    }
  }

  async function loadTask(projectId: string, taskId: string) {
    try {
      error.value = null
      selectedTask.value = await tasksApi.get(projectId, taskId)
    } catch (err) {
      error.value = `Failed to load task: ${err}`
      selectedTask.value = null
    }
  }

  async function createTask(projectId: string, title: string, section?: string, parentId?: string) {
    try {
      error.value = null
      const task = await tasksApi.create(projectId, title, section, parentId)
      
      // Refresh tasks list
      if (currentProjectId.value === projectId) {
        await loadProjectTasks(projectId)
      }
      
      return task
    } catch (err) {
      error.value = `Failed to create task: ${err}`
      throw err
    }
  }

  async function updateTaskContent(projectId: string, taskId: string, content: string) {
    try {
      error.value = null
      const task = await tasksApi.updateContent(projectId, taskId, content)
      selectedTask.value = task
      
      // Refresh tasks list to update timestamps
      if (currentProjectId.value === projectId) {
        await loadProjectTasks(projectId)
      }
      
      return task
    } catch (err) {
      error.value = `Failed to update task: ${err}`
      throw err
    }
  }

  async function toggleTask(projectId: string, taskId: string) {
    try {
      error.value = null
      await tasksApi.toggle(projectId, taskId)
      
      // Refresh tasks
      if (currentProjectId.value === projectId) {
        await loadProjectTasks(projectId)
      }
      
      // Update selected task if it's the one being toggled
      if (selectedTask.value?.id === taskId) {
        await loadTask(projectId, taskId)
      }
    } catch (err) {
      error.value = `Failed to toggle task: ${err}`
      throw err
    }
  }

  async function updateTaskMeta(
    projectId: string, 
    taskId: string, 
    meta: { title?: string; section?: string; priority?: string; due_date?: string; is_active?: boolean; tags?: string[]; recurrence?: string; recurrence_interval?: number }
  ) {
    try {
      error.value = null
      await tasksApi.updateMeta(projectId, taskId, meta)
      
      // Refresh tasks
      if (currentProjectId.value === projectId) {
        await loadProjectTasks(projectId)
      }
      
      // Update selected task if it's the one being updated
      if (selectedTask.value?.id === taskId) {
        await loadTask(projectId, taskId)
      }
    } catch (err) {
      error.value = `Failed to update task: ${err}`
      throw err
    }
  }

  async function deleteTask(projectId: string, taskId: string) {
    try {
      error.value = null
      await tasksApi.delete(projectId, taskId)
      
      // Clear selected task if it was deleted
      if (selectedTask.value?.id === taskId) {
        selectedTask.value = null
      }
      
      // Refresh tasks
      if (currentProjectId.value === projectId) {
        await loadProjectTasks(projectId)
      }
    } catch (err) {
      error.value = `Failed to delete task: ${err}`
      throw err
    }
  }

  function selectTask(task: Task | null) {
    if (task && currentProjectId.value) {
      loadTask(currentProjectId.value, task.id)
    } else {
      selectedTask.value = null
    }
  }

  function clearSelectedTask() {
    selectedTask.value = null
  }

  function clearProjectTasks() {
    tasks.value = []
    currentProjectId.value = null
    selectedTask.value = null
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    tasks,
    allTasks,
    currentProjectId,
    selectedTask,
    loading,
    error,
    // Getters
    activeTasks,
    completedTasks,
    backlogTasks,
    pendingTasks,
    tasksByProject,
    projectTags,
    // Actions
    loadAllTasks,
    loadProjectTasks,
    loadTask,
    createTask,
    updateTaskContent,
    toggleTask,
    updateTaskMeta,
    deleteTask,
    selectTask,
    clearSelectedTask,
    clearProjectTasks,
    clearError
  }
})
