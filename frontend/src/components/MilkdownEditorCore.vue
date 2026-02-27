<script setup lang="ts">
import { watch, computed, ref, onUnmounted, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { Milkdown, useEditor } from '@milkdown/vue'
import { Crepe, CrepeFeature } from '@milkdown/crepe'
import { listener, listenerCtx } from '@milkdown/kit/plugin/listener'
import { replaceAll } from '@milkdown/kit/utils'
import { useThemeStore } from '../stores'
import { setupLinkClickHandler } from './milkdown-link-handler'
import LinkAutocomplete from './LinkAutocomplete.vue'
import type { NoteTitleEntry } from '../types'

// Import Crepe common styles (layout, components)
import '@milkdown/crepe/theme/common/style.css'

// Import the frame theme (light) - we override for dark mode via CSS
import '@milkdown/crepe/theme/frame.css'

const props = defineProps<{
  modelValue: string
  readonly?: boolean
  projectId?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  'editor-ready': [editor: Crepe]
  'force-remount': []
}>()

// Raw (non-reactive) reference to the Crepe instance.
// Vue's reactive proxy breaks access to Crepe's private fields via getters,
// so we keep an unwrapped reference for programmatic editor updates.
let crepeRaw: Crepe | null = null

// Reference to the editor container for click handling
const editorContainer = ref<HTMLElement | null>(null)
const linkAutocompleteRef = ref<HTMLElement | null>(null)
let linkClickCleanup: (() => void) | null = null

const themeStore = useThemeStore()
const router = useRouter()

// Slash command state
const showLinkAutocomplete = ref(false)
const linkAutocompletePosition = ref({ x: 0, y: 0 })

// Use mode directly for reactivity, then compute effective theme
const isDarkMode = computed(() => {
  const mode = themeStore.mode
  if (mode === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches
  }
  return mode === 'dark'
})

// CRITICAL: Use refs for instance-scoped state that must reset when component recreates
// These were previously module-level lets which caused stale content bugs when switching notes/tasks
const isExternalUpdate = ref(false)
// Store the RAW content (with wikilinks) - this represents what the parent component expects
const currentContent = ref(props.modelValue)
const pendingContent = ref<string | null>(null)
const editorReady = ref(false)

// Cleanup any pending timeouts/intervals on unmount
let externalUpdateTimeout: ReturnType<typeof setTimeout> | null = null
let pendingRetryInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  // Setup link click handler when editor is mounted
  if (editorContainer.value) {
    linkClickCleanup = setupLinkClickHandler(editorContainer.value, router)
  }
})

onUnmounted(() => {
  crepeRaw = null
  if (externalUpdateTimeout) {
    clearTimeout(externalUpdateTimeout)
    externalUpdateTimeout = null
  }
  if (pendingRetryInterval) {
    clearInterval(pendingRetryInterval)
    pendingRetryInterval = null
  }
  if (linkClickCleanup) {
    linkClickCleanup()
    linkClickCleanup = null
  }
})

// Try to apply pending content - called when editor might be ready
function tryApplyPendingContent() {
  if (pendingContent.value === null) return false
  if (!crepeRaw) return false
  
  try {
    const editor = crepeRaw.editor
    if (!editor || typeof editor.action !== 'function') return false
    
    const processedContent = pendingContent.value
    
    isExternalUpdate.value = true
    editor.action(replaceAll(processedContent))
    currentContent.value = pendingContent.value
    pendingContent.value = null
    editorReady.value = true
    
    if (externalUpdateTimeout) clearTimeout(externalUpdateTimeout)
    externalUpdateTimeout = setTimeout(() => { isExternalUpdate.value = false }, 50)
    
    if (pendingRetryInterval) {
      clearInterval(pendingRetryInterval)
      pendingRetryInterval = null
    }
    return true
  } catch (err) {
    console.warn('[MilkdownEditorCore] Failed to apply pending content:', err)
    return false
  }
}

