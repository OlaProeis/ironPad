<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import type { CommitDetail, DiffInfo, PromptSummary } from '../types'
import { usePromptsStore, useWorkspaceStore } from '../stores'
import { gitApi, promptsApi } from '../api/client'
import AddLinkDialog from '../components/library/AddLinkDialog.vue'
import { buildBundle } from '../lib/copyBundle'
import { fileTitle, isLinkablePath } from '../lib/libraryFiles'
import { getLibraryRoot } from '../lib/libraryRoot'
import { sameLinkedSource, sourceForLibraryFile } from '../lib/markdownTarget'
import LibraryRootBar from '../components/library/LibraryRootBar.vue'
import LinkPane from '../components/library/LinkPane.vue'
import RenderedMarkdown from '../components/library/RenderedMarkdown.vue'
import SuggestFilesPanel from '../components/library/SuggestFilesPanel.vue'
import {
  defaultReadingMode,
  getReadingMode,
  setReadingMode,
  type ReadingMode,
} from '../lib/readingMode'

const props = defineProps<{
  id?: string
  promptId?: string
}>()

const route = useRoute()
const router = useRouter()
const promptsStore = usePromptsStore()
const workspaceStore = useWorkspaceStore()

const routeProjectId = computed(() => props.id || (route.params.id as string | undefined))
const routePromptId = computed(() => props.promptId || (route.params.promptId as string | undefined))
const isProjectScope = computed(() => !!routeProjectId.value)

const draftTitle = ref('')
const draftFolder = ref('root')
const draftDescription = ref('')
const draftTags = ref('')
const draftContent = ref('')
const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')
const creating = ref(false)
const addLinkOpen = ref(false)
const libraryRootRefresh = ref(0)
const filtersCollapsed = ref(true)

const VERSIONS_EXPANDED_KEY = 'ironpad-library-versions-expanded'

function readVersionsExpanded(): boolean {
  try {
    if (typeof localStorage === 'undefined') return false
    const stored = localStorage.getItem(VERSIONS_EXPANDED_KEY)
    if (stored === 'true') return true
    return false
  } catch {
    return false
  }
}

const versionsExpanded = ref(readVersionsExpanded())

function toggleVersionsExpanded() {
  versionsExpanded.value = !versionsExpanded.value
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(VERSIONS_EXPANDED_KEY, String(versionsExpanded.value))
    }
  } catch {
    // private mode / blocked storage
  }
}

const copyStatus = ref<'idle' | 'raw-copied' | 'rendered-copied' | 'error'>('idle')
const selectedIds = ref<Set<string>>(new Set())
const variableValues = ref<Record<string, string>>({})
const readingMode = ref<ReadingMode>('raw')
const followError = ref('')

const fileHistory = ref<CommitDetail[]>([])
const historyLoading = ref(false)
const selectedCommitId = ref<string | null>(null)
const selectedCommitDiff = ref<DiffInfo | null>(null)
const commitDiffLoading = ref(false)

let searchDebounce: number | null = null

const semanticScoreByPromptId = computed(() => {
  const map = new Map<string, number>()
  for (const result of promptsStore.semanticResults) {
    map.set(result.prompt.id, result.score)
  }
  return map
})

const projectScopeAllowed = computed(() => !!workspaceStore.activeProjectId)

const selectedPrompt = computed(() => promptsStore.selectedPrompt)
const prompts = computed(() => promptsStore.visiblePrompts)
const promptVariables = computed(() => {
  const seen = new Set<string>()
  const vars: string[] = []
  const matches = draftContent.value.matchAll(/\{\{\s*([a-zA-Z0-9_.-]+)\s*\}\}/g)
  for (const m of matches) {
    const name = m[1]?.trim()
    if (!name || seen.has(name)) continue
    seen.add(name)
    vars.push(name)
  }
  return vars
})
const renderedPrompt = computed(() =>
  draftContent.value.replace(/\{\{\s*([a-zA-Z0-9_.-]+)\s*\}\}/g, (_, varName: string) => {
    const key = String(varName).trim()
    return variableValues.value[key] ?? ''
  })
)

