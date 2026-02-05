<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useProjectsStore, useGitStore, useWorkspaceStore } from '../stores'
import { projectsApi } from '../api/client'
import type { ProjectWithContent } from '../types'
import MilkdownEditor from '../components/MilkdownEditor.vue'

const props = defineProps<{
  id: string
}>()

const route = useRoute()
const router = useRouter()
const projectsStore = useProjectsStore()
const gitStore = useGitStore()
const workspaceStore = useWorkspaceStore()

const projectId = computed(() => props.id || (route.params.id as string))
const project = computed(() => projectsStore.getProjectById(projectId.value))
const projectContent = ref<ProjectWithContent | null>(null)
const editorContent = ref('')
const loading = ref(false)
const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
let saveTimeout: number | null = null

function goToTasks() {
  router.push({ name: 'project-tasks', params: { id: projectId.value } })
}

function scheduleAutoSave() {
  if (saveTimeout) clearTimeout(saveTimeout)
  saveStatus.value = 'idle'
  saveTimeout = window.setTimeout(saveNote, 1000)
}

async function saveNote() {
  if (!projectContent.value) return
  
  try {
    saveStatus.value = 'saving'
    projectContent.value = await projectsApi.updateContent(projectId.value, editorContent.value)
    saveStatus.value = 'saved'
    gitStore.loadStatus()
    setTimeout(() => {
      if (saveStatus.value === 'saved') saveStatus.value = 'idle'
    }, 2000)
  } catch (err) {
    saveStatus.value = 'error'
  }
}

async function loadProjectContent() {
  loading.value = true
  try {
    projectContent.value = await projectsApi.getContent(projectId.value)
    editorContent.value = projectContent.value?.content ?? ''
  } catch {
    projectContent.value = null
  } finally {
    loading.value = false
  }
}

watch(editorContent, (newContent, oldContent) => {
  if (projectContent.value && oldContent !== undefined && newContent !== oldContent) {
    scheduleAutoSave()
  }
})

watch(projectId, () => {
  loadProjectContent()
}, { immediate: true })

onMounted(async () => {
  await projectsStore.loadProject(projectId.value)
  
  // Set as active project
  if (workspaceStore.activeProjectId !== projectId.value) {
    await workspaceStore.setActiveProject(projectId.value)
  }
})
</script>

<template>
  <div class="project-view">
    <div class="view-header">
      <h2>{{ project?.name ?? projectId }}</h2>
      <div class="button-group">
        <span :class="['status', saveStatus]">
          <template v-if="saveStatus === 'saving'">Saving...</template>
          <template v-else-if="saveStatus === 'saved'">Saved</template>
          <template v-else-if="saveStatus === 'error'">Save failed</template>
        </span>
        <button class="primary" @click="goToTasks">View Tasks</button>
        <button @click="saveNote" :disabled="saveStatus === 'saving'">Save</button>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading project...</div>
    
    <div v-else-if="projectContent" class="view-content">
      <div class="editor-container">
        <MilkdownEditor
          v-model="editorContent"
          :project-id="projectId"
          placeholder="Write about this project..."
        />
      </div>
    </div>

    <div v-else class="empty-state">
      <h3>Project not found</h3>
      <p>This project doesn't exist or couldn't be loaded.</p>
    </div>
  </div>
</template>

<style scoped>
.project-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.view-header {
  height: var(--header-height);
  min-height: var(--header-height);
  max-height: var(--header-height);
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.view-header h2 {
  font-size: 14px;
  font-weight: 500;
  margin: 0;
}

.button-group {
  display: flex;
  gap: 8px;
  align-items: center;
}

.status {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.status.saving { color: var(--color-primary); }
.status.saved { color: var(--color-success); }
.status.error { color: var(--color-danger); }

.view-content {
  flex: 1;
  overflow: hidden;
  display: flex;
}

.editor-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
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