const { get, loading } = useEditor((root) => {
  const initialContent = props.modelValue
  
  const crepe = new Crepe({
    root,
    defaultValue: initialContent,
    features: {
      [CrepeFeature.CodeMirror]: true,
      [CrepeFeature.ListItem]: true,
      [CrepeFeature.LinkTooltip]: true,
      [CrepeFeature.Cursor]: true,
      [CrepeFeature.ImageBlock]: true,
      [CrepeFeature.BlockEdit]: true,
      [CrepeFeature.Toolbar]: true,
      [CrepeFeature.Placeholder]: true,
      [CrepeFeature.Table]: true,
      [CrepeFeature.Latex]: false,
    },
    featureConfigs: {
      [CrepeFeature.Placeholder]: {
        text: 'Start writing... Type "/" for commands',
      },
    },
  })

  // Store raw reference before Vue can wrap it in a reactive proxy
  crepeRaw = crepe

  // Add listener plugin for content changes
  const editor = crepe.editor
  editor
    .config((ctx: any) => {
      const listenerHandler = ctx.get(listenerCtx)
      listenerHandler.markdownUpdated((_ctx: any, markdown: string, prevMarkdown: string) => {
        if (markdown !== prevMarkdown && !isExternalUpdate.value && editorReady.value && pendingContent.value === null) {
          currentContent.value = markdown
          emit('update:modelValue', markdown)
          
          nextTick(() => {
            checkForSlashCommand(markdown, prevMarkdown)
          })
        }
      })
    })
    .use(listener)

  return crepe
})

// Check if user typed "/link" to trigger the autocomplete
function checkForSlashCommand(newMarkdown: string, prevMarkdown: string) {
  if (showLinkAutocomplete.value || !props.projectId) return
  
  const hasLinkCmd = newMarkdown.match(/\/link\s*$/)
  const hadLinkCmd = prevMarkdown.match(/\/link\s*$/)
  
  if (!hasLinkCmd) return
  
  if (hadLinkCmd && newMarkdown.length > prevMarkdown.length) {
    const addedText = newMarkdown.substring(prevMarkdown.length)
    if (addedText === ' ' || addedText === '\n') {
      showLinkAutocompleteAtCursor()
    }
    return
  }
  
  if (!hadLinkCmd && hasLinkCmd) {
    showLinkAutocompleteAtCursor()
  }
}

// Show link autocomplete at the current cursor position
function showLinkAutocompleteAtCursor() {
  if (editorContainer.value) {
    const rect = editorContainer.value.getBoundingClientRect()
    linkAutocompletePosition.value = {
      x: rect.left + 20,
      y: rect.top + 100
    }
  }
  
  showLinkAutocomplete.value = true
}

// Handle note selection from autocomplete
function handleNoteSelect(note: NoteTitleEntry) {
  const linkMarkdown = `[${note.title}](${note.id})`
  
  const currentMarkdown = currentContent.value
  const linkIndex = currentMarkdown.lastIndexOf('/link')
  
  if (linkIndex < 0) {
    showLinkAutocomplete.value = false
    return
  }
  
  const newContent = currentMarkdown.substring(0, linkIndex) + 
                     linkMarkdown + 
                     currentMarkdown.substring(linkIndex + 5)
  
  let editorUpdated = false
  
  // Try updating the editor view directly via the raw Crepe reference
  if (crepeRaw) {
    try {
      const editor = crepeRaw.editor
      if (editor && typeof editor.action === 'function') {
        isExternalUpdate.value = true
        editor.action(replaceAll(newContent))
        currentContent.value = newContent
        editorUpdated = true
        
        if (externalUpdateTimeout) clearTimeout(externalUpdateTimeout)
        externalUpdateTimeout = setTimeout(() => { isExternalUpdate.value = false }, 50)
      }
    } catch {
      editorUpdated = false
    }
  }
  
  // Emit the update to parent
  emit('update:modelValue', newContent)
  
  if (!editorUpdated) {
    // Direct update failed - signal parent to force-remount the editor.
    // Don't set currentContent so the modelValue watcher won't skip the update.
    emit('force-remount')
  }
  
  showLinkAutocomplete.value = false
}

// Handle autocomplete cancel
function handleAutocompleteCancel() {
  showLinkAutocomplete.value = false
}

