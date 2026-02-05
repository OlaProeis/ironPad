<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTasksStore, useProjectsStore, useWorkspaceStore, useGitStore } from '../stores'
import { useWebSocket } from '../composables/useWebSocket'
import MilkdownEditor from '../components/MilkdownEditor.vue'
import type { Task } from '../types'

const props = defineProps<{
  id: string
  taskId?: string
}>()

const route = useRoute()
const router = useRouter()
const tasksStore = useTasksStore()
const projectsStore = useProjectsStore()
const workspaceStore = useWorkspaceStore()
const gitStore = useGitStore()

const projectId = computed(() => props.id || (route.params.id as string))
const currentTaskId = computed(() => props.taskId || (route.params.taskId as string | undefined))

// Editor state
const editorContent = ref('')
const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
let saveTimeout: number | null = null

// CRITICAL: Separate key for editor recreation - only update AFTER content is ready
const editorKey = ref<string | null>(null)

// Track the last saved/loaded content to detect actual user changes
// This prevents unnecessary saves when just opening a task
const lastSavedContent = ref<string | null>(null)

// Track which task ID the pending save is for
let pendingSaveTaskId: string | null = null

// New task form
const newTaskTitle = ref('')
const showNewTaskForm = ref(false)

// Editing title inline
const editingTaskId = ref<string | null>(null)
const editingTitle = ref('')

// Tag filter
const activeTagFilter = ref<string | null>(null)

// Tag editing
const newTagInput = ref('')
const showTagInput = ref(false)

// Subtask helpers
function getSubtasks(parentId: string): Task[] {
  return tasksStore.tasks.filter(t => t.parent_id === parentId)
}

function isTopLevel(task: Task): boolean {
  return !task.parent_id
}

// Filtered task lists (respects tag filter, only top-level tasks)
const filteredActiveTasks = computed(() => {
  let tasks = tasksStore.activeTasks.filter(isTopLevel)
  if (activeTagFilter.value) tasks = tasks.filter(t => t.tags?.includes(activeTagFilter.value!))
  return tasks
})

const filteredBacklogTasks = computed(() => {
  let tasks = tasksStore.backlogTasks.filter(isTopLevel)
  if (activeTagFilter.value) tasks = tasks.filter(t => t.tags?.includes(activeTagFilter.value!))
  return tasks
})

const filteredCompletedTasks = computed(() => {
  let tasks = tasksStore.completedTasks.filter(isTopLevel)
  if (activeTagFilter.value) tasks = tasks.filter(t => t.tags?.includes(activeTagFilter.value!))
  return tasks
})

// Subtask creation
const showSubtaskInput = ref(false)
const newSubtaskTitle = ref('')

// Tag autocomplete suggestions
const tagSuggestions = computed(() => {
  if (!newTagInput.value.trim()) return []
  const input = newTagInput.value.trim().toLowerCase()
  const currentTags = tasksStore.selectedTask?.tags || []
  return tasksStore.projectTags
    .filter(tag => tag.toLowerCase().includes(input) && !currentTags.includes(tag))
    .slice(0, 5)
})

// WebSocket for file locking
const { lockFile, unlockFile } = useWebSocket({
  onFileModified: (path) => {
    if (path.includes('/tasks/')) {
      tasksStore.loadProjectTasks(projectId.value)
    }
  }
})

// ============ Task Selection ============

function selectTask(task: Task) {
  router.push({ 
    name: 'project-tasks', 
    params: { id: projectId.value, taskId: task.id } 
  })
}

function isSelected(task: Task) {
  return task.id === currentTaskId.value
}

// ============ Task Creation ============

function showAddTask() {
  showNewTaskForm.value = true
  newTaskTitle.value = ''
}

function cancelAddTask() {
  showNewTaskForm.value = false
  newTaskTitle.value = ''
}

async function addTask() {
  if (!newTaskTitle.value.trim()) return
  
  try {
    const task = await tasksStore.createTask(projectId.value, newTaskTitle.value.trim())
    showNewTaskForm.value = false
    newTaskTitle.value = ''
    // Select the new task
    router.push({ 
      name: 'project-tasks', 
      params: { id: projectId.value, taskId: task.id } 
    })
  } catch {
    // Error handled in store
  }
}

// ============ Task Toggling ============

async function toggleTask(task: Task, event: Event) {
  event.stopPropagation()
  try {
    await tasksStore.toggleTask(projectId.value, task.id)
  } catch {
    // Error handled in store
  }
}

// ============ Toggle Active/Backlog ============

async function toggleActive(event: Event) {
  event.stopPropagation()
  if (!tasksStore.selectedTask) return
  
  try {
    const newIsActive = !tasksStore.selectedTask.is_active
    await tasksStore.updateTaskMeta(projectId.value, tasksStore.selectedTask.id, {
      is_active: newIsActive,
      section: newIsActive ? 'Active' : 'Backlog'
    })
  } catch {
    // Error handled in store
  }
}