const templateVariablesEnabled = computed(() => {
  const item = selectedPrompt.value
  if (!item) return false
  return item.kind === 'prompt' || (item.kind === 'link' && item.template)
})

const renderedAvailable = computed(() => {
  const item = selectedPrompt.value
  if (!item) return false
  if (item.kind === 'link' && item.supports_rendered === false) return false
  return true
})

const linkBlocked = computed(() => {
  const item = selectedPrompt.value
  if (!item || item.kind !== 'link') return false
  return (
    item.root_required === true ||
    item.source_missing === true ||
    Boolean(item.source_error)
  )
})

const renderedSource = computed(() => {
  const item = selectedPrompt.value
  if (!item) return ''
  const substitute = item.kind === 'prompt' || (item.kind === 'link' && item.template)
  return substitute ? renderedPrompt.value : draftContent.value
})

const markdownBasePath = computed(() => {
  const item = selectedPrompt.value
  if (!item || item.kind !== 'link' || linkBlocked.value) return null
  return item.resolved_path ?? null
})

const markdownLinkRoot = computed(() => {
  libraryRootRefresh.value
  if (!isProjectScope.value || !routeProjectId.value) return null
  return getLibraryRoot(routeProjectId.value)
})

function applyReadingMode() {
  const item = selectedPrompt.value
  if (!item) {
    readingMode.value = 'raw'
    return
  }
  if (item.kind === 'link' && item.supports_rendered === false) {
    readingMode.value = 'raw'
    return
  }
  readingMode.value = getReadingMode(item.id, defaultReadingMode(item))
}

function chooseReadingMode(mode: ReadingMode) {
  const item = selectedPrompt.value
  if (!item) return
  if (mode === 'rendered' && !renderedAvailable.value) return
  readingMode.value = mode
  setReadingMode(item.id, mode)
}

function parseTags(raw: string): string[] {
  return raw
    .split(',')
    .map((t) => t.trim().toLowerCase())
    .filter(Boolean)
}

function syncDraftFromSelected() {
  const p = promptsStore.selectedPrompt
  if (!p) {
    draftTitle.value = ''
    draftFolder.value = 'root'
    draftDescription.value = ''
    draftTags.value = ''
    draftContent.value = ''
    return
  }
  draftTitle.value = p.title
  draftFolder.value = p.folder || 'root'
  draftDescription.value = p.description || ''
  draftTags.value = p.tags.join(', ')
  draftContent.value = p.content
}

function baseRouteName() {
  return isProjectScope.value ? 'project-prompts' : 'prompts'
}

function navigateToPrompt(id?: string) {
  const name = baseRouteName()
  if (isProjectScope.value) {
    router.push({
      name,
      params: {
        id: routeProjectId.value,
        ...(id ? { promptId: id } : {})
      }
    })
  } else {
    router.push({
      name,
      params: {
        ...(id ? { promptId: id } : {})
      }
    })
  }
}

async function selectPrompt(prompt: PromptSummary) {
  navigateToPrompt(prompt.id)
  await promptsStore.loadPrompt(prompt.id)
}

async function openLinkedFile(absPath: string) {
  followError.value = ''
  if (!isLinkablePath(absPath)) {
    followError.value = 'That file can’t be opened in the library.'
    return
  }
  const projectId = promptsStore.activeProjectId
  const root =
    promptsStore.activeScope === 'project' && projectId ? getLibraryRoot(projectId) : null
  const existing = promptsStore.prompts.find(
    (item) => item.kind === 'link' && sameLinkedSource(item.source, absPath, root)
  )
  if (existing) {
    await selectPrompt(existing)
    return
  }
  const folder = promptsStore.activeFolder
  try {
    const created = await promptsStore.createPrompt({
      kind: 'link',
      title: fileTitle(absPath),
      folder: !folder || folder === 'root' ? undefined : folder,
      tags: [],
      template: false,
      source: sourceForLibraryFile(root, absPath),
    })
    navigateToPrompt(created.id)
    await promptsStore.loadPrompt(created.id)
  } catch (err) {
    followError.value = err instanceof Error ? err.message : 'Could not open that file.'
  }
}

