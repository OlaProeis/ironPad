<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useProjectsStore } from '../stores'

const emit = defineEmits<{
  create: []
}>()

const router = useRouter()
const route = useRoute()
const projectsStore = useProjectsStore()

const selectedProjectId = computed(() => route.params.id as string | undefined)

function selectProject(id: string) {
  router.push({ name: 'project', params: { id } })
}

function goToTasks(id: string, event: Event) {
  event.stopPropagation()
  router.push({ name: 'project-tasks', params: { id } })
}
</script>

<template>
  <div class="project-list-container">
    <div class="project-actions">
      <button class="create-btn" @click="emit('create')">+ New Project</button>
    </div>
    
    <div v-if="projectsStore.loading" class="loading">Loading projects...</div>
    <div v-else-if="projectsStore.sortedProjects.length === 0" class="empty">No projects yet</div>
    <ul v-else class="project-list">
      <li
        v-for="project in projectsStore.sortedProjects"
        :key="project.id"
        :class="['project-item', { active: project.id === selectedProjectId }]"
        @click="selectProject(project.id)"
      >
        <div class="project-item-content">
          <div class="project-item-name">{{ project.name }}</div>
          <button 
            class="tasks-btn" 
            @click="goToTasks(project.id, $event)"
            title="View Tasks"
          >
            ☑
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>

<style scoped>
.project-list-container {
  display: flex;
  flex-direction: column;
}

.project-actions {
  padding: 8px 16px;
}

.create-btn {
  width: 100%;
  padding: 8px;
  font-size: 13px;
}

.project-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.project-item {
  padding: 10px 16px;
  cursor: pointer;
  border-left: 3px solid transparent;
  transition: background 0.15s, border-color 0.15s;
}

.project-item:hover {
  background: var(--color-border);
}

.project-item.active {
  background: var(--color-border);
  border-left-color: var(--color-primary);
}

.project-item-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.project-item-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tasks-btn {
  padding: 4px 8px;
  font-size: 12px;
  opacity: 0;
  transition: opacity 0.15s;
}

.project-item:hover .tasks-btn {
  opacity: 1;
}

.loading,
.empty {
  padding: 16px;
  color: var(--color-text-secondary);
  text-align: center;
}
</style>
