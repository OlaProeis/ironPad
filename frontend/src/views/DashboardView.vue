<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectsStore, useTasksStore, useWorkspaceStore } from '../stores'
import type { Task } from '../types'

const router = useRouter()
const projectsStore = useProjectsStore()
const tasksStore = useTasksStore()
const workspaceStore = useWorkspaceStore()

const loading = ref(true)

// Group tasks by project
const projectSummaries = computed(() => {
  return projectsStore.sortedProjects.map(project => {
    const projectTasks = tasksStore.allTasks.filter(t => t.project_id === project.id)
    const active = projectTasks.filter(t => !t.completed && t.is_active)
    const backlog = projectTasks.filter(t => !t.completed && !t.is_active)
    const completed = projectTasks.filter(t => t.completed)
    const overdue = active.filter(t => {
      if (!t.due_date) return false
      return new Date(t.due_date) < new Date()
    })

    return {
      ...project,
      activeTasks: active,
      backlogCount: backlog.length,
      completedCount: completed.length,
      overdueCount: overdue.length,
      totalCount: projectTasks.length,
    }
  })
})

function formatDueDate(dateStr?: string) {
  if (!dateStr) return null
  try {
    const date = new Date(dateStr)
    const now = new Date()
    const diffDays = Math.ceil((date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))

    if (diffDays < 0) return { text: 'Overdue', class: 'overdue' }
    if (diffDays === 0) return { text: 'Today', class: 'today' }
    if (diffDays === 1) return { text: 'Tomorrow', class: 'soon' }
    if (diffDays <= 7) return { text: `${diffDays}d`, class: 'soon' }
    return { text: date.toLocaleDateString(), class: '' }
  } catch {
    return null
  }
}

function goToProject(projectId: string) {
  workspaceStore.setActiveProject(projectId)
  router.push({ name: 'project', params: { id: projectId } })
}

function goToProjectTasks(projectId: string) {
  workspaceStore.setActiveProject(projectId)
  router.push({ name: 'project-tasks', params: { id: projectId } })
}

function goToTask(projectId: string, task: Task) {
  workspaceStore.setActiveProject(projectId)
  router.push({ name: 'project-tasks', params: { id: projectId, taskId: task.id } })
}

async function createProject() {
  const name = prompt('Project name:')
  if (!name) return

  try {
    const project = await projectsStore.createProject(name)
    await workspaceStore.setActiveProject(project.id)
    router.push({ name: 'project', params: { id: project.id } })
  } catch {
    // Error handled in store
  }
}