// ============ Title Editing ============

function startEditTitle(task: Task, event: Event) {
  event.stopPropagation()
  editingTaskId.value = task.id
  editingTitle.value = task.title
}

function cancelEditTitle() {
  editingTaskId.value = null
  editingTitle.value = ''
}

async function saveTitle() {
  if (!editingTaskId.value || !editingTitle.value.trim()) {
    cancelEditTitle()
    return
  }
  
  try {
    await tasksStore.updateTaskMeta(projectId.value, editingTaskId.value, {
      title: editingTitle.value.trim()
    })
    cancelEditTitle()
  } catch {
    // Error handled in store
  }
}

// ============ Task Deletion ============

async function deleteTask(task: Task) {
  if (!confirm(`Delete task "${task.title}"?`)) return
  
  try {
    await tasksStore.deleteTask(projectId.value, task.id)
    
    // Navigate away if we deleted the selected task
    if (currentTaskId.value === task.id) {
      router.push({ name: 'project-tasks', params: { id: projectId.value } })
    }
  } catch {
    // Error handled in store
  }
}

// ============ Content Editing ============

function clearPendingSave() {
  if (saveTimeout) {
    clearTimeout(saveTimeout)
    saveTimeout = null
  }
  pendingSaveTaskId = null
}

// Save current content immediately before switching tasks
async function saveBeforeSwitch() {
  const taskIdToSave = pendingSaveTaskId
  const contentToSave = editorContent.value
  
  // Clear pending state first
  clearPendingSave()
  
  // Only save if we had a pending save for the current task
  if (!taskIdToSave || !tasksStore.selectedTask) return
  if (tasksStore.selectedTask.id !== taskIdToSave) return
  
  // Only save if content actually changed
  if (contentToSave === lastSavedContent.value) {
    console.log('[TasksView] Skipping save before switch - content unchanged')
    return
  }
  
  console.log('[TasksView] Saving before switch:', { taskIdToSave })
  try {
    await tasksStore.updateTaskContent(projectId.value, taskIdToSave, contentToSave)
    lastSavedContent.value = contentToSave
    gitStore.loadStatus()
  } catch {
    // Error handled in store
  }
}

function scheduleAutoSave() {
  console.log('[TasksView] scheduleAutoSave called for task:', currentTaskId.value)
  clearPendingSave()
  saveStatus.value = 'idle'
  // Capture the current task ID for this save operation
  pendingSaveTaskId = currentTaskId.value || null
  saveTimeout = window.setTimeout(saveContent, 1000)
}

async function saveContent() {
  const taskIdToSave = pendingSaveTaskId
  const contentToSave = editorContent.value
  
  // Clear pending state
  pendingSaveTaskId = null
  saveTimeout = null
  
  // Verify we're still on the same task - critical check to prevent overwrites
  if (!taskIdToSave || !tasksStore.selectedTask || currentTaskId.value !== taskIdToSave) {
    console.log('[TasksView] Skipping save - task changed:', { taskIdToSave, currentTaskId: currentTaskId.value })
    return
  }
  
  // Double-check the selected task matches
  if (tasksStore.selectedTask.id !== taskIdToSave) {
    console.log('[TasksView] Skipping save - selectedTask mismatch:', { taskIdToSave, selectedTaskId: tasksStore.selectedTask.id })
    return
  }
  
  // Final check: only save if content actually changed from last saved
  if (contentToSave === lastSavedContent.value) {
    console.log('[TasksView] Skipping save - content unchanged from last save')
    return
  }
  
  try {
    saveStatus.value = 'saving'
    await tasksStore.updateTaskContent(projectId.value, taskIdToSave, contentToSave)
    
    // Only update status if we're still on the same task
    if (currentTaskId.value === taskIdToSave) {
      lastSavedContent.value = contentToSave
      saveStatus.value = 'saved'
      setTimeout(() => {
        if (saveStatus.value === 'saved') saveStatus.value = 'idle'
      }, 2000)
    }
    gitStore.loadStatus()
  } catch {
    if (currentTaskId.value === taskIdToSave) {
      saveStatus.value = 'error'
    }
  }
}

// ============ Subtask Management ============

function openSubtaskInput() {
  showSubtaskInput.value = true
  newSubtaskTitle.value = ''
}

function closeSubtaskInput() {
  showSubtaskInput.value = false
  newSubtaskTitle.value = ''
}

async function addSubtask() {
  if (!newSubtaskTitle.value.trim() || !tasksStore.selectedTask) return
  try {
    await tasksStore.createTask(
      projectId.value,
      newSubtaskTitle.value.trim(),
      undefined,
      tasksStore.selectedTask.id
    )
    closeSubtaskInput()
  } catch {
    // Error handled in store
  }
}

