<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useGitStore } from '../stores'

const gitStore = useGitStore()

// Local state
const commitMessage = ref('')
const activeTab = ref<'changes' | 'history'>('changes')
const expandedFiles = ref<Set<string>>(new Set())

// Computed
const canCommit = computed(() => 
  gitStore.hasChanges && commitMessage.value.trim().length > 0
)

const remoteDisplay = computed(() => {
  if (!gitStore.remote) return null
  const url = gitStore.remote.url
  // Extract repo name from URL (handles both https and ssh)
  const match = url.match(/[:/]([^/]+\/[^/.]+)(?:\.git)?$/)
  return match ? match[1] : url
})

// Actions
async function doCommit() {
  if (!canCommit.value) return
  const result = await gitStore.commit(commitMessage.value.trim())
  if (result) {
    commitMessage.value = ''
  }
}

async function doPush() {
  try {
    await gitStore.push()
  } catch {
    // Error handled in store
  }
}

async function doFetch() {
  try {
    await gitStore.fetchRemote()
  } catch {
    // Error handled in store
  }
}

function selectCommit(commitId: string) {
  if (gitStore.selectedCommitId === commitId) {
    gitStore.clearSelectedCommit()
  } else {
    gitStore.loadCommitDiff(commitId)
  }
}

function toggleFile(path: string) {
  if (expandedFiles.value.has(path)) {
    expandedFiles.value.delete(path)
  } else {
    expandedFiles.value.add(path)
  }
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  
  return date.toLocaleDateString()
}

function getStatusIcon(status: string): string {
  switch (status) {
    case 'new':
    case 'added': return '+'
    case 'modified': return '~'
    case 'deleted': return '-'
    case 'renamed': return '→'
    default: return '?'
  }
}

function getStatusClass(status: string): string {
  switch (status) {
    case 'new':
    case 'added': return 'status-added'
    case 'modified': return 'status-modified'
    case 'deleted': return 'status-deleted'
    case 'renamed': return 'status-renamed'
    default: return ''
  }
}

// Close panel on escape
function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    gitStore.togglePanel()
  }
}

// Watch for panel open to load data
watch(() => gitStore.panelOpen, (open) => {
  if (open) {
    document.addEventListener('keydown', handleKeydown)
  } else {
    document.removeEventListener('keydown', handleKeydown)
    gitStore.clearSelectedCommit()
  }
})
</script>