// Emit editor instance when ready, and apply any pending content
watch(loading, (isLoading) => {
  if (!isLoading) {
    if (crepeRaw) {
      emit('editor-ready', crepeRaw)
      
      if (pendingContent.value !== null) {
        if (!tryApplyPendingContent()) {
          startPendingRetry()
        }
      } else {
        editorReady.value = true
      }
    }
  }
}, { immediate: true })

// Start a retry interval for applying pending content
function startPendingRetry() {
  if (pendingRetryInterval) return
  
  let retryCount = 0
  const maxRetries = 20
  
  pendingRetryInterval = setInterval(() => {
    retryCount++
    
    if (tryApplyPendingContent()) return
    
    if (retryCount >= maxRetries) {
      console.warn('[MilkdownEditorCore] Failed to apply pending content after retries')
      if (pendingRetryInterval) {
        clearInterval(pendingRetryInterval)
        pendingRetryInterval = null
      }
    }
  }, 100)
}

// Watch for external content changes
watch(() => props.modelValue, async (newValue) => {
  if (loading.value) {
    pendingContent.value = newValue
    return
  }
  
  if (newValue === currentContent.value) return
  
  pendingContent.value = newValue
  
  if (!tryApplyPendingContent()) {
    startPendingRetry()
  }
})
</script>

<template>
  <div ref="editorContainer" :class="['crepe-editor', { 'dark-theme': isDarkMode }]" style="position: relative;">
    <Milkdown />
    <LinkAutocomplete
      ref="linkAutocompleteRef"
      :project-id="projectId || ''"
      :visible="showLinkAutocomplete"
      :anchor-position="linkAutocompletePosition"
      @select="handleNoteSelect"
      @cancel="handleAutocompleteCancel"
    />
  </div>
</template>

<style>
/* 
 * Dark theme override for Milkdown Crepe
 * When .dark-theme is applied, override Crepe's light theme variables
 */
.crepe-editor.dark-theme .milkdown {
  --crepe-color-background: #1a1a1a;
  --crepe-color-on-background: #e0e0e0;
  --crepe-color-surface: #232323;
  --crepe-color-surface-low: #1a1a1a;
  --crepe-color-on-surface: #e0e0e0;
  --crepe-color-on-surface-variant: #999999;
  --crepe-color-outline: #3c3c3c;
  --crepe-color-primary: #58a6ff;
  --crepe-color-secondary: #232323;
  --crepe-color-on-secondary: #e0e0e0;
  --crepe-color-inverse: #e0e0e0;
  --crepe-color-on-inverse: #1a1a1a;
  --crepe-color-inline-code: #58a6ff;
  --crepe-color-error: #f85149;
  --crepe-color-hover: #2d2d2d;
  --crepe-color-selected: #3c3c3c;
  --crepe-color-inline-area: #2d2d2d;
  --crepe-shadow-1: 0px 1px 2px 0px rgba(0, 0, 0, 0.5), 0px 1px 3px 1px rgba(0, 0, 0, 0.3);
  --crepe-shadow-2: 0px 1px 2px 0px rgba(0, 0, 0, 0.5), 0px 2px 6px 2px rgba(0, 0, 0, 0.3);
}

/* Editor container layout */
.crepe-editor {
  height: 100%;
  display: flex;
  flex-direction: column;
}

/* Dark theme background for container */
.crepe-editor.dark-theme {
  background: #1a1a1a;
  color: #e0e0e0;
}

.crepe-editor .milkdown {
  height: 100%;
  flex: 1;
  display: flex;
  flex-direction: column;
}

/* The actual editor area */
.crepe-editor .ProseMirror {
  flex: 1;
  min-height: 200px;
  outline: none;
  padding: 16px;
}

/* Dark theme for ProseMirror content area */
.crepe-editor.dark-theme .ProseMirror {
  background: #1a1a1a;
  color: #e0e0e0;
}

/* Toolbar styling - dark theme */
.crepe-editor.dark-theme .milkdown-toolbar,
.crepe-editor.dark-theme milkdown-toolbar {
  background: #232323;
  border-bottom: 1px solid #3c3c3c;
}