// ============ Due Date Management ============

async function setDueDate(date: string) {
  if (!tasksStore.selectedTask) return
  try {
    await tasksStore.updateTaskMeta(projectId.value, tasksStore.selectedTask.id, {
      due_date: date || ''
    })
  } catch {
    // Error handled in store
  }
}

// ============ Recurrence Management ============

const recurrenceOptions = [
  { value: '', label: 'None' },
  { value: 'daily', label: 'Daily' },
  { value: 'weekly', label: 'Weekly' },
  { value: 'monthly', label: 'Monthly' },
  { value: 'yearly', label: 'Yearly' },
]

async function setRecurrence(recurrence: string) {
  if (!tasksStore.selectedTask) return
  try {
    await tasksStore.updateTaskMeta(projectId.value, tasksStore.selectedTask.id, {
      recurrence: recurrence || '',
      recurrence_interval: recurrence ? (tasksStore.selectedTask.recurrence_interval || 1) : undefined
    })
  } catch {
    // Error handled in store
  }
}

function recurrenceLabel(task: Task): string | null {
  if (!task.recurrence) return null
  const interval = task.recurrence_interval || 1
  if (interval === 1) {
    return task.recurrence.charAt(0).toUpperCase() + task.recurrence.slice(1)
  }
  return `Every ${interval} ${task.recurrence.replace('ly', '')}s`
}

// ============ Tag Management ============

function setTagFilter(tag: string | null) {
  activeTagFilter.value = activeTagFilter.value === tag ? null : tag
}

function openTagInput() {
  showTagInput.value = true
  newTagInput.value = ''
}

function closeTagInput() {
  showTagInput.value = false
  newTagInput.value = ''
}

async function addTag(tag?: string) {
  const tagToAdd = (tag || newTagInput.value).trim().toLowerCase()
  if (!tagToAdd || !tasksStore.selectedTask) return

  const currentTags = tasksStore.selectedTask.tags || []
  if (currentTags.includes(tagToAdd)) {
    closeTagInput()
    return
  }

  try {
    await tasksStore.updateTaskMeta(projectId.value, tasksStore.selectedTask.id, {
      tags: [...currentTags, tagToAdd]
    })
    closeTagInput()
  } catch {
    // Error handled in store
  }
}

async function removeTag(tag: string) {
  if (!tasksStore.selectedTask) return

  const currentTags = tasksStore.selectedTask.tags || []
  try {
    await tasksStore.updateTaskMeta(projectId.value, tasksStore.selectedTask.id, {
      tags: currentTags.filter(t => t !== tag)
    })
  } catch {
    // Error handled in store
  }
}

// ============ Navigation ============

function goToProject() {
  router.push({ name: 'project', params: { id: projectId.value } })
}

// ============ Due Date Formatting ============

function formatDueDate(dateStr?: string) {
  if (!dateStr) return null
  try {
    const date = new Date(dateStr)
    const now = new Date()
    const diffDays = Math.ceil((date.getTime() - now.getTime()) / (1000 * 60 * 60 * 24))
    
    if (diffDays < 0) return { text: 'Overdue', class: 'overdue' }
    if (diffDays === 0) return { text: 'Today', class: 'today' }
    if (diffDays === 1) return { text: 'Tomorrow', class: 'soon' }
    if (diffDays <= 7) return { text: `${diffDays} days`, class: 'soon' }
    return { text: date.toLocaleDateString(), class: '' }
  } catch {
    return null
  }
}

// ============ Watchers ============

// Watch for content changes - ONLY save when content differs from last saved
watch(editorContent, (newContent) => {
  // Skip if no task loaded
  if (!tasksStore.selectedTask) {
    return
  }
  
  // CRITICAL: Only schedule auto-save if content actually differs from last saved/loaded
  // This prevents unnecessary saves when just opening a task
  if (lastSavedContent.value !== null && newContent !== lastSavedContent.value) {
    console.log('[TasksView] Content changed from last saved, scheduling auto-save')
    scheduleAutoSave()
  }
})

watch(projectId, () => {
  // Clear any pending saves when switching projects
  clearPendingSave()
  editorContent.value = ''
  lastSavedContent.value = null
  
  tasksStore.loadProjectTasks(projectId.value)
  tasksStore.clearSelectedTask()
}, { immediate: true })