async function setScope(scope: 'global' | 'project') {
  if (scope === 'project') {
    const projectId = routeProjectId.value || workspaceStore.activeProjectId
    if (!projectId) return
    await promptsStore.setScope('project', projectId)
    router.push({ name: 'project-prompts', params: { id: projectId } })
  } else {
    await promptsStore.setScope('global', null)
    router.push({ name: 'prompts' })
  }
}

async function createPrompt() {
  try {
    creating.value = true
    const created = await promptsStore.createPrompt({
      title: 'New Prompt',
      folder: promptsStore.activeFolder,
      tags: [],
      description: '',
      content: ''
    })
    navigateToPrompt(created.id)
    await promptsStore.loadPrompt(created.id)
  } finally {
    creating.value = false
  }
}

async function duplicatePrompt() {
  if (!selectedPrompt.value) return
  const p = selectedPrompt.value
  const base = {
    title: `${p.title} (Copy)`,
    folder: p.folder,
    tags: p.tags,
    description: p.description
  }
  const created =
    p.kind === 'link'
      ? await promptsStore.createPrompt({
          ...base,
          kind: 'link',
          source: p.source,
          template: p.template
        })
      : await promptsStore.createPrompt({
          ...base,
          content: p.content
        })
  navigateToPrompt(created.id)
  await promptsStore.loadPrompt(created.id)
}

async function savePrompt() {
  if (!selectedPrompt.value) return
  try {
    saveStatus.value = 'saving'
    const meta = {
      title: draftTitle.value.trim() || 'Untitled Prompt',
      folder: draftFolder.value.trim() || 'root',
      description: draftDescription.value.trim(),
      tags: parseTags(draftTags.value)
    }
    await promptsStore.updatePrompt(
      selectedPrompt.value.id,
      selectedPrompt.value.kind === 'link' ? meta : { ...meta, content: draftContent.value }
    )
    saveStatus.value = 'saved'
    setTimeout(() => {
      if (saveStatus.value === 'saved') saveStatus.value = 'idle'
    }, 1200)
  } catch {
    saveStatus.value = 'error'
  }
}

async function deletePrompt() {
  if (!selectedPrompt.value) return
  const p = selectedPrompt.value
  const message =
    p.kind === 'link'
      ? `Delete the link "${p.title}"? The file on disk stays.`
      : `Delete prompt "${p.title}"?`
  if (!confirm(message)) return
  const id = selectedPrompt.value.id
  await promptsStore.deletePrompt(id)
  navigateToPrompt()
}

async function onLinkTemplateToggle(enabled: boolean) {
  const item = selectedPrompt.value
  if (!item || item.kind !== 'link' || item.template === enabled) return
  await promptsStore.updatePrompt(item.id, { template: enabled })
}

async function copyPrompt() {
  if (!selectedPrompt.value) return
  const text =
    selectedPrompt.value.kind === 'link' && !selectedPrompt.value.template
      ? draftContent.value
      : selectedPrompt.value.content
  try {
    await navigator.clipboard.writeText(text)
    copyStatus.value = 'raw-copied'
    setTimeout(() => {
      if (copyStatus.value === 'raw-copied') copyStatus.value = 'idle'
    }, 1200)
  } catch {
    copyStatus.value = 'error'
  }
}

