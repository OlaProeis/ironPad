<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectsStore } from '../stores'

const router = useRouter()
const projectsStore = useProjectsStore()

async function createProject() {
  const name = prompt('Project name:')
  if (!name) return
  
  try {
    const project = await projectsStore.createProject(name)
    router.push({ name: 'project', params: { id: project.id } })
  } catch {
    // Error handled in store
  }
}

function selectProject(id: string) {
  router.push({ name: 'project', params: { id } })
}

onMounted(() => {
  projectsStore.loadProjects()
})
</script>

<template>
  <div class="projects-view">
    <div class="view-header">
      <h2>Projects</h2>
      <button class="primary" @click="createProject">+ New Project</button>
    </div>

    <div v-if="projectsStore.loading" class="loading">Loading projects...</div>
    
    <div v-else-if="projectsStore.projects.length === 0" class="empty-state">
      <h3>No projects yet</h3>
      <p>Create your first project to get started.</p>
      <button class="primary" @click="createProject" style="margin-top: 16px">Create Project</button>
    </div>

    <div v-else class="projects-grid">
      <div 
        v-for="project in projectsStore.sortedProjects" 
        :key="project.id"
        class="project-card"
        @click="selectProject(project.id)"
      >
        <h3>{{ project.name }}</h3>
        <p class="project-path">{{ project.path }}</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.projects-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.view-header {
  height: var(--header-height);
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.view-header h2 {
  font-size: 14px;
  font-weight: 500;
  margin: 0;
}

.projects-grid {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 16px;
  align-content: start;
}

.project-card {
  padding: 20px;
  border-radius: 8px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.project-card:hover {
  border-color: var(--color-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.project-card h3 {
  margin: 0 0 8px 0;
  font-size: 16px;
  font-weight: 600;
}

.project-path {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
}

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
}
</style>