watch(currentTaskId, async (taskId, oldTaskId) => {
  console.log('[TasksView] currentTaskId changed:', { oldTaskId, taskId, pendingSaveTaskId })
  
  // Save any pending content from the previous task BEFORE switching
  if (oldTaskId && pendingSaveTaskId) {
    console.log('[TasksView] Has pending save, calling saveBeforeSwitch')
    await saveBeforeSwitch()
  } else {
    clearPendingSave()
  }
  saveStatus.value = 'idle'
  
  if (taskId) {
    // Load the new task
    console.log('[TasksView] Loading task:', taskId)
    await tasksStore.loadTask(projectId.value, taskId)
    
    // After loading, the selectedTask watcher handles setting content and editorKey
  } else {
    tasksStore.clearSelectedTask()
    editorContent.value = ''
    lastSavedContent.value = null
    editorKey.value = null
  }
}, { immediate: true })

watch(() => tasksStore.selectedTask, (task) => {
  if (task) {
    console.log('[TasksView] selectedTask changed, setting content length:', task.content?.length)
    editorContent.value = task.content
    
    // Track this as the "original" content - only save if user makes changes
    lastSavedContent.value = task.content
    
    // CRITICAL: Set editorKey AFTER content is set
    // This ensures the editor recreates with the correct defaultValue
    editorKey.value = task.id
    console.log('[TasksView] Updated editorKey to:', task.id)
  } else {
    // Handle case where task becomes null (e.g., load failed)
    console.log('[TasksView] selectedTask became null, clearing state')
    editorContent.value = ''
    lastSavedContent.value = null
    editorKey.value = null
  }
})

// ============ Lifecycle ============

onMounted(async () => {
  await projectsStore.loadProject(projectId.value)
  
  if (workspaceStore.activeProjectId !== projectId.value) {
    await workspaceStore.setActiveProject(projectId.value)
  }
  
  // Lock current task file if selected
  if (currentTaskId.value) {
    const taskPath = `projects/${projectId.value}/tasks/${currentTaskId.value}.md`
    lockFile(taskPath, 'editor')
  }
})

onUnmounted(() => {
  // Clear any pending auto-save to prevent saving after unmount
  clearPendingSave()
  
  if (currentTaskId.value) {
    const taskPath = `projects/${projectId.value}/tasks/${currentTaskId.value}.md`
    unlockFile(taskPath)
  }
  // Don't clear tasks - the sidebar needs them to show task counts
  tasksStore.clearSelectedTask()
})
</script>