function toggleBundleSelect(id: string) {
  const next = new Set(selectedIds.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  selectedIds.value = next
}

async function copyBundle() {
  const ordered = prompts.value.filter((item) => selectedIds.value.has(item.id))
  const ids =
    ordered.length > 0
      ? ordered.map((item) => item.id)
      : selectedPrompt.value
        ? [selectedPrompt.value.id]
        : []
  if (ids.length === 0) return

  const projectId = promptsStore.activeProjectId
  const root =
    promptsStore.activeScope === 'project' && projectId
      ? (getLibraryRoot(projectId) ?? undefined)
      : undefined

  const sections: { title: string; path: string; text: string }[] = []
  for (const id of ids) {
    try {
      const full = await promptsApi.get(id, root)
      if (full.kind === 'link') {
        sections.push({
          title: full.title,
          path: full.resolved_path || full.source || '',
          text: full.source_missing ? '(file missing)' : full.content,
        })
      } else {
        sections.push({
          title: full.title,
          path: full.path || `data/prompts/${full.id}.md`,
          text: full.content,
        })
      }
    } catch {
      sections.push({ title: id, path: '', text: '(file missing)' })
    }
  }

  try {
    await navigator.clipboard.writeText(buildBundle(sections))
  } catch {
    copyStatus.value = 'error'
  }
}

async function copyRenderedPrompt() {
  if (!selectedPrompt.value) return
  try {
    await navigator.clipboard.writeText(renderedPrompt.value)
    copyStatus.value = 'rendered-copied'
    setTimeout(() => {
      if (copyStatus.value === 'rendered-copied') copyStatus.value = 'idle'
    }, 1200)
  } catch {
    copyStatus.value = 'error'
  }
}

async function loadHistoryForPrompt() {
  const prompt = selectedPrompt.value
  if (!prompt) {
    fileHistory.value = []
    selectedCommitId.value = null
    selectedCommitDiff.value = null
    return
  }
  if (prompt.kind === 'link') {
    fileHistory.value = []
    selectedCommitId.value = null
    selectedCommitDiff.value = null
    return
  }
  try {
    historyLoading.value = true
    fileHistory.value = await gitApi.logByPath(prompt.path, 30)
    selectedCommitId.value = null
    selectedCommitDiff.value = null
  } finally {
    historyLoading.value = false
  }
}

async function openCommit(commit: CommitDetail) {
  if (selectedCommitId.value === commit.id) {
    selectedCommitId.value = null
    selectedCommitDiff.value = null
    return
  }
  try {
    commitDiffLoading.value = true
    selectedCommitId.value = commit.id
    const fullDiff = await gitApi.commitDiff(commit.id)
    const promptPath = selectedPrompt.value?.path
    selectedCommitDiff.value = {
      ...fullDiff,
      files: fullDiff.files.filter((f) => f.path === promptPath)
    }
  } finally {
    commitDiffLoading.value = false
  }
}

function scheduleSearch() {
  if (searchDebounce) clearTimeout(searchDebounce)
  searchDebounce = window.setTimeout(async () => {
    if (promptsStore.semanticMode) {
      await promptsStore.runSemanticSearch()
    }
  }, 200)
}

watch(
  () => promptsStore.query,
  () => {
    scheduleSearch()
  }
)

watch(
  () => promptsStore.semanticMode,
  async (enabled) => {
    if (!enabled) {
      promptsStore.semanticResults = []
      return
    }
    await promptsStore.runSemanticSearch()
  }
)

watch(
  () =>
    [
      selectedPrompt.value?.id,
      selectedPrompt.value?.content,
      selectedPrompt.value?.root_required,
      selectedPrompt.value?.source_missing,
      selectedPrompt.value?.source_error,
    ] as const,
  () => {
    followError.value = ''
    syncDraftFromSelected()
    applyReadingMode()
    loadHistoryForPrompt()
  }
)

watch(
  () => promptVariables.value,
  (variables) => {
    const nextValues: Record<string, string> = {}
    for (const key of variables) {
      nextValues[key] = variableValues.value[key] ?? ''
    }
    variableValues.value = nextValues
  },
  { immediate: true }
)

watch(
  () => routePromptId.value,
  async (id) => {
    if (id) {
      await promptsStore.loadPrompt(id)
    } else {
      promptsStore.selectedPrompt = null
      syncDraftFromSelected()
    }
  }
)

watch(
  () => routeProjectId.value,
  async (projectId) => {
    if (projectId) {
      await promptsStore.setScope('project', projectId)
    } else {
      await promptsStore.setScope('global', null)
    }
    if (routePromptId.value) {
      await promptsStore.loadPrompt(routePromptId.value)
    }
  }
)

onMounted(async () => {
  promptsStore.registerFocusRefresh()
  if (routeProjectId.value) {
    await promptsStore.setScope('project', routeProjectId.value)
  } else {
    await promptsStore.setScope('global', null)
  }
  if (routePromptId.value) {
    await promptsStore.loadPrompt(routePromptId.value)
  }
  syncDraftFromSelected()
  applyReadingMode()
})

onUnmounted(() => {
  promptsStore.unregisterFocusRefresh()
})
</script>

<template>
  <div class="prompts-view">
    <aside class="library-panel">
      <div class="library-header">
        <h3>Library</h3>
        <div class="library-header-actions">
          <button class="small" type="button" data-testid="copy-bundle" @click="copyBundle">Copy bundle</button>
          <button class="small" type="button" data-testid="add-link-open" @click="addLinkOpen = true">Add link</button>
          <button class="primary small" :disabled="creating" @click="createPrompt">+ New</button>
        </div>
      </div>
      <AddLinkDialog
        :open="addLinkOpen"
        :mode="promptsStore.activeScope"
        :project-id="promptsStore.activeProjectId"
        @close="addLinkOpen = false"
      />

      <div class="scope-tabs">
        <button :class="{ active: promptsStore.activeScope === 'global' }" @click="setScope('global')">Global</button>
        <button
          :class="{ active: promptsStore.activeScope === 'project' }"
          :disabled="!projectScopeAllowed && !isProjectScope"
          @click="setScope('project')"
        >
          Project
        </button>
      </div>

      <div class="search-row">
        <input v-model="promptsStore.query" type="search" placeholder="Semantic search prompts..." />
        <label class="semantic-toggle">
          <input v-model="promptsStore.semanticMode" type="checkbox" />
          semantic
        </label>
      </div>

      <div class="filters">
        <div class="filters-header">
          <span>Filters</span>
          <button class="small" @click="filtersCollapsed = !filtersCollapsed">
            {{ filtersCollapsed ? 'Show' : 'Hide' }}
          </button>
        </div>
        <template v-if="!filtersCollapsed">
          <div class="filter-group">
            <span class="filter-label">Folders</span>
            <button
              :class="['chip', { active: promptsStore.activeFolder === 'root' }]"
              @click="promptsStore.activeFolder = 'root'"
            >
              root
            </button>
            <button
              v-for="folder in promptsStore.folders"
              :key="folder.path"
              :class="['chip', { active: promptsStore.activeFolder === folder.path }]"
              @click="promptsStore.activeFolder = folder.path"
            >
              {{ folder.path }} ({{ folder.count }})
            </button>
          </div>

          <div class="filter-group" v-if="promptsStore.allTags.length > 0">
            <span class="filter-label">Tags</span>
            <button :class="['chip', { active: !promptsStore.activeTag }]" @click="promptsStore.activeTag = null">
              all
            </button>
            <button
              v-for="tag in promptsStore.allTags"
              :key="tag"
              :class="['chip', { active: promptsStore.activeTag === tag }]"
              @click="promptsStore.activeTag = promptsStore.activeTag === tag ? null : tag"
            >
              {{ tag }}
            </button>
          </div>
        </template>
      </div>

      <LibraryRootBar
        v-if="isProjectScope && routeProjectId"
        :project-id="routeProjectId"
        @root-changed="libraryRootRefresh++"
      />
      <SuggestFilesPanel
        v-if="isProjectScope && routeProjectId"
        :project-id="routeProjectId"
        :refresh-token="libraryRootRefresh"
      />

      <div class="prompt-list">
        <div v-if="promptsStore.loading" class="empty">Loading prompts...</div>
        <div v-else-if="prompts.length === 0" class="empty">No prompts found</div>
        <button
          v-for="prompt in prompts"
          :key="prompt.id"
          :class="['prompt-item', { active: selectedPrompt?.id === prompt.id }]"
          @click="selectPrompt(prompt)"
        >
          <div class="prompt-title-row">
            <input
              type="checkbox"
              class="bundle-select"
              :data-testid="`bundle-select-${prompt.id}`"
              :checked="selectedIds.has(prompt.id)"
              :aria-label="`Select ${prompt.title}`"
              @click.stop
              @change="toggleBundleSelect(prompt.id)"
            />
            <div class="prompt-title">{{ prompt.title }}</div>
          </div>
          <div class="prompt-meta">
            <span class="kind-badge">{{ prompt.kind === 'link' ? 'Link' : 'Prompt' }}</span>
            <span class="scope-badge">{{ prompt.scope }}</span>
            <span class="folder">{{ prompt.folder }}</span>
            <span v-if="semanticScoreByPromptId.has(prompt.id)" class="score">
              {{ (semanticScoreByPromptId.get(prompt.id)! * 100).toFixed(0) }}%
            </span>
          </div>
        </button>
      </div>
    </aside>

    <section class="editor-panel">
      <template v-if="selectedPrompt">
        <div class="editor-header">
          <div class="header-row">
            <input v-model="draftTitle" type="text" class="title-input" placeholder="Prompt title" />
            <div class="actions">
              <button @click="copyPrompt">Copy</button>
              <button
                v-if="templateVariablesEnabled && promptVariables.length > 0"
                @click="copyRenderedPrompt"
              >
                Copy Rendered
              </button>
              <button @click="duplicatePrompt">
                {{ selectedPrompt.kind === 'link' ? 'Duplicate link' : 'Duplicate' }}
              </button>
              <button class="danger" @click="deletePrompt">Delete</button>
              <button class="primary" @click="savePrompt">Save</button>
            </div>
          </div>
          <div class="sub-row">
            <input v-model="draftFolder" type="text" placeholder="Folder (e.g. coding/review)" />
            <input v-model="draftTags" type="text" placeholder="Tags (comma separated)" />
            <input v-model="draftDescription" type="text" placeholder="Short description" />
            <label v-if="selectedPrompt.kind === 'link'" class="template-toggle">
              <input
                type="checkbox"
                data-testid="link-template"
                :checked="selectedPrompt.template"
                @change="onLinkTemplateToggle(($event.target as HTMLInputElement).checked)"
              />
              Template
            </label>
            <span class="status">{{ saveStatus }}</span>
            <span class="status">{{ copyStatus === 'raw-copied' ? 'raw copied' : '' }}</span>
            <span class="status">{{ copyStatus === 'rendered-copied' ? 'rendered copied' : '' }}</span>
          </div>
          <div v-if="templateVariablesEnabled && promptVariables.length > 0" class="variables-row">
            <span class="variables-title">Variables</span>
            <div class="variable-fields">
              <label v-for="varName in promptVariables" :key="varName" class="variable-field">
                <span>{{ varName }}</span>
                <input
                  v-model="variableValues[varName]"
                  type="text"
                  :placeholder="`Value for ${varName}`"
                />
              </label>
            </div>
          </div>
        </div>

        <div class="reading-mode" data-testid="reading-mode" role="group" aria-label="Reading mode">
          <button
            type="button"
            data-testid="reading-mode-raw"
            :class="{ active: readingMode === 'raw' }"
            @click="chooseReadingMode('raw')"
          >
            Raw
          </button>
          <button
            v-if="renderedAvailable"
            type="button"
            data-testid="reading-mode-rendered"
            :class="{ active: readingMode === 'rendered' }"
            @click="chooseReadingMode('rendered')"
          >
            Rendered
          </button>
        </div>

        <div class="versions-header">
          <span>Versions (Git)</span>
          <button
            type="button"
            class="small"
            data-testid="versions-toggle"
            @click="toggleVersionsExpanded"
          >
            {{ versionsExpanded ? 'Hide' : 'Show' }}
          </button>
        </div>

        <p v-if="followError" class="follow-error" data-testid="library-follow-error">
          {{ followError }}
        </p>
        <div
          class="content-grid"
          :class="{ 'content-grid--full-width': !versionsExpanded }"
          data-testid="content-grid"
        >
          <RenderedMarkdown
            v-if="readingMode === 'rendered' && renderedAvailable && !linkBlocked"
            :source="renderedSource"
            :base-path="markdownBasePath"
            :link-root="markdownLinkRoot"
            @open-file="openLinkedFile"
          />
          <LinkPane v-else-if="selectedPrompt.kind === 'link'" :prompt="selectedPrompt" />
          <textarea
            v-else
            v-model="draftContent"
            class="content-editor"
            data-testid="raw-editor"
            spellcheck="false"
          ></textarea>
          <div v-if="versionsExpanded" class="history-panel" data-testid="versions-panel">
            <div v-if="templateVariablesEnabled && promptVariables.length > 0" class="rendered-preview">
              <div class="history-title">Rendered Preview</div>
              <pre>{{ renderedPrompt }}</pre>
            </div>
            <template v-if="selectedPrompt.kind === 'link'">
              <p class="link-outside-repo" data-testid="link-versions-notice">
                This file is outside Ironpad's repository.
              </p>
            </template>
            <template v-else>
              <div v-if="historyLoading" class="empty">Loading history...</div>
              <div v-else-if="fileHistory.length === 0" class="empty">No commits for this prompt yet</div>
              <button
                v-for="commit in fileHistory"
                :key="commit.id"
                :class="['history-item', { active: selectedCommitId === commit.id }]"
                @click="openCommit(commit)"
              >
                <span>{{ commit.short_id }}</span>
                <span>{{ new Date(commit.timestamp).toLocaleDateString() }}</span>
                <span class="message">{{ commit.message }}</span>
              </button>
              <div v-if="commitDiffLoading" class="empty">Loading diff...</div>
              <div v-if="selectedCommitDiff?.files?.length" class="diff-block">
                <div v-for="file in selectedCommitDiff.files" :key="file.path" class="diff-file">
                  <div class="diff-path">{{ file.path }}</div>
                  <pre v-for="(hunk, idx) in file.hunks.slice(0, 3)" :key="idx" class="diff-hunk">{{ hunk.header }}</pre>
                </div>
              </div>
            </template>
          </div>
        </div>
      </template>

      <div v-else class="empty-editor">
        <h3>Select a prompt</h3>
        <p>Choose a prompt from the library or create a new one.</p>
        <p v-if="promptsStore.error" class="load-error" data-testid="prompt-load-error">
          {{ promptsStore.error }}
        </p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.prompts-view {
  flex: 1;
  display: flex;
  min-width: 0;
  overflow: hidden;
}

.library-panel {
  width: 340px;
  border-right: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  display: flex;
  flex-direction: column;
}

.library-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--color-border);
}

