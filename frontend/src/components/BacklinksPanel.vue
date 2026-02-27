<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { backlinksApi } from '../api/client'
import type { Backlink, ForwardLink } from '../types'

const props = defineProps<{
  noteId: string
  projectId?: string
  refreshKey?: number
}>()

const router = useRouter()

// State
const backlinks = ref<Backlink[]>([])
const forwardLinks = ref<ForwardLink[]>([])
const loading = ref(false)
const error = ref<string | null>(null)
const expanded = ref(true)

const totalLinks = computed(() => backlinks.value.length + forwardLinks.value.length)

const hasLinks = computed(() => totalLinks.value > 0)

async function loadLinks() {
  if (!props.noteId) return

  loading.value = true
  error.value = null

  try {
    const response = await backlinksApi.getLinks(props.noteId)
    backlinks.value = response.backlinks || []
    forwardLinks.value = response.forward_links || []
  } catch (err) {
    error.value = `Failed to load links: ${err}`
    console.error('[BacklinksPanel] Error loading links:', err)
  } finally {
    loading.value = false
  }
}

// Navigate to a note
function navigateToNote(link: Backlink | ForwardLink) {
  const targetId = 'source_id' in link ? link.source_id : link.target_id
  
  // Check if it's a project note by looking at the path
  const path = 'source_path' in link ? link.source_path : null
  const isProjectNote = path?.startsWith('projects/')
  
  if (isProjectNote && path) {
    // Extract project ID from path: projects/{id}/notes/{note}.md
    const parts = path.split('/')
    if (parts.length >= 3) {
      const projectId = parts[1]
      const filename = parts[parts.length - 1]
      if (filename) {
        const noteFilename = filename.replace('.md', '')
        router.push({
          name: 'project-notes',
          params: { id: projectId, noteId: noteFilename }
        })
        return
      }
    }
  }
  
  // Regular note or project index
  router.push({ name: 'note', params: { id: targetId } })
}

// Get context preview (truncated)
function getContextPreview(context: string): string {
  if (!context) return ''
  const maxLen = 80
  if (context.length > maxLen) {
    return context.substring(0, maxLen) + '...'
  }
  return context
}

// Watch for note ID changes and reload
watch(() => props.noteId, (newId, oldId) => {
  if (newId && newId !== oldId) {
    loadLinks()
  }
}, { immediate: true })

watch(() => props.refreshKey, (newKey, oldKey) => {
  if (newKey !== undefined && newKey !== oldKey) {
    loadLinks()
  }
})

// Reload on mount
onMounted(() => {
  if (props.noteId) {
    loadLinks()
  }
})
</script>

<template>
  <div class="backlinks-panel">
    <div class="panel-header" @click="expanded = !expanded">
      <h4>
        <span class="icon">{{ expanded ? '▼' : '▶' }}</span>
        Linked Notes
        <span v-if="totalLinks > 0" class="badge">{{ totalLinks }}</span>
      </h4>
    </div>

    <div v-show="expanded" class="panel-content">
      <div v-if="loading" class="loading">Loading...</div>
      <div v-else-if="error" class="error">{{ error }}</div>
      <div v-else-if="!hasLinks" class="empty-state">
        <p>No links yet</p>
        <p class="hint">Type /link to insert a note link</p>
      </div>
      <div v-else>
        <!-- Forward Links (Outgoing) -->
        <div v-if="forwardLinks.length > 0" class="link-section">
          <h5 class="section-title">Links To</h5>
          <div class="link-list">
            <div
              v-for="link in forwardLinks"
              :key="link.target_id"
              class="link-item"
              @click="navigateToNote(link)"
            >
              <div class="link-title">{{ link.target_title || link.target_id }}</div>
              <div v-if="link.context" class="link-context">{{ getContextPreview(link.context) }}</div>
            </div>
          </div>
        </div>

        <!-- Backlinks (Incoming) -->
        <div v-if="backlinks.length > 0" class="link-section">
          <h5 class="section-title">Linked From</h5>
          <div class="link-list">
            <div
              v-for="link in backlinks"
              :key="`${link.source_id}-${link.line_number}`"
              class="link-item"
              @click="navigateToNote(link)"
            >
              <div class="link-title">{{ link.source_title || link.source_id }}</div>
              <div v-if="link.context" class="link-context">{{ getContextPreview(link.context) }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backlinks-panel {
  border-top: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
  flex-shrink: 0;
}

.panel-header {
  padding: 8px 16px;
  cursor: pointer;
  user-select: none;
  transition: background 0.15s;
}

.panel-header:hover {
  background: var(--color-border);
}

.panel-header h4 {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--color-text);
}

.icon {
  font-size: 10px;
  color: var(--color-text-secondary);
}

.badge {
  background: var(--color-primary);
  color: white;
  padding: 1px 6px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: 500;
}

.panel-content {
  max-height: 300px;
  overflow-y: auto;
}

.link-section {
  padding: 8px 0;
  border-bottom: 1px solid var(--color-border);
}

.link-section:last-child {
  border-bottom: none;
}

.section-title {
  margin: 0 0 6px 16px;
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--color-text-secondary);
}

.link-list {
  padding: 0 8px;
}

.link-item {
  padding: 8px 12px;
  margin-bottom: 4px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.link-item:hover {
  background: var(--color-border);
}

.link-title {
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.link-context {
  font-size: 11px;
  color: var(--color-text-secondary);
  margin-top: 2px;
  line-height: 1.3;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.loading,
.error {
  padding: 16px;
  font-size: 12px;
  text-align: center;
  color: var(--color-text-secondary);
}

.error {
  color: var(--color-danger);
}

.empty-state {
  padding: 16px;
  text-align: center;
  color: var(--color-text-secondary);
}

.empty-state p {
  margin: 0;
  font-size: 12px;
}

.empty-state .hint {
  margin-top: 4px;
  font-size: 11px;
  opacity: 0.7;
}
</style>