<template>
  <div class="tasks-split-view">
    <!-- Tasks List Panel -->
    <div class="tasks-list-panel">
      <div class="panel-header">
        <div class="header-left">
          <button class="back-btn" @click="goToProject" title="Back to project">←</button>
          <h3>Tasks</h3>
        </div>
        <button class="primary small" @click="showAddTask">+ New</button>
      </div>

      <!-- New Task Form -->
      <div v-if="showNewTaskForm" class="new-task-form">
        <input
          v-model="newTaskTitle"
          type="text"
          placeholder="Task title..."
          class="new-task-input"
          @keyup.enter="addTask"
          @keyup.escape="cancelAddTask"
          autofocus
        />
        <div class="new-task-actions">
          <button class="primary small" @click="addTask" :disabled="!newTaskTitle.trim()">Add</button>
          <button class="small" @click="cancelAddTask">Cancel</button>
        </div>
      </div>

      <!-- Tag Filter -->
      <div v-if="tasksStore.projectTags.length > 0" class="tag-filter-bar">
        <button
          :class="['tag-filter-pill', { active: !activeTagFilter }]"
          @click="setTagFilter(null)"
        >All</button>
        <button
          v-for="tag in tasksStore.projectTags"
          :key="tag"
          :class="['tag-filter-pill', { active: activeTagFilter === tag }]"
          @click="setTagFilter(tag)"
        >{{ tag }}</button>
      </div>

      <div v-if="tasksStore.loading" class="loading-small">Loading...</div>
      
      <div v-else class="tasks-sections">
        <!-- Active Tasks -->
        <section v-if="filteredActiveTasks.length > 0" class="task-section">
          <h4>Active ({{ filteredActiveTasks.length }})</h4>
          <div class="task-list">
            <template v-for="task in filteredActiveTasks" :key="task.id">
              <div
                :class="['task-item', { selected: isSelected(task) }]"
                @click="selectTask(task)"
              >
                <button class="task-checkbox" @click="toggleTask(task, $event)" title="Mark complete">
                  &#9744;
                </button>
                <template v-if="editingTaskId === task.id">
                  <input
                    v-model="editingTitle"
                    type="text"
                    class="edit-title-input"
                    @keyup.enter="saveTitle"
                    @keyup.escape="cancelEditTitle"
                    @blur="saveTitle"
                    @click.stop
                    autofocus
                  />
                </template>
                <template v-else>
                  <div class="task-info">
                    <span class="task-title" @dblclick="startEditTitle(task, $event)">
                      {{ task.title }}
                    </span>
                    <div class="task-item-meta">
                      <span v-if="task.recurrence" class="recurrence-badge" :title="recurrenceLabel(task) ?? ''">&#x21bb;</span>
                      <span v-if="getSubtasks(task.id).length > 0" class="subtask-count">{{ getSubtasks(task.id).filter(s => s.completed).length }}/{{ getSubtasks(task.id).length }}</span>
                      <span
                        v-for="tag in task.tags?.slice(0, 2)"
                        :key="tag"
                        class="task-item-tag"
                        @click.stop="setTagFilter(tag)"
                      >{{ tag }}</span>
                      <span 
                        v-if="task.due_date && formatDueDate(task.due_date)" 
                        :class="['task-due', formatDueDate(task.due_date)?.class]"
                      >
                        {{ formatDueDate(task.due_date)?.text }}
                      </span>
                    </div>
                  </div>
                </template>
                <button 
                  class="delete-btn" 
                  @click.stop="deleteTask(task)" 
                  title="Delete task"
                >&times;</button>
              </div>
              <!-- Subtasks (indented) -->
              <div
                v-for="sub in getSubtasks(task.id)"
                :key="sub.id"
                :class="['task-item', 'subtask-item', { selected: isSelected(sub), completed: sub.completed }]"
                @click="selectTask(sub)"
              >
                <button class="task-checkbox" @click="toggleTask(sub, $event)">
                  {{ sub.completed ? '&#9745;' : '&#9744;' }}
                </button>
                <span :class="['task-title', { 'line-through': sub.completed }]">{{ sub.title }}</span>
              </div>
            </template>
          </div>
        </section>

        <!-- Backlog Tasks -->
        <section v-if="filteredBacklogTasks.length > 0" class="task-section">
          <h4>Backlog ({{ filteredBacklogTasks.length }})</h4>
          <div class="task-list">
            <template v-for="task in filteredBacklogTasks" :key="task.id">
              <div
                :class="['task-item', { selected: isSelected(task) }]"
                @click="selectTask(task)"
              >
                <button class="task-checkbox" @click="toggleTask(task, $event)" title="Mark complete">
                  &#9744;
                </button>
                <template v-if="editingTaskId === task.id">
                  <input
                    v-model="editingTitle"
                    type="text"
                    class="edit-title-input"
                    @keyup.enter="saveTitle"
                    @keyup.escape="cancelEditTitle"
                    @blur="saveTitle"
                    @click.stop
                    autofocus
                  />
                </template>
                <template v-else>
                  <div class="task-info">
                    <span class="task-title" @dblclick="startEditTitle(task, $event)">
                      {{ task.title }}
                    </span>
                    <div class="task-item-meta">
                      <span v-if="task.recurrence" class="recurrence-badge">&#x21bb;</span>
                      <span
                        v-for="tag in task.tags?.slice(0, 2)"
                        :key="tag"
                        class="task-item-tag"
                        @click.stop="setTagFilter(tag)"
                      >{{ tag }}</span>
                      <span 
                        v-if="task.due_date && formatDueDate(task.due_date)" 
                        :class="['task-due', formatDueDate(task.due_date)?.class]"
                      >
                        {{ formatDueDate(task.due_date)?.text }}
                      </span>
                    </div>
                  </div>
                </template>
                <button 
                  class="delete-btn" 
                  @click.stop="deleteTask(task)" 
                  title="Delete task"
                >&times;</button>
              </div>
              <div
                v-for="sub in getSubtasks(task.id)"
                :key="sub.id"
                :class="['task-item', 'subtask-item', { selected: isSelected(sub), completed: sub.completed }]"
                @click="selectTask(sub)"
              >
                <button class="task-checkbox" @click="toggleTask(sub, $event)">
                  {{ sub.completed ? '&#9745;' : '&#9744;' }}
                </button>
                <span :class="['task-title', { 'line-through': sub.completed }]">{{ sub.title }}</span>
              </div>
            </template>
          </div>
        </section>

        <!-- Completed Tasks -->
        <section v-if="filteredCompletedTasks.length > 0" class="task-section">
          <h4>Completed ({{ filteredCompletedTasks.length }})</h4>
          <div class="task-list">
            <div
              v-for="task in filteredCompletedTasks"
              :key="task.id"
              :class="['task-item', 'completed', { selected: isSelected(task) }]"
              @click="selectTask(task)"
            >
              <button class="task-checkbox" @click="toggleTask(task, $event)" title="Mark incomplete">
                &#9745;
              </button>
              <template v-if="editingTaskId === task.id">
                <input
                  v-model="editingTitle"
                  type="text"
                  class="edit-title-input"
                  @keyup.enter="saveTitle"
                  @keyup.escape="cancelEditTitle"
                  @blur="saveTitle"
                  @click.stop
                  autofocus
                />
              </template>
              <template v-else>
                <span class="task-title" @dblclick="startEditTitle(task, $event)">
                  {{ task.title }}
                </span>
              </template>
              <button 
                class="delete-btn" 
                @click.stop="deleteTask(task)" 
                title="Delete task"
              >&times;</button>
            </div>
          </div>
        </section>

        <!-- Empty State -->
        <div v-if="tasksStore.tasks.length === 0 && !showNewTaskForm" class="empty-list">
          <p>No tasks yet</p>
          <button class="primary small" @click="showAddTask">+ Add Task</button>
        </div>
        <div v-else-if="activeTagFilter && filteredActiveTasks.length === 0 && filteredBacklogTasks.length === 0 && filteredCompletedTasks.length === 0" class="empty-list">
          <p>No tasks with tag "{{ activeTagFilter }}"</p>
          <button class="small" @click="setTagFilter(null)">Clear Filter</button>
        </div>
      </div>
    </div>

    <!-- Task Detail Panel -->
    <div class="editor-panel">
      <template v-if="currentTaskId && tasksStore.selectedTask && editorKey">
        <div class="editor-header">
          <div class="task-header-info">
            <button 
              :class="['task-status-btn', tasksStore.selectedTask.is_active ? 'active' : 'backlog']"
              @click="toggleActive"
              :title="tasksStore.selectedTask.is_active ? 'Move to Backlog' : 'Make Active'"
            >
              {{ tasksStore.selectedTask.is_active ? 'Active' : 'Backlog' }}
            </button>
            <h3>{{ tasksStore.selectedTask.title }}</h3>
          </div>
          <div class="editor-actions">
            <span :class="['status', saveStatus]">
              <template v-if="saveStatus === 'saving'">Saving...</template>
              <template v-else-if="saveStatus === 'saved'">Saved</template>
              <template v-else-if="saveStatus === 'error'">Error</template>
            </span>
            <button @click="saveContent" :disabled="saveStatus === 'saving'">Save</button>
            <button 
              @click="toggleTask(tasksStore.selectedTask, $event)"
              :class="tasksStore.selectedTask.completed ? '' : 'primary'"
            >
              {{ tasksStore.selectedTask.completed ? 'Reopen' : 'Complete' }}
            </button>
          </div>
        </div>
        <!-- Tag Editor -->
        <div class="tag-editor-bar">
          <span
            v-for="tag in tasksStore.selectedTask.tags"
            :key="tag"
            class="tag-pill"
          >
            {{ tag }}
            <button class="tag-remove" @click="removeTag(tag)" title="Remove tag">&times;</button>
          </span>
          <div v-if="showTagInput" class="tag-input-wrapper">
            <input
              v-model="newTagInput"
              type="text"
              class="tag-input"
              placeholder="Tag name..."
              @keyup.enter="addTag()"
              @keyup.escape="closeTagInput"
              autofocus
            />
            <div v-if="tagSuggestions.length > 0" class="tag-suggestions">
              <div
                v-for="suggestion in tagSuggestions"
                :key="suggestion"
                class="tag-suggestion"
                @mousedown.prevent="addTag(suggestion)"
              >{{ suggestion }}</div>
            </div>
          </div>
          <button v-else class="tag-add-btn" @click="openTagInput">+ Tag</button>

          <span class="meta-separator">|</span>

          <!-- Due date picker (inline) -->
          <input
            type="date"
            class="due-date-input"
            :value="tasksStore.selectedTask.due_date || ''"
            @change="setDueDate(($event.target as HTMLInputElement).value)"
            title="Set due date"
          />
          <button
            v-if="tasksStore.selectedTask.due_date"
            class="tag-remove"
            @click="setDueDate('')"
            title="Clear due date"
          >&times;</button>

          <span class="meta-separator">|</span>

          <!-- Recurrence selector (inline) -->
          <select
            class="recurrence-select"
            :value="tasksStore.selectedTask.recurrence || ''"
            @change="setRecurrence(($event.target as HTMLSelectElement).value)"
          >
            <option v-for="opt in recurrenceOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
          <span v-if="tasksStore.selectedTask.recurrence" class="recurrence-label">
            &#x21bb; {{ recurrenceLabel(tasksStore.selectedTask) }}
          </span>
        </div>

        <!-- Subtasks Panel -->
        <div v-if="getSubtasks(tasksStore.selectedTask.id).length > 0 || showSubtaskInput" class="subtask-panel">
          <div class="subtask-header">
            <span class="subtask-label">Subtasks ({{ getSubtasks(tasksStore.selectedTask.id).filter(s => s.completed).length }}/{{ getSubtasks(tasksStore.selectedTask.id).length }})</span>
            <button v-if="!showSubtaskInput" class="tag-add-btn" @click="openSubtaskInput">+ Subtask</button>
          </div>
          <div class="subtask-list">
            <div
              v-for="sub in getSubtasks(tasksStore.selectedTask.id)"
              :key="sub.id"
              :class="['subtask-row', { completed: sub.completed }]"
            >
              <button class="task-checkbox" @click="toggleTask(sub, $event)">
                {{ sub.completed ? '&#9745;' : '&#9744;' }}
              </button>
              <span class="subtask-title" @click="selectTask(sub)">{{ sub.title }}</span>
            </div>
          </div>
          <div v-if="showSubtaskInput" class="subtask-input-row">
            <input
              v-model="newSubtaskTitle"
              type="text"
              class="tag-input"
              placeholder="Subtask title..."
              style="width: 100%; border-radius: 4px;"
              @keyup.enter="addSubtask"
              @keyup.escape="closeSubtaskInput"
              autofocus
            />
          </div>
        </div>
        <div v-else class="subtask-add-bar">
          <button class="tag-add-btn" @click="openSubtaskInput">+ Add subtask</button>
        </div>

        <div class="editor-content">
          <div class="editor-pane">
            <MilkdownEditor
              v-model="editorContent"
              :editor-key="editorKey"
              :project-id="projectId"
              placeholder="Add task description, notes, acceptance criteria..."
            />
          </div>
        </div>
      </template>

      <template v-else>
        <div class="editor-placeholder">
          <h3>Select a task</h3>
          <p>Choose a task from the list to view and edit its description.</p>
          <button class="primary" @click="showAddTask">+ New Task</button>
        </div>
      </template>
    </div>

    <div v-if="tasksStore.error" class="error-banner">
      {{ tasksStore.error }}
      <button @click="tasksStore.clearError">Dismiss</button>
    </div>
  </div>