.library-header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.scope-tabs,
.search-row,
.filters {
  padding: 10px 12px;
  border-bottom: 1px solid var(--color-border);
}

.scope-tabs {
  display: flex;
  gap: 8px;
}

.scope-tabs button {
  flex: 1;
}

.scope-tabs button.active {
  background: var(--color-primary);
  color: #fff;
  border-color: var(--color-primary);
}

.search-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.search-row input {
  flex: 1;
}

.semantic-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.filter-group {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}

.filters-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.filter-label {
  width: 100%;
  font-size: 11px;
  text-transform: uppercase;
  color: var(--color-text-secondary);
}

.chip {
  font-size: 11px;
  padding: 3px 8px;
  border-radius: 10px;
}

.chip.active {
  background: var(--color-primary);
  color: #fff;
  border-color: var(--color-primary);
}

.prompt-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.prompt-item {
  width: 100%;
  text-align: left;
  padding: 10px;
  margin-bottom: 6px;
}

.prompt-item.active {
  border-color: var(--color-primary);
}

.prompt-title-row {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.bundle-select {
  margin-top: 3px;
  flex-shrink: 0;
}

.prompt-title {
  font-weight: 600;
  margin-bottom: 4px;
}

.prompt-meta {
  display: flex;
  gap: 6px;
  font-size: 11px;
  color: var(--color-text-secondary);
}

.kind-badge {
  font-weight: 600;
  text-transform: capitalize;
}

.scope-badge {
  text-transform: uppercase;
}

.score {
  color: var(--color-primary);
}

.editor-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.editor-header {
  border-bottom: 1px solid var(--color-border);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.header-row,
.sub-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.title-input {
  flex: 1;
}

.actions {
  display: flex;
  gap: 8px;
}

.sub-row input {
  flex: 1;
  min-width: 0;
}

.template-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  white-space: nowrap;
  flex: 0 0 auto;
}

.status {
  font-size: 12px;
  color: var(--color-text-secondary);
}

.variables-row {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  border-top: 1px solid var(--color-border);
  padding-top: 8px;
}

.variables-title {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--color-text-secondary);
  min-width: 62px;
  padding-top: 6px;
}