.crepe-editor.dark-theme .milkdown-toolbar button,
.crepe-editor.dark-theme milkdown-toolbar button {
  color: #e0e0e0;
}

.crepe-editor.dark-theme .milkdown-toolbar button:hover,
.crepe-editor.dark-theme milkdown-toolbar button:hover {
  background: #2d2d2d;
}

/* Block handle and menus - dark theme */
.crepe-editor.dark-theme [data-block-handle],
.crepe-editor.dark-theme .slash-menu,
.crepe-editor.dark-theme .link-tooltip,
.crepe-editor.dark-theme milkdown-slash-menu,
.crepe-editor.dark-theme milkdown-link-tooltip {
  background: #232323;
  border: 1px solid #3c3c3c;
  color: #e0e0e0;
}

/* Menu items - dark theme */
.crepe-editor.dark-theme .slash-menu-item,
.crepe-editor.dark-theme [role="menuitem"] {
  color: #e0e0e0;
}

.crepe-editor.dark-theme .slash-menu-item:hover,
.crepe-editor.dark-theme [role="menuitem"]:hover {
  background: #2d2d2d;
}

/* Code blocks - dark theme */
.crepe-editor.dark-theme pre {
  background: #232323;
  border: 1px solid #3c3c3c;
  border-radius: 6px;
  padding: 12px;
}

.crepe-editor.dark-theme pre code {
  background: transparent;
  color: #e0e0e0;
}

.crepe-editor code {
  font-family: var(--font-mono, 'SF Mono', 'Monaco', 'Menlo', 'Consolas', monospace);
  font-size: 13px;
}

/* Inline code - dark theme */
.crepe-editor.dark-theme :not(pre) > code {
  background: #2d2d2d;
  padding: 2px 6px;
  border-radius: 4px;
  color: #58a6ff;
}

/* Blockquote - dark theme */
.crepe-editor.dark-theme blockquote {
  border-left: 3px solid #3c3c3c;
  padding-left: 16px;
  margin-left: 0;
  color: #999999;
}

/* Tables - dark theme */
.crepe-editor table {
  border-collapse: collapse;
  width: 100%;
}

.crepe-editor.dark-theme th,
.crepe-editor.dark-theme td {
  border: 1px solid #3c3c3c;
  padding: 8px 12px;
  color: #e0e0e0;
}

.crepe-editor.dark-theme th {
  background: #232323;
  font-weight: 600;
}

/* Links - dark theme */
.crepe-editor.dark-theme a {
  color: #58a6ff;
  text-decoration: none;
}

.crepe-editor.dark-theme a:hover {
  text-decoration: underline;
}

/* Horizontal rule - dark theme */
.crepe-editor.dark-theme hr {
  border: none;
  border-top: 1px solid #3c3c3c;
  margin: 24px 0;
}

/* Task list */
.crepe-editor li[data-task-list-item] {
  list-style: none;
}

.crepe-editor li[data-task-list-item]::before {
  content: none;
}

/* Headings - dark theme */
.crepe-editor.dark-theme h1,
.crepe-editor.dark-theme h2,
.crepe-editor.dark-theme h3,
.crepe-editor.dark-theme h4,
.crepe-editor.dark-theme h5,
.crepe-editor.dark-theme h6 {
  color: #e0e0e0;
}

/* Lists - dark theme */
.crepe-editor.dark-theme ul,
.crepe-editor.dark-theme ol,
.crepe-editor.dark-theme li {
  color: #e0e0e0;
}

/* Placeholder - dark theme */
.crepe-editor.dark-theme .ProseMirror p.is-editor-empty:first-child::before {
  color: #999999;
  content: attr(data-placeholder);
  float: left;
  height: 0;
  pointer-events: none;
}

/* Selection - dark theme */
.crepe-editor.dark-theme .ProseMirror ::selection {
  background: #58a6ff;
  color: white;
}

/* Focus state */
.crepe-editor .ProseMirror:focus {
  outline: none;
}

/* Image blocks */
.crepe-editor img {
  max-width: 100%;
  border-radius: 6px;
}
</style>
