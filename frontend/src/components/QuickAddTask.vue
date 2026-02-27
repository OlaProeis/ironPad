<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useProjectsStore, useTasksStore } from '../stores'
import { tasksApi } from '../api/client'

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
  (e: 'created', taskId: string, projectId: string): void
}>()

const projectsStore = useProjectsStore()
const tasksStore = useTasksStore()

// Form state
const selectedProjectId = ref('')
const title = ref('')
const dueDate = ref('')
const estimatedMinutes = ref<number | undefined>(undefined)
const isSubmitting = ref(false)
const error = ref('')

// Refs for focus
const titleInput = ref<HTMLInputElement | null>(null)

// Time presets
const timePresets = [
  { minutes: 15, label: '15m' },
  { minutes: 30, label: '30m' },
  { minutes: 60, label: '1h' },
  { minutes: 120, label: '2h' },
  { minutes: 240, label: '4h' },
]

// Computed
const canSubmit = computed(() => {
  return title.value.trim() && selectedProjectId.value && !isSubmitting.value
})

const today = computed(() => {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
})

// Reset form when opened
onMounted(() => {
  if (props.visible) {
    resetForm()
    nextTick(() => {
      titleInput.value?.focus()
    })
  }
})

// Watch for visibility changes
import { watch } from 'vue'
watch(() => props.visible, (newValue) => {
  if (newValue) {
    resetForm()
    nextTick(() => {
      titleInput.value?.focus()
    })
  }
})

function resetForm() {
  // Default to first project or currently active project
  const activeProject = projectsStore.projects.find(p => p.id === projectsStore.activeProjectId)
  selectedProjectId.value = activeProject?.id || projectsStore.projects[0]?.id || ''
  title.value = ''
  dueDate.value = today.value
  estimatedMinutes.value = undefined
  error.value = ''
}

function selectTimePreset(minutes: number) {
  if (estimatedMinutes.value === minutes) {
    estimatedMinutes.value = undefined
  } else {
    estimatedMinutes.value = minutes
  }
}

async function submit() {
  if (!canSubmit.value) return

  isSubmitting.value = true
  error.value = ''

  try {
    // Create task via API
    const task = await tasksApi.create(selectedProjectId.value, title.value.trim())

    // Update with due date if set
    if (dueDate.value) {
      await tasksApi.updateMeta(selectedProjectId.value, task.id, {
        due_date: dueDate.value
      })
    }

    // Update with time estimate if set
    if (estimatedMinutes.value) {
      await tasksApi.updateMeta(selectedProjectId.value, task.id, {
        estimated_minutes: estimatedMinutes.value
      })
    }

    // Refresh tasks in store
    await tasksStore.loadAllTasks()
    if (tasksStore.activeProjectId === selectedProjectId.value) {
      await tasksStore.loadProjectTasks(selectedProjectId.value)
    }

    emit('created', task.id, selectedProjectId.value)
    emit('close')
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to create task'
  } finally {
    isSubmitting.value = false
  }
}

function onKeydown(e: KeyboardEvent) {
  // Ctrl+Enter to submit
  if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
    e.preventDefault()
    submit()
  }
  // Escape to close
  if (e.key === 'Escape') {
    e.preventDefault()
    emit('close')
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="quick-add-modal-overlay" @click.self="emit('close')">
      <div class="quick-add-modal" @keydown="onKeydown">
        <div class="modal-header">
          <h3>Quick Add Task</h3>
          <button class="close-btn" @click="emit('close')" title="Close (Esc)">&times;</button>
        </div>

        <div class="modal-body">
          <!-- Project selector -->
          <div class="form-group">
            <label for="project-select">Project</label>
            <select
              id="project-select"
              v-model="selectedProjectId"
              class="form-select"
            >
              <option v-for="project in projectsStore.projects" :key="project.id" :value="project.id">
                {{ project.name }}
              </option>
            </select>
          </div>

          <!-- Title input -->
          <div class="form-group">
            <label for="task-title">Task Title</label>
            <input
              id="task-title"
              ref="titleInput"
              v-model="title"
              type="text"
              class="form-input"
              placeholder="What needs to be done?"
              @keyup.enter="submit"
            />
          </div>

          <!-- Due date -->
          <div class="form-group">
            <label for="due-date">Due Date (optional)</label>
            <input
              id="due-date"
              v-model="dueDate"
              type="date"
              class="form-input date-input"
            />
          </div>

          <!-- Time estimate -->
          <div class="form-group">
            <label>Time Estimate (optional)</label>
            <div class="time-presets">
              <button
                v-for="preset in timePresets"
                :key="preset.minutes"
                :class="['preset-btn', { active: estimatedMinutes === preset.minutes }]"
                @click="selectTimePreset(preset.minutes)"
                type="button"
              >
                {{ preset.label }}
              </button>
            </div>
          </div>

          <!-- Error message -->
          <div v-if="error" class="error-message">
            {{ error }}
          </div>
        </div>

        <div class="modal-footer">
          <span class="hint">Ctrl+Enter to submit</span>
          <div class="actions">
            <button class="secondary" @click="emit('close')" :disabled="isSubmitting">
              Cancel
            </button>
            <button
              class="primary"
              @click="submit"
              :disabled="!canSubmit"
            >
              {{ isSubmitting ? 'Creating...' : 'Add Task' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.quick-add-modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.quick-add-modal {
  background: var(--color-bg);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  width: 100%;
  max-width: 420px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--color-border);
}

.modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: 0;
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.close-btn:hover {
  background: var(--color-bg-hover);
  color: var(--color-text);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
  margin-bottom: 6px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

.form-input,
.form-select {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
}

.form-input:focus,
.form-select:focus {
  border-color: var(--color-primary);
}

.date-input {
  font-family: inherit;
}

.date-input::-webkit-calendar-picker-indicator {
  cursor: pointer;
  opacity: 0.6;
  filter: invert(0.8);
}

.time-presets {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.preset-btn {
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg-secondary);
  color: var(--color-text);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.12s;
}

.preset-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
}

.preset-btn.active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: white;
}

.error-message {
  padding: 10px 12px;
  background: rgba(248, 81, 73, 0.1);
  border: 1px solid rgba(248, 81, 73, 0.3);
  border-radius: 6px;
  color: var(--color-danger);
  font-size: 13px;
}

.modal-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
}

.hint {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.actions {
  display: flex;
  gap: 8px;
}

.actions button {
  padding: 8px 16px;
  font-size: 13px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s;
}

.actions button.secondary {
  background: transparent;
  border: 1px solid var(--color-border);
  color: var(--color-text);
}

.actions button.secondary:hover {
  background: var(--color-bg-hover);
}

.actions button.primary {
  background: var(--color-primary);
  border: 1px solid var(--color-primary);
  color: white;
}

.actions button.primary:hover:not(:disabled) {
  opacity: 0.9;
}

.actions button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
