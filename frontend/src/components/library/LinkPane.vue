<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Prompt } from '../../types'
import { libraryApi } from '../../api/client'
import { getLibraryRoot } from '../../lib/libraryRoot'
import { usePromptsStore } from '../../stores/prompts'
import LibraryRootBar from './LibraryRootBar.vue'

const props = defineProps<{
  prompt: Prompt
}>()

const store = usePromptsStore()
const relinkSource = ref('')
const openError = ref('')

const actionsDisabled = computed(
  () =>
    props.prompt.source_missing === true ||
    props.prompt.root_required === true ||
    Boolean(props.prompt.source_error),
)

async function onReload() {
  await store.reloadOpen()
}

async function onRelink() {
  const source = relinkSource.value.trim()
  if (!source) return
  await store.updatePrompt(props.prompt.id, { source })
}

async function onCopyPath() {
  if (actionsDisabled.value) return
  const path = props.prompt.resolved_path
  if (!path) return
  await navigator.clipboard.writeText(path)
}

async function onOpenExternally() {
  if (actionsDisabled.value) return
  openError.value = ''
  const root = props.prompt.project_id
    ? getLibraryRoot(props.prompt.project_id) ?? undefined
    : undefined
  try {
    await libraryApi.openLibraryItem(props.prompt.id, root)
  } catch (err) {
    openError.value = err instanceof Error ? err.message : String(err)
  }
}
</script>

<template>
  <div class="link-pane">
    <div class="link-header">
      <div class="paths">
        <span class="source" data-testid="link-source">{{ prompt.source }}</span>
        <span v-if="prompt.resolved_path" class="resolved" data-testid="link-resolved-path">
          {{ prompt.resolved_path }}
        </span>
      </div>
      <div class="link-actions">
        <button
          type="button"
          data-testid="link-copy-path"
          :disabled="actionsDisabled"
          @click="onCopyPath"
        >
          Copy path
        </button>
        <button
          type="button"
          data-testid="link-open-externally"
          :disabled="actionsDisabled"
          @click="onOpenExternally"
        >
          Open externally
        </button>
        <button type="button" data-testid="link-reload" @click="onReload">Reload</button>
      </div>
    </div>
    <p v-if="openError" class="link-open-error" data-testid="link-open-error">{{ openError }}</p>

    <div v-if="prompt.source_missing" class="link-state" data-testid="link-missing">
      <p class="stored-path" data-testid="link-stored-path">{{ prompt.source }}</p>
      <p>File not found</p>
      <form class="relink" @submit.prevent="onRelink">
        <input
          v-model="relinkSource"
          type="text"
          data-testid="link-relink-input"
          :placeholder="prompt.scope === 'project' ? 'Relative path' : 'Absolute path'"
        />
        <button type="submit" data-testid="link-relink">Relink</button>
      </form>
    </div>

    <div v-else-if="prompt.root_required" class="link-state" data-testid="link-root-required">
      <p>Set a folder root for this project to open this link</p>
      <LibraryRootBar v-if="prompt.project_id" :project-id="prompt.project_id" />
    </div>

    <div v-else-if="prompt.source_error" class="link-state" data-testid="link-source-error">
      <p>{{ prompt.source_error }}</p>
    </div>

    <pre v-else class="link-raw" data-testid="link-content">{{ prompt.content }}</pre>
  </div>
</template>

<style scoped>
.link-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
  border-right: 1px solid var(--color-border);
}

.link-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border);
}

.link-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}

.link-open-error {
  margin: 0;
  padding: 8px 12px;
  font-size: 12px;
  color: var(--color-danger, #c62828);
  border-bottom: 1px solid var(--color-border);
}

.paths {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  font-size: 12px;
  word-break: break-all;
}

.source {
  color: var(--color-text-primary);
}

.resolved {
  color: var(--color-text-secondary);
}

.link-state {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.relink {
  display: flex;
  gap: 8px;
}

.relink input {
  flex: 1;
}

.link-raw {
  margin: 0;
  padding: 12px;
  overflow: auto;
  white-space: pre-wrap;
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 13px;
}
</style>
