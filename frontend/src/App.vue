<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { 
  useNotesStore, 
  useProjectsStore, 
  useTasksStore, 
  useUiStore, 
  useGitStore,
  useWebSocketStore,
  useWorkspaceStore,
  useThemeStore 
} from './stores'
import { useWebSocket } from './composables/useWebSocket'
import TopBar from './components/TopBar.vue'
import Sidebar from './components/Sidebar.vue'
import ConflictBanner from './components/ConflictBanner.vue'

const notesStore = useNotesStore()
const projectsStore = useProjectsStore()
const tasksStore = useTasksStore()
const uiStore = useUiStore()
const gitStore = useGitStore()
const wsStore = useWebSocketStore()
const workspaceStore = useWorkspaceStore()
const themeStore = useThemeStore()

// Non-blocking external edit notification (replaces blocking confirm())
const externalEditPath = ref<string | null>(null)

function reloadExternalEdit() {
  if (notesStore.currentNote && externalEditPath.value) {
    notesStore.loadNote(notesStore.currentNote.id)
  }
  externalEditPath.value = null
}

function dismissExternalEdit() {
  externalEditPath.value = null
}

// Initialize theme immediately (before mount for no flash)
themeStore.init()

// WebSocket connection with handlers
const { connected, clientId } = useWebSocket({
  onFileCreated: () => {
    notesStore.loadNotes()
    projectsStore.loadProjects()
  },
  onFileModified: (path) => {
    notesStore.loadNotes()
    // Non-blocking notification if current note was modified externally
    if (notesStore.currentNote?.path === path) {
      externalEditPath.value = path
    }
  },
  onFileDeleted: () => {
    notesStore.loadNotes()
    projectsStore.loadProjects()
  },
  onFileLocked: (path, lockClientId, lockType) => {
    wsStore.addFileLock({ 
      path, 
      client_id: lockClientId, 
      lock_type: lockType as 'editor' | 'task_view' 
    })
  },
  onFileUnlocked: (path) => {
    wsStore.removeFileLock(path)
  },
  onGitConflict: (files) => {
    wsStore.setGitConflicts(files)
  }
})

// Sync WebSocket state to store
import { watch } from 'vue'
// Note: watch imported separately to maintain original code structure
watch(connected, (val) => wsStore.setConnected(val))
watch(clientId, (val) => wsStore.setClientId(val))

// Keyboard shortcuts
function handleKeydown(e: KeyboardEvent) {
  // Ctrl/Cmd + K for search
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault()
    uiStore.toggleSearch()
  }
  // Ctrl/Cmd + S to save
  if ((e.ctrlKey || e.metaKey) && e.key === 's') {
    e.preventDefault()
    if (notesStore.currentNote) {
      // Trigger save - the view will handle it
      const event = new CustomEvent('save-note')
      window.dispatchEvent(event)
    }
  }
  // Escape to close panels
  if (e.key === 'Escape') {
    uiStore.closeSearch()
    uiStore.closeTasks()
  }
}

onMounted(async () => {
  // Load initial data
  await Promise.all([
    notesStore.loadNotes(),
    projectsStore.loadProjects(),
    tasksStore.loadAllTasks(),
    gitStore.loadStatus(),
    gitStore.loadRemote()
  ])
  
  // Load saved active project
  await workspaceStore.loadSavedProject()
  
  // Check for git conflicts and remote status periodically
  setInterval(() => {
    gitStore.checkConflicts()
    gitStore.loadRemote()
  }, 60000)

  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div id="app-layout">
    <TopBar />
    <div id="app-container">
      <Sidebar />
      <main class="main">
        <ConflictBanner />
        <div v-if="externalEditPath" class="external-edit-banner">
          File modified externally.
          <button @click="reloadExternalEdit" class="primary" style="margin-left: 8px">Reload</button>
          <button @click="dismissExternalEdit" style="margin-left: 4px">Dismiss</button>
        </div>
        <div v-if="uiStore.globalError" class="error-message">
          {{ uiStore.globalError }}
          <button @click="uiStore.clearGlobalError" style="margin-left: 12px">Dismiss</button>
        </div>
        <router-view />
      </main>
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  width: 100%;
  height: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden;
}

:root {
  --sidebar-width: 280px;
  --header-height: 52px;
  --font-mono: 'SF Mono', 'Monaco', 'Menlo', 'Consolas', monospace;
  
  /* Light theme as base (will be overridden by dark) */
  --color-bg: #ffffff;
  --color-bg-secondary: #f8f9fa;
  --color-bg-hover: #f1f3f5;
  --color-border: #e1e4e8;
  --color-text: #24292e;
  --color-text-secondary: #586069;
  --color-primary: #0366d6;
  --color-danger: #cb2431;
  --color-success: #28a745;
  --color-warning: #f0ad4e;
}

/* Light theme explicit */
[data-theme="light"] {
  --color-bg: #ffffff;
  --color-bg-secondary: #f8f9fa;
  --color-bg-hover: #f1f3f5;
  --color-border: #e1e4e8;
  --color-text: #24292e;
  --color-text-secondary: #586069;
  --color-primary: #0366d6;
  --color-danger: #cb2431;
  --color-success: #28a745;
  --color-warning: #f0ad4e;
}

/* Dark theme - overrides light when data-theme="dark" */
[data-theme="dark"] {
  --color-bg: #1a1a1a;
  --color-bg-secondary: #232323;
  --color-bg-hover: #2d2d2d;
  --color-border: #3c3c3c;
  --color-text: #e0e0e0;
  --color-text-secondary: #999999;
  --color-primary: #58a6ff;
  --color-danger: #f85149;
  --color-success: #56d364;
  --color-warning: #d29922;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
  font-size: 14px;
  line-height: 1.5;
  color: var(--color-text);
  background: var(--color-bg);
}

#app {
  width: 100%;
  height: 100vh;
  overflow: hidden;
}

#app-layout {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  overflow: hidden;
}

#app-container {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-width: 0;
}

.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.external-edit-banner {
  padding: 10px 16px;
  background: var(--color-warning);
  color: #1a1a1a;
  font-size: 13px;
  display: flex;
  align-items: center;
}

.error-message {
  padding: 12px 16px;
  background: var(--color-danger);
  color: white;
  font-size: 13px;
}

button {
  padding: 6px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;
}

button:hover {
  background: var(--color-bg-secondary);
  border-color: var(--color-text-secondary);
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

button.primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: white;
}

button.primary:hover {
  opacity: 0.9;
}

button.danger {
  color: var(--color-danger);
}

button.danger:hover {
  background: var(--color-danger);
  border-color: var(--color-danger);
  color: white;
}

input[type="text"],
input[type="search"] {
  padding: 8px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 14px;
  outline: none;
}

input[type="text"]:focus,
input[type="search"]:focus {
  border-color: var(--color-primary);
}
</style>
