<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectsStore, useWorkspaceStore, useUiStore, useThemeStore } from '../stores'

const router = useRouter()
const projectsStore = useProjectsStore()
const workspaceStore = useWorkspaceStore()
const uiStore = useUiStore()
const themeStore = useThemeStore()

const isDark = computed(() => themeStore.getEffectiveTheme() === 'dark')

const dropdownOpen = ref(false)
const showNewProjectInput = ref(false)
const newProjectName = ref('')

const activeProject = computed(() => workspaceStore.activeProject)
const projects = computed(() => projectsStore.sortedProjects)

function toggleDropdown() {
  dropdownOpen.value = !dropdownOpen.value
  showNewProjectInput.value = false
}

function closeDropdown() {
  dropdownOpen.value = false
  showNewProjectInput.value = false
  newProjectName.value = ''
}

async function selectProject(projectId: string) {
  await workspaceStore.setActiveProject(projectId)
  closeDropdown()
  router.push({ name: 'project', params: { id: projectId } })
}

function showCreateProject() {
  showNewProjectInput.value = true
}

async function createProject() {
  if (!newProjectName.value.trim()) return
  
  try {
    const project = await projectsStore.createProject(newProjectName.value.trim())
    await workspaceStore.setActiveProject(project.id)
    closeDropdown()
    router.push({ name: 'project', params: { id: project.id } })
  } catch {
    // Error handled in store
  }
}

function goToProjects() {
  closeDropdown()
  router.push({ name: 'projects' })
}

function goHome() {
  router.push({ name: 'home' })
}
</script>

<template>
  <header class="topbar">
    <div class="topbar-left">
      <h1 class="app-title" @click="goHome" style="cursor: pointer" title="Dashboard">Ironpad</h1>
      
      <div class="project-selector">
        <button class="project-button" @click="toggleDropdown">
          <span class="project-name">
            {{ activeProject?.name ?? 'Select Project' }}
          </span>
          <span class="dropdown-arrow">{{ dropdownOpen ? '▲' : '▼' }}</span>
        </button>
        
        <div v-if="dropdownOpen" class="dropdown-menu" @click.stop>
          <div v-if="!showNewProjectInput" class="dropdown-content">
            <div 
              v-for="project in projects" 
              :key="project.id"
              :class="['dropdown-item', { active: project.id === activeProject?.id }]"
              @click="selectProject(project.id)"
            >
              {{ project.name }}
            </div>
            
            <div v-if="projects.length === 0" class="dropdown-empty">
              No projects yet
            </div>
            
            <div class="dropdown-divider"></div>
            
            <div class="dropdown-item action" @click="showCreateProject">
              + New Project
            </div>
            
            <div class="dropdown-item action" @click="goToProjects">
              Manage Projects
            </div>
          </div>
          
          <div v-else class="new-project-form">
            <input
              v-model="newProjectName"
              type="text"
              placeholder="Project name..."
              class="new-project-input"
              @keyup.enter="createProject"
              @keyup.escape="closeDropdown"
              autofocus
            />
            <div class="form-buttons">
              <button class="primary" @click="createProject" :disabled="!newProjectName.trim()">
                Create
              </button>
              <button @click="closeDropdown">Cancel</button>
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <div class="topbar-right">
      <button 
        @click="uiStore.toggleSearch()" 
        :class="{ active: uiStore.showSearch }" 
        title="Search (Ctrl+K)"
      >
        Search
      </button>
      <button 
        @click="uiStore.toggleTasks()" 
        :class="{ active: uiStore.showTasks }" 
        title="Tasks"
      >
        Tasks
      </button>
      <button 
        class="theme-toggle"
        @click="themeStore.toggleTheme()" 
        :title="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
      >
        {{ isDark ? '☀️' : '🌙' }}
      </button>
    </div>
  </header>
  
  <!-- Click outside to close dropdown -->
  <div v-if="dropdownOpen" class="dropdown-overlay" @click="closeDropdown"></div>
</template>

<style scoped>
.topbar {
  height: var(--header-height);
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  position: relative;
  z-index: 100;
}

.topbar-left {
  display: flex;
  align-items: center;
  gap: 28px;
}

.app-title {
  font-size: 17px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text);
}

.project-selector {
  position: relative;
}

.project-button {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 14px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  cursor: pointer;
  min-width: 180px;
  max-width: 280px;
  justify-content: space-between;
}

.project-button:hover {
  border-color: var(--color-primary);
}

.project-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dropdown-arrow {
  font-size: 10px;
  color: var(--color-text-secondary);
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 4px;
  min-width: 220px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 200;
}

.dropdown-content {
  padding: 8px 0;
}

.dropdown-item {
  padding: 10px 16px;
  cursor: pointer;
  transition: background 0.15s;
}

.dropdown-item:hover {
  background: var(--color-bg-secondary);
}

.dropdown-item.active {
  background: var(--color-primary);
  color: white;
}

.dropdown-item.action {
  color: var(--color-primary);
  font-weight: 500;
}

.dropdown-divider {
  height: 1px;
  background: var(--color-border);
  margin: 8px 0;
}

.dropdown-empty {
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary);
  font-style: italic;
}

.new-project-form {
  padding: 12px;
}

.new-project-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 14px;
  margin-bottom: 12px;
}

.new-project-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.form-buttons {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.topbar-right {
  display: flex;
  gap: 10px;
}

.topbar-right button {
  padding: 8px 16px;
  font-size: 13px;
}

.topbar-right button.active {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.theme-toggle {
  font-size: 16px;
  padding: 8px 12px !important;
}

.dropdown-overlay {
  position: fixed;
  inset: 0;
  z-index: 99;
}
</style>
