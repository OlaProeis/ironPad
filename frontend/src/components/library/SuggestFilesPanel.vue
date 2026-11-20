<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { libraryApi } from '../../api/client'
import { getLibraryRoot } from '../../lib/libraryRoot'
import {
  dismissSuggest,
  findSuggestions,
  isSuggestDismissed,
  titleForSuggestedSource,
} from '../../lib/librarySuggest'
import { usePromptsStore } from '../../stores/prompts'

const props = defineProps<{
  projectId: string
  refreshToken?: number
}>()

const promptsStore = usePromptsStore()

const visible = ref(false)
const suggestions = ref<string[]>([])
const checked = ref<Set<string>>(new Set())
const loading = ref(false)
const adding = ref(false)
const error = ref('')

function existingLinkSources(): string[] {
  return promptsStore.prompts
    .filter((p) => p.kind === 'link' && p.source)
    .map((p) => p.source as string)
}

function activeFolderPayload(): string | undefined {
  const folder = promptsStore.activeFolder
  if (!folder || folder === 'root') return undefined
  return folder
}

function syncChecked(paths: string[]) {
  checked.value = new Set(paths)
}

async function refresh() {
  error.value = ''
  const root = getLibraryRoot(props.projectId)
  if (!root || isSuggestDismissed(props.projectId)) {
    visible.value = false
    suggestions.value = []
    return
  }

  loading.value = true
  try {
    const found = await findSuggestions(root, existingLinkSources(), libraryApi.browseLibrary)
    suggestions.value = found
    syncChecked(found)
    visible.value = found.length > 0
  } catch (err) {
    suggestions.value = []
    visible.value = false
    error.value = err instanceof Error ? err.message : 'Could not load suggestions.'
  } finally {
    loading.value = false
  }
}

function toggle(path: string) {
  const next = new Set(checked.value)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  checked.value = next
}

function onDismiss() {
  dismissSuggest(props.projectId)
  visible.value = false
  suggestions.value = []
}

async function onAddSelected() {
  if (checked.value.size === 0) return
  adding.value = true
  error.value = ''
  try {
    const chosen = suggestions.value.filter((p) => checked.value.has(p))
    for (const source of chosen) {
      await promptsStore.createPrompt({
        kind: 'link',
        title: titleForSuggestedSource(source),
        folder: activeFolderPayload(),
        tags: [],
        template: false,
        source,
      })
    }
    await refresh()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Could not create links.'
  } finally {
    adding.value = false
  }
}

watch(
  () => [props.projectId, props.refreshToken] as const,
  () => {
    void refresh()
  }
)

watch(
  () => promptsStore.prompts,
  () => {
    if (!visible.value && !getLibraryRoot(props.projectId)) return
    void refresh()
  },
  { deep: true }
)

onMounted(() => {
  void refresh()
})
</script>

<template>
  <div v-if="visible" class="suggest-files-panel" data-testid="suggest-files-panel">
    <div class="suggest-header">
      <span class="title">Suggested repo files</span>
    </div>
    <ul class="suggest-list">
      <li v-for="path in suggestions" :key="path" class="suggest-item">
        <label>
          <input
            type="checkbox"
            data-testid="suggest-file-checkbox"
            :checked="checked.has(path)"
            @change="toggle(path)"
          />
          {{ path }}
        </label>
      </li>
    </ul>
    <p v-if="error" class="suggest-error" data-testid="suggest-error">{{ error }}</p>
    <div class="suggest-actions">
      <button
        type="button"
        class="small primary"
        data-testid="suggest-add-selected"
        :disabled="adding || checked.size === 0"
        @click="onAddSelected"
      >
        Add selected
      </button>
      <button type="button" class="small" data-testid="suggest-dismiss" @click="onDismiss">
        Dismiss
      </button>
    </div>
  </div>
</template>

<style scoped>
.suggest-files-panel {
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.suggest-header .title {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--color-text-secondary);
}

.suggest-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.suggest-item label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  cursor: pointer;
}

.suggest-error {
  margin: 0;
  font-size: 12px;
  color: var(--color-danger, #c62828);
}

.suggest-actions {
  display: flex;
  gap: 6px;
}
</style>