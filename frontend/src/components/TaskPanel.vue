<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useTasksStore } from '../stores'

const router = useRouter()
const tasksStore = useTasksStore()

const taskFilter = ref<'all' | 'pending' | 'completed'>('pending')

const filteredTasks = computed(() => {
  const tasks = tasksStore.allTasks
  if (taskFilter.value === 'all') return tasks
  if (taskFilter.value === 'pending') return tasks.filter(t => !t.completed)
  return tasks.filter(t => t.completed)
})

function goToTask(task: { id: string; project_id: string }) {
  router.push({ 
    name: 'project-tasks', 
    params: { id: task.project_id, taskId: task.id } 
  })
}

async function toggleTask(task: { id: string; project_id: string }) {
  try {
    await tasksStore.toggleTask(task.project_id, task.id)
    // Refresh global task list
    await tasksStore.loadAllTasks()
  } catch {
    // Error handled in store
  }
}

onMounted(() => {
  tasksStore.loadAllTasks()
})
</script>

<template>
  <div class="tasks-panel">
    <div class="task-filters">
      <button :class="{ active: taskFilter === 'pending' }" @click="taskFilter = 'pending'">
        Pending
      </button>
      <button :class="{ active: taskFilter === 'all' }" @click="taskFilter = 'all'">
        All
      </button>
      <button :class="{ active: taskFilter === 'completed' }" @click="taskFilter = 'completed'">
        Done
      </button>
    </div>
    <div v-if="tasksStore.loading" class="loading">Loading tasks...</div>
    <div v-else-if="filteredTasks.length === 0" class="no-tasks">No tasks</div>
    <div v-else class="task-list">
      <div 
        v-for="task in filteredTasks" 
        :key="task.id" 
        class="task-item" 
        :class="{ completed: task.completed }"
        @click="goToTask(task)"
      >
        <button class="task-checkbox" @click.stop="toggleTask(task)">
          {{ task.completed ? '☑' : '☐' }}
        </button>
        <span class="task-text">{{ task.title }}</span>
        <span class="task-source">
          {{ task.project_id }}
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tasks-panel {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.task-filters {
  padding: 8px 12px;
  display: flex;
  gap: 4px;
  border-bottom: 1px solid var(--color-border);
}

.task-filters button {
  padding: 4px 8px;
  font-size: 12px;
}

.task-filters button.active {
  background: var(--color-primary);
  color: white;
}

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.task-item {
  padding: 8px;
  border-radius: 4px;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
}

.task-item:hover {
  background: var(--color-border);
}

.task-item.completed {
  opacity: 0.6;
}

.task-item.completed .task-text {
  text-decoration: line-through;
}

.task-checkbox {
  flex-shrink: 0;
  padding: 0;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
}

.task-text {
  flex: 1;
}

.task-source {
  font-size: 11px;
  color: var(--color-text-secondary);
}

.task-source:hover {
  color: var(--color-primary);
}

.no-tasks,
.loading {
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary);
}
</style>
