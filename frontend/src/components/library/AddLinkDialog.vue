<script setup lang="ts">
import { ref, watch } from 'vue'
import { libraryApi } from '../../api/client'
import { fileTitle, suffixOfPath, LINK_SUFFIXES } from '../../lib/libraryFiles'
import {
  getLibraryRoot,
  normalizeAbsolutePath,
  sanitizeUserPath,
} from '../../lib/libraryRoot'
import { sourceForLibraryFile } from '../../lib/markdownTarget'
import { usePromptsStore } from '../../stores/prompts'
import type { BrowseEntry, BrowseResponse } from '../../types'

const MAX_LINK_BYTES = 1_048_576

const props = defineProps<{
  open: boolean
  mode: 'global' | 'project'
  projectId: string | null
}>()

const emit = defineEmits<{
  close: []
}>()

const promptsStore = usePromptsStore()

const listing = ref<BrowseResponse | null>(null)
const loading = ref(false)
const error = ref('')
const selected = ref<Set<string>>(new Set())
const confirming = ref(false)
const globalPath = ref('')
const pastePath = ref('')
const needsRoot = ref(false)
const projectRoot = ref<string | null>(null)
const pasting = ref(false)
const picking = ref(false)

const MSG_GLOBAL_ABSOLUTE = 'Enter an absolute path.'
const MSG_PROJECT_PARENT = "Project link source must not contain '..'."
const MSG_PROJECT_RELATIVE = 'Enter an absolute path, or set a folder root.'

function disabledReason(entry: BrowseEntry): string {
  if (entry.is_dir) return 'directory'
  const suffix = suffixOfPath(entry.name)
  if (!suffix || !LINK_SUFFIXES.has(suffix)) return 'not allowed'
  if (entry.size > MAX_LINK_BYTES) return 'too large'
  return 'not allowed'
}

async function load(path: string) {
  const trimmed = sanitizeUserPath(path)
  if (!trimmed) return
  loading.value = true
  error.value = ''
  selected.value = new Set()
  try {
    listing.value = await libraryApi.browseLibrary(trimmed)
  } catch (err) {
    listing.value = null
    error.value = err instanceof Error ? err.message : 'Could not list that directory.'
  } finally {
    loading.value = false
  }
}

function slashNormalize(path: string): string {
  return path.replace(/\\/g, '/')
}

function isAbsoluteFilePath(path: string): boolean {
  const s = slashNormalize(sanitizeUserPath(path))
  return /^[A-Za-z]:/.test(s) || s.startsWith('/') || s.startsWith('//')
}

function hasParentSegment(path: string): boolean {
  return slashNormalize(sanitizeUserPath(path))
    .split('/')
    .some((segment) => segment === '..')
}

function storedSource(absPath: string): string {
  if (props.mode === 'project') {
    return sourceForLibraryFile(projectRoot.value, absPath)
  }
  return normalizeAbsolutePath(absPath)
}

function resetAndOpen() {
  listing.value = null
  error.value = ''
  selected.value = new Set()
  confirming.value = false
  pasting.value = false
  picking.value = false
  pastePath.value = ''
  needsRoot.value = false
  projectRoot.value = null
  if (props.mode === 'project') {
    const root = props.projectId ? getLibraryRoot(props.projectId) : null
    projectRoot.value = root
    if (!root) {
      needsRoot.value = true
      return
    }
    void load(root)
    return
  }
  globalPath.value = ''
}

watch(
  () => [props.open, props.mode, props.projectId] as const,
  ([open]) => {
    if (open) resetAndOpen()
  },
  { immediate: true }
)

function toggleFile(entry: BrowseEntry) {
  if (!entry.linkable) return
  const next = new Set(selected.value)
  if (next.has(entry.path)) next.delete(entry.path)
  else next.add(entry.path)
  selected.value = next
}

function activeFolderPayload(): string | undefined {
  const folder = promptsStore.activeFolder
  if (!folder || folder === 'root') return undefined
  return folder
}

async function confirm() {
  if (!listing.value || selected.value.size === 0) return
  const chosen = listing.value.entries.filter((entry) => selected.value.has(entry.path))
  confirming.value = true
  error.value = ''
  try {
    for (const entry of chosen) {
      const source = storedSource(entry.path)
      await promptsStore.createPrompt({
        kind: 'link',
        title: entry.name,
        folder: activeFolderPayload(),
        tags: [],
        template: false,
        source,
      })
    }
    emit('close')
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Could not create the link.'
  } finally {
    confirming.value = false
  }
}

function resolvePastedSource(raw: string): string | null {
  const trimmed = raw.trim()
  if (!trimmed) return null
  if (props.mode === 'project') {
    if (isAbsoluteFilePath(trimmed)) {
      return sourceForLibraryFile(projectRoot.value, trimmed)
    }
    if (!projectRoot.value) {
      error.value = MSG_PROJECT_RELATIVE
      return null
    }
    if (hasParentSegment(trimmed)) {
      error.value = MSG_PROJECT_PARENT
      return null
    }
    return slashNormalize(sanitizeUserPath(trimmed)).replace(/^\/+/, '')
  }
  if (!isAbsoluteFilePath(trimmed)) {
    error.value = MSG_GLOBAL_ABSOLUTE
    return null
  }
  return normalizeAbsolutePath(trimmed)
}

async function createFromPaths(paths: string[]) {
  error.value = ''
  if (paths.length === 0) return
  picking.value = true
  try {
    for (const path of paths) {
      await promptsStore.createPrompt({
        kind: 'link',
        title: fileTitle(path),
        folder: activeFolderPayload(),
        tags: [],
        template: false,
        source: storedSource(path),
      })
    }
    emit('close')
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Could not create the link.'
  } finally {
    picking.value = false
  }
}