</template>

<style scoped>
.tasks-split-view {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

/* Tasks List Panel */
.tasks-list-panel {
  width: 280px;
  flex-shrink: 0;
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  background: var(--color-bg-secondary);
  overflow: hidden;
}

.panel-header {
  height: 52px;
  min-height: 52px;
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.back-btn {
  padding: 4px 8px;
  font-size: 14px;
}

.panel-header h3 {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
}

button.small {
  padding: 4px 10px;
  font-size: 12px;
}

/* New Task Form */
.new-task-form {
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg);
  flex-shrink: 0;
}

.new-task-input {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 13px;
  outline: none;
  margin-bottom: 8px;
  box-sizing: border-box;
}

.new-task-input:focus {
  border-color: var(--color-primary);
}

.new-task-actions {
  display: flex;
  gap: 8px;
}

/* Task Sections */
.tasks-sections {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 8px 0;
}

.task-section {
  margin-bottom: 8px;
}

.task-section h4 {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 8px 16px 4px;
  margin: 0;
}

.task-list {
  display: flex;
  flex-direction: column;
}

.task-item {
  padding: 10px 16px;
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  transition: background 0.15s;
}

.task-item:hover {
  background: var(--color-border);
}

.task-item:hover .delete-btn {
  opacity: 1;
}

.task-item.selected {
  background: var(--color-primary);
  color: white;
}

.task-item.selected .task-checkbox {
  color: white;
}

.task-item.selected .delete-btn {
  color: rgba(255, 255, 255, 0.7);
}

.task-item.selected .delete-btn:hover {
  color: white;
  background: rgba(255, 255, 255, 0.2);
}

.task-item.completed .task-title {
  text-decoration: line-through;
  opacity: 0.7;
}

.task-checkbox {
  flex-shrink: 0;
  padding: 0;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 14px;
  color: var(--color-text);
}

.task-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  overflow: hidden;
}