<template>
  <Teleport to="body">
    <Transition name="panel">
      <div v-if="gitStore.panelOpen" class="git-panel-overlay" @click.self="gitStore.togglePanel()">
        <div class="git-panel">
          <!-- Header -->
          <header class="panel-header">
            <div class="header-left">
              <span class="branch-icon">⎇</span>
              <span class="branch-name">{{ gitStore.branch }}</span>
              <span v-if="gitStore.hasChanges" class="changes-badge">
                {{ gitStore.changedFilesCount }}
              </span>
            </div>
            <div class="header-right">
              <div v-if="gitStore.remote" class="remote-info">
                <span class="remote-name" :title="gitStore.remote.url">{{ remoteDisplay }}</span>
                <span v-if="gitStore.remote.ahead > 0" class="ahead">↑{{ gitStore.remote.ahead }}</span>
                <span v-if="gitStore.remote.behind > 0" class="behind">↓{{ gitStore.remote.behind }}</span>
              </div>
              <button class="close-btn" @click="gitStore.togglePanel()" title="Close (Esc)">×</button>
            </div>
          </header>

          <!-- Error Banner -->
          <div v-if="gitStore.error" class="error-banner">
            {{ gitStore.error }}
            <button @click="gitStore.clearError()">×</button>
          </div>

          <!-- Tabs -->
          <div class="tabs">
            <button 
              :class="['tab', { active: activeTab === 'changes' }]"
              @click="activeTab = 'changes'"
            >
              Changes
              <span v-if="gitStore.hasChanges" class="tab-badge">{{ gitStore.changedFilesCount }}</span>
            </button>
            <button 
              :class="['tab', { active: activeTab === 'history' }]"
              @click="activeTab = 'history'"
            >
              History
            </button>
          </div>

          <!-- Content -->
          <div class="panel-content">
            <!-- Changes Tab -->
            <div v-if="activeTab === 'changes'" class="changes-tab">
              <!-- Commit Form -->
              <div class="commit-form">
                <textarea 
                  v-model="commitMessage"
                  placeholder="Commit message..."
                  class="commit-input"
                  rows="2"
                  @keydown.ctrl.enter="doCommit"
                ></textarea>
                <div class="commit-actions">
                  <button 
                    class="commit-btn"
                    :disabled="!canCommit || gitStore.committing"
                    @click="doCommit"
                  >
                    {{ gitStore.committing ? 'Committing...' : 'Commit' }}
                  </button>
                  <button 
                    v-if="gitStore.hasRemote"
                    class="push-btn"
                    :disabled="gitStore.pushing"
                    @click="doPush"
                    :title="gitStore.canPush ? `Push ${gitStore.remote?.ahead} commits` : 'Push to remote'"
                  >
                    {{ gitStore.pushing ? '...' : '↑ Push' }}
                  </button>
                  <button 
                    v-if="gitStore.hasRemote"
                    class="fetch-btn"
                    :disabled="gitStore.fetching"
                    @click="doFetch"
                    title="Fetch from remote"
                  >
                    {{ gitStore.fetching ? '...' : '↓ Fetch' }}
                  </button>
                </div>
              </div>

              <!-- Changed Files List -->
              <div v-if="gitStore.hasChanges" class="files-list">
                <div class="section-header">
                  <span>Staged Changes</span>
                  <span class="stats" v-if="gitStore.workingDiff">
                    <span class="insertions">+{{ gitStore.workingDiff.stats.insertions }}</span>
                    <span class="deletions">-{{ gitStore.workingDiff.stats.deletions }}</span>
                  </span>
                </div>
                
                <div v-if="gitStore.diffLoading && !gitStore.workingDiff" class="loading">
                  Loading diff...
                </div>
                
                <div v-else-if="gitStore.workingDiff" class="diff-files">
                  <div 
                    v-for="file in gitStore.workingDiff.files" 
                    :key="file.path"
                    class="diff-file"
                  >
                    <div 
                      class="file-header"
                      @click="toggleFile(file.path)"
                    >
                      <span :class="['status-icon', getStatusClass(file.status)]">
                        {{ getStatusIcon(file.status) }}
                      </span>
                      <span class="file-path">{{ file.path }}</span>
                      <span class="file-stats">
                        <span v-if="file.additions" class="insertions">+{{ file.additions }}</span>
                        <span v-if="file.deletions" class="deletions">-{{ file.deletions }}</span>
                      </span>
                      <span class="expand-icon">{{ expandedFiles.has(file.path) ? '▼' : '▶' }}</span>
                    </div>
                    
                    <div v-if="expandedFiles.has(file.path)" class="file-diff">
                      <div v-for="(hunk, i) in file.hunks" :key="i" class="diff-hunk">
                        <div class="hunk-header">{{ hunk.header }}</div>
                        <div class="diff-lines">
                          <div 
                            v-for="(line, j) in hunk.lines" 
                            :key="j"
                            :class="['diff-line', {
                              'line-add': line.origin === '+',
                              'line-del': line.origin === '-',
                              'line-ctx': line.origin === ' '
                            }]"
                          >
                            <span class="line-origin">{{ line.origin }}</span>
                            <span class="line-content">{{ line.content }}</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              
              <div v-else class="no-changes">
                <span class="icon">✓</span>
                <span>No changes</span>
              </div>
            </div>

            <!-- History Tab -->
            <div v-if="activeTab === 'history'" class="history-tab">
              <div v-if="gitStore.historyLoading && !gitStore.history.length" class="loading">
                Loading history...
              </div>
              
              <div v-else-if="gitStore.history.length === 0" class="no-history">
                No commits yet
              </div>
              
              <div v-else class="commit-list">
                <div 
                  v-for="commit in gitStore.history" 
                  :key="commit.id"
                  :class="['commit-item', { selected: gitStore.selectedCommitId === commit.id }]"
                  @click="selectCommit(commit.id)"
                >
                  <div class="commit-header">
                    <span class="commit-id">{{ commit.short_id }}</span>
                    <span class="commit-time">{{ formatTimestamp(commit.timestamp) }}</span>
                  </div>
                  <div class="commit-message">{{ commit.message }}</div>
                  <div class="commit-meta">
                    <span class="commit-author">{{ commit.author }}</span>
                    <span v-if="commit.files_changed" class="commit-files">
                      {{ commit.files_changed }} file{{ commit.files_changed !== 1 ? 's' : '' }}
                    </span>
                  </div>
                  
                  <!-- Commit Diff (expanded) -->
                  <div 
                    v-if="gitStore.selectedCommitId === commit.id && gitStore.selectedCommitDiff" 
                    class="commit-diff"
                    @click.stop
                  >
                    <div 
                      v-for="file in gitStore.selectedCommitDiff.files" 
                      :key="file.path"
                      class="diff-file"
                    >
                      <div 
                        class="file-header"
                        @click="toggleFile(`${commit.id}:${file.path}`)"
                      >
                        <span :class="['status-icon', getStatusClass(file.status)]">
                          {{ getStatusIcon(file.status) }}
                        </span>
                        <span class="file-path">{{ file.path }}</span>
                        <span class="file-stats">
                          <span v-if="file.additions" class="insertions">+{{ file.additions }}</span>
                          <span v-if="file.deletions" class="deletions">-{{ file.deletions }}</span>
                        </span>
                        <span class="expand-icon">
                          {{ expandedFiles.has(`${commit.id}:${file.path}`) ? '▼' : '▶' }}
                        </span>
                      </div>
                      
                      <div v-if="expandedFiles.has(`${commit.id}:${file.path}`)" class="file-diff">
                        <div v-for="(hunk, i) in file.hunks" :key="i" class="diff-hunk">
                          <div class="hunk-header">{{ hunk.header }}</div>
                          <div class="diff-lines">
                            <div 
                              v-for="(line, j) in hunk.lines" 
                              :key="j"
                              :class="['diff-line', {
                                'line-add': line.origin === '+',
                                'line-del': line.origin === '-',
                                'line-ctx': line.origin === ' '
                              }]"
                            >
                              <span class="line-origin">{{ line.origin }}</span>
                              <span class="line-content">{{ line.content }}</span>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                  
                  <div 
                    v-else-if="gitStore.selectedCommitId === commit.id && gitStore.diffLoading" 
                    class="loading"
                  >
                    Loading diff...
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* Panel Overlay */
.git-panel-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 1000;
  display: flex;
  justify-content: flex-end;
}

