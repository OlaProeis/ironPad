<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { libraryApi } from '../../api/client'
import { clearLibraryRoot, getLibraryRoot, setLibraryRoot } from '../../lib/libraryRoot'

const props = defineProps<{
  projectId: string
}>()

const emit = defineEmits<{
  'root-changed': []
}>()

const storedRoot = ref<string | null>(null)
const draftPath = ref('')
const browsing = ref(false)
const error = ref('')

function refreshFromStorage() {
  storedRoot.value = getLibraryRoot(props.projectId)
  if (storedRoot.value) {
    draftPath.value = storedRoot.value
  }
}

function onSet() {
  const value = draftPath.value.trim()
  if (!value) return
  setLibraryRoot(props.projectId, value)
  refreshFromStorage()
  emit('root-changed')
}

async function onBrowse() {
  if (browsing.value) return
  error.value = ''
  browsing.value = true
  try {
    const current = storedRoot.value ?? undefined
    const result = await libraryApi.pickLibrary('directory', current)
    const path = result.paths[0]
    if (!path) return
    draftPath.value = path
    onSet()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Could not open the folder dialog.'
  } finally {
    browsing.value = false
  }
}

function onClear() {
  clearLibraryRoot(props.projectId)
  draftPath.value = ''
  refreshFromStorage()
  emit('root-changed')
}

watch(
  () => props.projectId,
  () => {
    refreshFromStorage()
  }
)

onMounted(() => {
  refreshFromStorage()
})
</script>

<template>
  <div class="library-root-bar">
    <div class="root-label">
      <span class="title">Folder root</span>
      <span class="current" data-testid="library-root-current">
        {{ storedRoot ?? 'No folder root' }}
      </span>
    </div>
    <div class="root-actions">
      <input
        v-model="draftPath"
        type="text"
        class="root-input"
        placeholder="Absolute path to coding repo"
        data-testid="library-root-input"
      />
      <button type="button" class="small" data-testid="library-root-browse" :disabled="browsing" @click="onBrowse">
        {{ browsing ? 'Waiting…' : 'Browse' }}
      </button>
      <button type="button" class="small" data-testid="library-root-set" @click="onSet">Set</button>
      <button type="button" class="small" data-testid="library-root-clear" @click="onClear">Clear</button>
    </div>
    <p v-if="error" class="root-error" data-testid="library-root-error">{{ error }}</p>
  </div>
</template>

<style scoped>
.library-root-bar {
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.root-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.title {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--color-text-secondary);
}

.current {
  font-size: 12px;
  color: var(--color-text);
  word-break: break-all;
}

.root-actions {
  display: flex;
  gap: 6px;
  align-items: center;
}

.root-input {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  background: var(--color-bg);
  color: var(--color-text);
  border: 1px solid var(--color-border);
  padding: 4px 8px;
}

.root-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

button.small {
  padding: 4px 8px;
  flex-shrink: 0;
}

.root-error {
  margin: 0;
  color: var(--color-danger, #c44);
  font-size: 12px;
}
</style>