.task-title {
  font-size: 13px;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.task-due {
  font-size: 11px;
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

.edit-title-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--color-primary);
  border-radius: 4px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 13px;
  outline: none;
  min-width: 0;
}

.delete-btn {
  flex-shrink: 0;
  padding: 2px 6px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-size: 16px;
  color: var(--color-text-secondary);
  opacity: 0;
  border-radius: 4px;
  transition: opacity 0.15s, background 0.15s;
}

.delete-btn:hover {
  background: var(--color-danger);
  color: white;
}

.loading-small,
.empty-list {
  padding: 24px 16px;
  text-align: center;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.empty-list p {
  margin-bottom: 12px;
}

/* Editor Panel */
.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.editor-header {
  height: 52px;
  min-height: 52px;
  padding: 0 16px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
  gap: 12px;
}

.task-header-info {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
  flex: 1;
  overflow: hidden;
}

.task-status-btn {
  flex-shrink: 0;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  border: none;
  transition: background 0.15s, opacity 0.15s;
}

.task-status-btn.active {
  background: var(--color-primary);
  color: white;
}

.task-status-btn.active:hover {
  opacity: 0.85;
}

.task-status-btn.backlog {
  background: var(--color-text-secondary);
  color: white;
}

.task-status-btn.backlog:hover {
  background: var(--color-primary);
}

.editor-header h3 {
  font-size: 14px;
  font-weight: 500;
  margin: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

.editor-actions {
  flex-shrink: 0;
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

.editor-content {
  flex: 1;
  overflow: hidden;
  display: flex;
}

.editor-pane {
  flex: 1;
  display: flex;
  overflow: hidden;
  min-width: 0;
}

.editor-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  color: var(--color-text-secondary);
  padding: 32px;
}

.editor-placeholder h3 {
  margin-bottom: 8px;
  color: var(--color-text);
}

.editor-placeholder p {
  margin-bottom: 16px;
}

/* Subtask items in list panel */
.subtask-item {
  padding-left: 36px !important;
}

.subtask-item .task-title {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.subtask-item .task-title.line-through {
  text-decoration: line-through;
  opacity: 0.6;
}

/* Subtask count badge */
.subtask-count {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 8px;
  background: var(--color-border);
  color: var(--color-text-secondary);
  white-space: nowrap;
}

/* Recurrence badge */
.recurrence-badge {
  font-size: 12px;
  color: var(--color-primary);
}

/* Meta separator in tag-editor-bar */
.meta-separator {
  color: var(--color-border);
  font-size: 14px;
  user-select: none;
}

/* Due date input */
.due-date-input {
  font-size: 11px;
  padding: 2px 6px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  cursor: pointer;
  outline: none;
}

.due-date-input:focus {
  border-color: var(--color-primary);
}

.due-date-input::-webkit-calendar-picker-indicator {
  cursor: pointer;
  opacity: 0.6;
  filter: invert(0.8);
}

/* Recurrence select */
.recurrence-select {
  font-size: 11px;
  padding: 2px 6px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  cursor: pointer;
  outline: none;
}

.recurrence-select:focus {
  border-color: var(--color-primary);
}

.recurrence-label {
  font-size: 11px;
  color: var(--color-primary);
  white-space: nowrap;
}

/* Subtask panel in detail view */
.subtask-panel {
  border-bottom: 1px solid var(--color-border);
  padding: 8px 16px;
  flex-shrink: 0;
}

.subtask-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.subtask-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.subtask-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.subtask-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 0;
}

.subtask-row.completed .subtask-title {
  text-decoration: line-through;
  opacity: 0.6;
}

.subtask-title {
  font-size: 13px;
  cursor: pointer;
  flex: 1;
}

.subtask-title:hover {
  color: var(--color-primary);
}

.subtask-input-row {
  margin-top: 6px;
}

.subtask-add-bar {
  padding: 4px 16px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
}

/* Tag Filter Bar */
.tag-filter-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--color-border);
  flex-shrink: 0;
  overflow-x: auto;
}