onMounted(async () => {
  try {
    await Promise.all([
      projectsStore.loadProjects(),
      tasksStore.loadAllTasks()
    ])
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="dashboard">
    <div class="dashboard-header">
      <div class="header-left">
        <h2>Dashboard</h2>
        <span class="project-count">{{ projectSummaries.length }} projects</span>
      </div>
      <button class="primary" @click="createProject">+ New Project</button>
    </div>

    <div v-if="loading" class="loading">Loading...</div>

    <div v-else-if="projectSummaries.length === 0" class="empty-state">
      <h3>Welcome to Ironpad</h3>
      <p>Create your first project to get started.</p>
      <button class="primary" @click="createProject" style="margin-top: 16px">Create Project</button>
    </div>

    <div v-else class="dashboard-grid">
      <div
        v-for="project in projectSummaries"
        :key="project.id"
        class="project-card"
      >
        <!-- Card Header -->
        <div class="card-header" @click="goToProject(project.id)">
          <h3 class="card-title">{{ project.name }}</h3>
          <div class="card-stats">
            <span class="stat active-stat" :title="`${project.activeTasks.length} active`">
              {{ project.activeTasks.length }} active
            </span>
            <span v-if="project.backlogCount > 0" class="stat backlog-stat" :title="`${project.backlogCount} backlog`">
              {{ project.backlogCount }} backlog
            </span>
            <span v-if="project.overdueCount > 0" class="stat overdue-stat" :title="`${project.overdueCount} overdue`">
              {{ project.overdueCount }} overdue
            </span>
          </div>
        </div>

        <!-- Active Tasks List -->
        <div class="card-tasks" v-if="project.activeTasks.length > 0">
          <div
            v-for="task in project.activeTasks.slice(0, 5)"
            :key="task.id"
            class="card-task-item"
            @click="goToTask(project.id, task)"
          >
            <span class="task-checkbox">&#9744;</span>
            <span class="task-title">{{ task.title }}</span>
            <div class="task-meta">
              <span
                v-for="tag in task.tags?.slice(0, 2)"
                :key="tag"
                class="task-tag"
              >{{ tag }}</span>
              <span
                v-if="task.due_date && formatDueDate(task.due_date)"
                :class="['task-due', formatDueDate(task.due_date)?.class]"
              >{{ formatDueDate(task.due_date)?.text }}</span>
            </div>
          </div>
          <div
            v-if="project.activeTasks.length > 5"
            class="card-task-more"
            @click="goToProjectTasks(project.id)"
          >
            +{{ project.activeTasks.length - 5 }} more tasks...
          </div>
        </div>
        <div v-else class="card-empty">
          No active tasks
        </div>

        <!-- Card Footer -->
        <div class="card-footer">
          <span class="completed-count" v-if="project.completedCount > 0">
            {{ project.completedCount }} completed
          </span>
          <button class="link-btn" @click="goToProjectTasks(project.id)">View All Tasks</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dashboard-header {
  height: var(--header-height);
  min-height: var(--header-height);
  padding: 0 24px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: baseline;
  gap: 12px;
}

.dashboard-header h2 {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
}

.project-count {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.dashboard-grid {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 20px;
  align-content: start;
}

/* Project Card */
.project-card {
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  border-radius: 10px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.project-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.card-header {
  padding: 16px 20px 12px;
  cursor: pointer;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 8px 0;
}

.card-stats {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.stat {
  font-size: 12px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 10px;
}

.active-stat {
  background: rgba(88, 166, 255, 0.15);
  color: var(--color-primary);
}

.backlog-stat {
  background: rgba(153, 153, 153, 0.15);
  color: var(--color-text-secondary);
}

.overdue-stat {
  background: rgba(248, 81, 73, 0.15);
  color: var(--color-danger);
}

/* Tasks in card */
.card-tasks {
  padding: 0 12px;
  flex: 1;
}

.card-task-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
}

.card-task-item:hover {
  background: var(--color-bg-hover);
}

.card-task-item .task-checkbox {
  flex-shrink: 0;
  color: var(--color-text-secondary);
  font-size: 14px;
}

.card-task-item .task-title {
  flex: 1;
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.task-meta {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-shrink: 0;
}

.task-tag {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 8px;
  background: var(--color-border);
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.task-due {
  font-size: 11px;
  white-space: nowrap;
  color: var(--color-text-secondary);
}

.task-due.overdue {
  color: var(--color-danger);
  font-weight: 500;
}

.task-due.today {
  color: var(--color-danger);
}

.task-due.soon {
  color: var(--color-primary);
}

.card-task-more {
  padding: 8px 8px;
  font-size: 12px;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.card-task-more:hover {
  color: var(--color-primary);
}

.card-empty {
  padding: 16px 20px;
  font-size: 13px;
  color: var(--color-text-secondary);
  font-style: italic;
  flex: 1;
}

/* Card Footer */
.card-footer {
  padding: 10px 20px;
  border-top: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.completed-count {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.link-btn {
  padding: 4px 8px;
  border: none;
  background: transparent;
  color: var(--color-primary);
  font-size: 12px;
  cursor: pointer;
  font-weight: 500;
}

.link-btn:hover {
  text-decoration: underline;
}

/* States */
.loading,
.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  color: var(--color-text-secondary);
  padding: 32px;
}

.empty-state h3 {
  margin-bottom: 8px;
  color: var(--color-text);
  font-size: 20px;
}
</style>