async function browseFiles() {
  if (picking.value) return
  error.value = ''
  picking.value = true
  try {
    const directory =
      props.mode === 'project' ? (projectRoot.value ?? undefined) : listing.value?.path
    const result = await libraryApi.pickLibrary('file', directory)
    picking.value = false
    await createFromPaths(result.paths)
  } catch (err) {
    picking.value = false
    error.value = err instanceof Error ? err.message : 'Could not open the file dialog.'
  }
}

async function browseFolder() {
  if (picking.value) return
  error.value = ''
  picking.value = true
  try {
    const result = await libraryApi.pickLibrary('directory')
    const path = result.paths[0]
    picking.value = false
    if (!path) return
    globalPath.value = path
    await load(path)
  } catch (err) {
    picking.value = false
    error.value = err instanceof Error ? err.message : 'Could not open the folder dialog.'
  }
}

async function addPastedPath() {
  error.value = ''
  const source = resolvePastedSource(pastePath.value)
  if (source === null) {
    if (!pastePath.value.trim()) return
    return
  }
  pasting.value = true
  try {
    await promptsStore.createPrompt({
      kind: 'link',
        title: fileTitle(pastePath.value),
      folder: activeFolderPayload(),
      tags: [],
      template: false,
      source,
    })
    emit('close')
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Could not create the link.'
  } finally {
    pasting.value = false
  }
}
</script>

<template>
  <div v-if="open" class="add-link-dialog" role="dialog" aria-label="Add link" data-testid="add-link-dialog">
    <div class="dialog-card">
      <header class="dialog-header">
        <h3>Add link</h3>
        <button type="button" class="small" data-testid="add-link-close" @click="emit('close')">Close</button>
      </header>

      <div class="pick-actions">
        <button
          type="button"
          class="small"
          data-testid="add-link-browse-file"
          :disabled="picking || pasting || confirming"
          @click="browseFiles"
        >
          {{ picking ? 'Waiting for the file dialog…' : 'Browse for file' }}
        </button>
      </div>

      <p v-if="needsRoot" class="root-prompt" data-testid="add-link-root-prompt">
        Set a folder root for this project to browse inside the repo, or pick a file anywhere on disk.
      </p>

      <div v-else-if="mode === 'global' && !listing" class="global-go">
        <input
          v-model="globalPath"
          type="text"
          placeholder="Absolute directory path"
          data-testid="add-link-path"
          @keydown.enter.prevent="load(globalPath)"
        />
        <button type="button" class="small" data-testid="add-link-go" @click="load(globalPath)">Go</button>
        <button
          type="button"
          class="small"
          data-testid="add-link-browse-folder"
          :disabled="picking"
          @click="browseFolder"
        >
          Browse
        </button>
      </div>

      <div v-else class="listing">
        <p v-if="loading" class="status">Loading…</p>
        <template v-else-if="listing">
          <p class="current-path" data-testid="add-link-current">{{ listing.path }}</p>
          <button
            v-if="listing.parent"
            type="button"
            class="entry dir"
            data-testid="browse-up"
            @click="load(listing.parent)"
          >
            Up
          </button>
          <ul class="entries">
            <li v-for="entry in listing.entries" :key="entry.path">
              <button
                v-if="entry.is_dir"
                type="button"
                class="entry dir"
                data-testid="browse-dir"
                @click="load(entry.path)"
              >
                {{ entry.name }}
              </button>
              <label v-else class="entry file" :data-testid="entry.linkable ? 'browse-file' : 'browse-file-disabled'">
                <input
                  type="checkbox"
                  :disabled="!entry.linkable"
                  :checked="selected.has(entry.path)"
                  @change="toggleFile(entry)"
                />
                <span>{{ entry.name }}</span>
                <span v-if="!entry.linkable" class="reason">{{ disabledReason(entry) }}</span>
              </label>
            </li>
          </ul>
          <button
            type="button"
            class="primary small"
            data-testid="add-link-confirm"
            :disabled="selected.size === 0 || confirming"
            @click="confirm"
          >
            Confirm
          </button>
        </template>
      </div>

      <div class="paste-path">
        <input
          v-model="pastePath"
          type="text"
          placeholder="Paste path"
          data-testid="add-link-paste-path"
          @keydown.enter.prevent="addPastedPath"
        />
        <button
          type="button"
          class="small"
          data-testid="add-link-paste-add"
          :disabled="!pastePath.trim() || pasting"
          @click="addPastedPath"
        >
          Add
        </button>
      </div>

      <p v-if="error" class="error" data-testid="add-link-error">{{ error }}</p>
    </div>
  </div>
</template>

<style scoped>
.add-link-dialog {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 20;
}

.dialog-card {
  width: min(480px, calc(100vw - 32px));
  max-height: calc(100vh - 48px);
  overflow: auto;
  background: var(--color-bg-elevated, var(--color-bg));
  color: var(--color-text);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dialog-header h3 {
  margin: 0;
  font-size: 14px;
}

.pick-actions,
.global-go,
.paste-path {
  display: flex;
  gap: 8px;
}

.global-go input,
.paste-path input {
  flex: 1;
  min-width: 0;
  background: var(--color-bg);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.entries {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.entry {
  display: flex;
  gap: 8px;
  align-items: center;
  text-align: left;
  width: 100%;
}

button.entry {
  background: transparent;
  color: var(--color-text);
  border: 1px solid var(--color-border);
  padding: 6px 8px;
  cursor: pointer;
}

.reason {
  color: var(--color-text-secondary);
  font-size: 12px;
}

.current-path,
.status,
.root-prompt {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary);
  word-break: break-all;
}

.error {
  margin: 0;
  color: var(--color-danger, #c44);
  font-size: 12px;
}
</style>