.tag-filter-pill {
  font-size: 11px;
  padding: 3px 10px;
  border-radius: 12px;
  border: 1px solid var(--color-border);
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.12s;
}

.tag-filter-pill:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.tag-filter-pill.active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: white;
}

/* Task Item Tags */
.task-item-meta {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-wrap: wrap;
}

.task-item-tag {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 8px;
  background: var(--color-border);
  color: var(--color-text-secondary);
  cursor: pointer;
  white-space: nowrap;
}

.task-item-tag:hover {
  background: var(--color-primary);
  color: white;
}

.task-item.selected .task-item-tag {
  background: rgba(255, 255, 255, 0.25);
  color: white;
}

/* Tag Editor Bar (detail panel) */
.tag-editor-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px 16px;
  border-bottom: 1px solid var(--color-border);
  align-items: center;
  min-height: 36px;
  flex-shrink: 0;
}

.tag-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  padding: 3px 8px;
  border-radius: 10px;
  background: var(--color-border);
  color: var(--color-text);
}

.tag-remove {
  padding: 0;
  border: none;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  display: inline-flex;
  align-items: center;
}

.tag-remove:hover {
  color: var(--color-danger);
}

.tag-add-btn {
  font-size: 11px;
  padding: 3px 8px;
  border: 1px dashed var(--color-border);
  border-radius: 10px;
  background: transparent;
  color: var(--color-text-secondary);
  cursor: pointer;
}

.tag-add-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.tag-input-wrapper {
  position: relative;
}

.tag-input {
  width: 120px;
  padding: 3px 8px;
  border: 1px solid var(--color-primary);
  border-radius: 10px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 12px;
  outline: none;
}

.tag-suggestions {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 4px;
  min-width: 140px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 50;
  overflow: hidden;
}

.tag-suggestion {
  padding: 6px 12px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.1s;
}

.tag-suggestion:hover {
  background: var(--color-bg-hover);
}

/* Error Banner */
.error-banner {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 12px 16px;
  background: var(--color-danger);
  color: white;
  font-size: 13px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.error-banner button {
  background: transparent;
  border: 1px solid white;
  color: white;
}
</style>