/* Panel */
.git-panel {
  width: 480px;
  max-width: 100%;
  height: 100%;
  background: var(--color-bg);
  display: flex;
  flex-direction: column;
  box-shadow: -4px 0 20px rgba(0, 0, 0, 0.2);
}

/* Transitions */
.panel-enter-active,
.panel-leave-active {
  transition: opacity 0.2s ease;
}

.panel-enter-active .git-panel,
.panel-leave-active .git-panel {
  transition: transform 0.2s ease;
}

.panel-enter-from,
.panel-leave-to {
  opacity: 0;
}

.panel-enter-from .git-panel,
.panel-leave-to .git-panel {
  transform: translateX(100%);
}

/* Header */
.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-bg-secondary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.branch-icon {
  font-size: 14px;
  color: var(--color-text-secondary);
}

.branch-name {
  font-weight: 600;
  font-size: 14px;
}

.changes-badge {
  background: var(--color-primary);
  color: white;
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 10px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.remote-info {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--color-text-secondary);
}

.remote-name {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.ahead {
  color: var(--color-success);
}

.behind {
  color: var(--color-warning);
}

.close-btn {
  background: none;
  border: none;
  font-size: 20px;
  color: var(--color-text-secondary);
  cursor: pointer;
  padding: 0 4px;
}

.close-btn:hover {
  color: var(--color-text);
}

/* Error Banner */
.error-banner {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 16px;
  background: var(--color-danger);
  color: white;
  font-size: 13px;
}

.error-banner button {
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  font-size: 16px;
}

/* Tabs */
.tabs {
  display: flex;
  border-bottom: 1px solid var(--color-border);
}

.tab {
  flex: 1;
  padding: 10px 16px;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  font-size: 13px;
  color: var(--color-text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.tab:hover {
  color: var(--color-text);
  background: var(--color-bg-hover);
}

.tab.active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.tab-badge {
  background: var(--color-primary);
  color: white;
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 8px;
}

/* Content */
.panel-content {
  flex: 1;
  overflow-y: auto;
}

/* Changes Tab */
.changes-tab {
  padding: 16px;
}

.commit-form {
  margin-bottom: 16px;
}

.commit-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-bg);
  color: var(--color-text);
  font-family: inherit;
  font-size: 13px;
  resize: vertical;
  min-height: 60px;
}

.commit-input:focus {
  outline: none;
  border-color: var(--color-primary);
}

.commit-input::placeholder {
  color: var(--color-text-secondary);
}

.commit-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.commit-btn,
.push-btn,
.fetch-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}