.variable-fields {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  flex: 1;
}

.variable-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 180px;
  flex: 1;
}

.variable-field span {
  font-size: 11px;
  color: var(--color-text-secondary);
}

input[type="text"],
input[type="search"],
textarea {
  background: var(--color-bg);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

input[type="text"]:focus,
input[type="search"]:focus,
textarea:focus {
  outline: none;
  border-color: var(--color-primary);
}

.reading-mode {
  display: flex;
  gap: 0;
  padding: 8px 12px 0;
}

.reading-mode button {
  background: var(--color-bg);
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
  padding: 4px 12px;
  font-size: 12px;
  cursor: pointer;
}

.reading-mode button + button {
  border-left: none;
}

.reading-mode button.active {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.versions-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px 8px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.content-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 2fr 1fr;
  min-height: 0;
}

.content-grid--full-width {
  grid-template-columns: 1fr;
}

.content-grid :deep(.rendered-markdown) {
  min-height: 0;
  height: 100%;
  padding: 16px;
}

.content-editor {
  width: 100%;
  height: 100%;
  border: none;
  outline: none;
  padding: 16px;
  background: var(--color-bg);
  color: var(--color-text);
  resize: none;
  font-family: var(--font-mono);
}

.history-panel {
  border-left: 1px solid var(--color-border);
  padding: 10px;
  overflow-y: auto;
  background: var(--color-bg-secondary);
}

.history-title {
  font-size: 12px;
  text-transform: uppercase;
  color: var(--color-text-secondary);
  margin-bottom: 8px;
}

.link-outside-repo {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.rendered-preview {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  padding: 8px;
  margin-bottom: 10px;
}

.rendered-preview pre {
  max-height: 180px;
  overflow: auto;
  margin: 0;
  white-space: pre-wrap;
  font-size: 12px;
  color: var(--color-text);
}

.history-item {
  width: 100%;
  text-align: left;
  margin-bottom: 6px;
  font-size: 12px;
  display: flex;
  gap: 8px;
  align-items: center;
}

.history-item .message {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-item.active {
  border-color: var(--color-primary);
}

.diff-block {
  margin-top: 8px;
  border-top: 1px solid var(--color-border);
  padding-top: 8px;
}

.diff-path {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-bottom: 4px;
}

.diff-hunk {
  font-size: 11px;
  padding: 4px 6px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  margin-bottom: 4px;
}

.empty,
.empty-editor {
  color: var(--color-text-secondary);
  font-size: 13px;
}

.empty {
  padding: 10px;
}

.empty-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.load-error {
  max-width: 420px;
  text-align: center;
  color: var(--color-danger, #c44);
}

.follow-error {
  margin: 8px 0 0;
  color: var(--color-danger, #c44);
  font-size: 12px;
}

button.small {
  padding: 4px 8px;
}
</style>
