<script setup lang="ts">
import { computed, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useWorkspaceStore, useTasksStore, useUiStore } from '../stores'
import SearchPanel from './SearchPanel.vue'
import TaskPanel from './TaskPanel.vue'
import GitStatus from './GitStatus.vue'

const router = useRouter()
const route = useRoute()
const workspaceStore = useWorkspaceStore()
const tasksStore = useTasksStore()
const uiStore = useUiStore()

const activeProject = computed(() => workspaceStore.activeProject)
const activeProjectId = computed(() => workspaceStore.activeProjectId)

// Load tasks when project changes
watch(activeProjectId, async (id) => {
  if (id) {
    await tasksStore.loadProjectTasks(id)
  }
}, { immediate: true })

const activeTasks = computed(() => tasksStore.activeTasks)
const completedTasks = computed(() => tasksStore.completedTasks)

function goToProjectOverview() {
  if (activeProjectId.value) {
    router.push({ name: 'project', params: { id: activeProjectId.value } })
  }
}

function goToProjectTasks() {
  if (activeProjectId.value) {
    router.push({ name: 'project-tasks', params: { id: activeProjectId.value } })
  }
}

function goToProjectNotes() {
  if (activeProjectId.value) {
    router.push({ name: 'project-notes', params: { id: activeProjectId.value } })
  }
}

function goToDaily() {
  router.push({ name: 'daily' })
}

function goToCalendar() {
  router.push({ name: 'calendar' })
}

function goToProjects() {
  router.push({ name: 'projects' })
}
</script>

<template>
  <div class="sidebar">
    <!-- Search Panel (overlay) -->
    <SearchPanel v-if="uiStore.showSearch" />

    <!-- Tasks Panel (overlay) -->
    <TaskPanel v-else-if="uiStore.showTasks" />

    <!-- Main Content -->
    <template v-else>
      <!-- No Project Selected -->
      <div v-if="!activeProject" class="no-project">
        <div class="no-project-content">
          <h3>No Project Selected</h3>
          <p>Select a project from the dropdown above to get started.</p>
          <button class="primary" @click="goToProjects">Browse Projects</button>
        </div>
      </div>

      <!-- Project Content -->
      <template v-else>
        <!-- Project Navigation -->
        <nav class="project-nav">
          <button 
            :class="['nav-item', { active: route.name === 'project' }]"
            @click="goToProjectOverview"
          >
            Overview
          </button>
          <button 
            :class="['nav-item', { active: route.name === 'project-notes' }]"
            @click="goToProjectNotes"
          >
            Notes
          </button>
          <button 
            :class="['nav-item', { active: route.name === 'project-tasks' }]"
            @click="goToProjectTasks"
          >
            Tasks
          </button>
          <button 
            :class="['nav-item', { active: route.name === 'daily' || route.name === 'daily-note' }]"
            @click="goToDaily"
          >
            Daily
          </button>
          <button 
            :class="['nav-item', { active: route.name === 'calendar' }]"
            @click="goToCalendar"
          >
            Calendar
          </button>
        </nav>

        <!-- Quick Stats -->
        <div class="quick-stats">
          <div class="stat-item" @click="goToProjectTasks">
            <span class="stat-value">{{ activeTasks.length }}</span>
            <span class="stat-label">Active Tasks</span>
          </div>
          <div class="stat-item" @click="goToProjectTasks">
            <span class="stat-value">{{ completedTasks.length }}</span>
            <span class="stat-label">Completed</span>
          </div>
        </div>

        <!-- Recent Tasks Preview -->
        <div class="sidebar-section">
          <div class="section-header">
            <h4>Active Tasks</h4>
            <button class="link-btn" @click="goToProjectTasks">View All</button>
          </div>
          <div v-if="activeTasks.length === 0" class="empty-section">
            No active tasks
          </div>
          <ul v-else class="task-preview-list">
            <li v-for="task in activeTasks.slice(0, 5)" :key="task.id" class="task-preview-item">
              <span class="task-checkbox">☐</span>
              <span class="task-text">{{ task.title }}</span>
            </li>
            <li v-if="activeTasks.length > 5" class="task-more">
              +{{ activeTasks.length - 5 }} more...
            </li>
          </ul>
        </div>
      </template>
    </template>

    <!-- Git Status -->
    <GitStatus />
  </div>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  max-width: var(--sidebar-width);
  flex-shrink: 0;
  background: var(--color-bg-secondary);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.no-project {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
}

.no-project-content {
  text-align: center;
}

.no-project-content h3 {
  margin-bottom: 8px;
  color: var(--color-text);
}

.no-project-content p {
  margin-bottom: 16px;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.project-nav {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 12px;
  border-bottom: 1px solid var(--color-border);
}

.nav-item {
  flex: 1;
  min-width: 70px;
  padding: 8px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.nav-item:hover {
  background: var(--color-border);
  color: var(--color-text);
}

.nav-item.active {
  background: var(--color-primary);
  color: white;
}

.quick-stats {
  display: flex;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--color-border);
}

.stat-item {
  flex: 1;
  text-align: center;
  padding: 12px;
  background: var(--color-bg);
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
}

.stat-item:hover {
  background: var(--color-border);
}

.stat-value {
  display: block;
  font-size: 24px;
  font-weight: 600;
  color: var(--color-text);
}

.stat-label {
  display: block;
  font-size: 11px;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-top: 4px;
}

.sidebar-section {
  padding: 16px;
  flex: 1;
  overflow-y: auto;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-header h4 {
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin: 0;
}

.link-btn {
  padding: 4px 8px;
  border: none;
  background: transparent;
  color: var(--color-primary);
  font-size: 12px;
  cursor: pointer;
}

.link-btn:hover {
  text-decoration: underline;
}

.empty-section {
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary);
  font-style: italic;
  font-size: 13px;
}

.task-preview-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.task-preview-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border);
}

.task-preview-item:last-child {
  border-bottom: none;
}

.task-checkbox {
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.task-text {
  font-size: 13px;
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.task-more {
  padding: 8px 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  font-style: italic;
}
</style>