.commit-btn {
  flex: 1;
  background: var(--color-primary);
  color: white;
}

.commit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.push-btn,
.fetch-btn {
  background: var(--color-bg-secondary);
  color: var(--color-text);
  border: 1px solid var(--color-border);
}

.push-btn:hover:not(:disabled),
.fetch-btn:hover:not(:disabled) {
  background: var(--color-bg-hover);
}

.push-btn:disabled,
.fetch-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Files List */
.files-list {
  border-top: 1px solid var(--color-border);
  padding-top: 16px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
}

.stats {
  font-weight: normal;
  display: flex;
  gap: 8px;
}

.insertions {
  color: var(--color-success);
}

.deletions {
  color: var(--color-danger);
}

/* Diff Files */
.diff-files {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.diff-file {
  border: 1px solid var(--color-border);
  border-radius: 6px;
  overflow: hidden;
}

.file-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--color-bg-secondary);
  cursor: pointer;
  font-size: 13px;
}

.file-header:hover {
  background: var(--color-bg-hover);
}

.status-icon {
  font-family: monospace;
  font-weight: bold;
  width: 16px;
  text-align: center;
}

.status-added { color: var(--color-success); }
.status-modified { color: var(--color-warning); }
.status-deleted { color: var(--color-danger); }
.status-renamed { color: var(--color-primary); }

.file-path {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: var(--font-mono);
  font-size: 12px;
}

.file-stats {
  display: flex;
  gap: 6px;
  font-size: 11px;
  font-family: var(--font-mono);
}

.expand-icon {
  font-size: 10px;
  color: var(--color-text-secondary);
}

/* File Diff Content */
.file-diff {
  border-top: 1px solid var(--color-border);
  max-height: 300px;
  overflow-y: auto;
}

.diff-hunk {
  font-family: var(--font-mono);
  font-size: 11px;
}

.hunk-header {
  padding: 4px 12px;
  background: var(--color-bg-secondary);
  color: var(--color-text-secondary);
  border-bottom: 1px solid var(--color-border);
}

.diff-lines {
  background: var(--color-bg);
}

.diff-line {
  display: flex;
  line-height: 1.5;
  white-space: pre;
}

.line-origin {
  width: 20px;
  text-align: center;
  user-select: none;
  color: var(--color-text-secondary);
}

.line-content {
  flex: 1;
  padding-right: 12px;
  overflow-x: auto;
}

.line-add {
  background: rgba(46, 160, 67, 0.15);
}

.line-add .line-origin {
  color: var(--color-success);
}

.line-del {
  background: rgba(248, 81, 73, 0.15);
}

.line-del .line-origin {
  color: var(--color-danger);
}

/* No Changes */
.no-changes {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--color-text-secondary);
  gap: 8px;
}

.no-changes .icon {
  font-size: 32px;
  color: var(--color-success);
}

/* History Tab */
.history-tab {
  padding: 8px;
}

.commit-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.commit-item {
  padding: 12px;
  border-radius: 6px;
  cursor: pointer;
  border: 1px solid transparent;
}

.commit-item:hover {
  background: var(--color-bg-hover);
}

.commit-item.selected {
  background: var(--color-bg-secondary);
  border-color: var(--color-border);
}

.commit-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.commit-id {
  font-family: var(--font-mono);
  font-size: 12px;
  color: var(--color-primary);
  font-weight: 500;
}

.commit-time {
  font-size: 11px;
  color: var(--color-text-secondary);
}

.commit-message {
  font-size: 13px;
  margin-bottom: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-meta {
  display: flex;
  gap: 12px;
  font-size: 11px;
  color: var(--color-text-secondary);
}

.commit-diff {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

/* Loading */
.loading {
  text-align: center;
  padding: 20px;
  color: var(--color-text-secondary);
  font-size: 13px;
}

.no-history {
  text-align: center;
  padding: 40px 20px;
  color: var(--color-text-secondary);
}
</style>
