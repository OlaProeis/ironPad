<script setup lang="ts">
import { useGitStore } from '../stores'
import GitPanel from './GitPanel.vue'

const gitStore = useGitStore()

function openPanel() {
  gitStore.togglePanel()
}
</script>

<template>
  <!-- Git Panel (slides in from right) -->
  <GitPanel />
  
  <div v-if="gitStore.isRepo" class="git-status" @click="openPanel" title="Open Git panel">
    <div class="git-info">
      <span :class="['git-indicator', { 'has-changes': gitStore.hasChanges }]">●</span>
      <span class="git-branch">{{ gitStore.branch }}</span>
      <span v-if="gitStore.hasChanges" class="git-changes">
        {{ gitStore.changedFilesCount }} changes
      </span>
    </div>
    <div class="git-sync-status">
      <span v-if="gitStore.remote?.ahead" class="sync-ahead" title="Commits ahead">
        ↑{{ gitStore.remote.ahead }}
      </span>
      <span v-if="gitStore.remote?.behind" class="sync-behind" title="Commits behind">
        ↓{{ gitStore.remote.behind }}
      </span>
      <span class="expand-hint">⋯</span>
    </div>
  </div>
  
  <!-- Conflict Warning -->
  <div v-if="gitStore.hasConflicts" class="git-conflicts" @click="openPanel">
    ⚠️ Git conflicts detected ({{ gitStore.conflicts.length }} files)
  </div>
</template>

<style scoped>
.git-status {
  padding: 10px 16px;
  border-top: 1px solid var(--color-border);
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  transition: background 0.15s;
}

.git-status:hover {
  background: var(--color-bg-hover);
}

.git-info {
  display: flex;
  align-items: center;
  gap: 6px;
}

.git-indicator {
  color: var(--color-success);
}

.git-indicator.has-changes {
  color: var(--color-primary);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.git-branch {
  font-weight: 500;
}

.git-changes {
  color: var(--color-text-secondary);
  background: var(--color-bg);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
}

.git-sync-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.sync-ahead {
  color: var(--color-success);
  font-size: 11px;
  font-weight: 500;
}

.sync-behind {
  color: var(--color-warning);
  font-size: 11px;
  font-weight: 500;
}

.expand-hint {
  color: var(--color-text-secondary);
  font-size: 14px;
}

.git-conflicts {
  padding: 8px 16px;
  background: var(--color-danger);
  color: white;
  font-size: 12px;
  cursor: pointer;
}

.git-conflicts:hover {
  background: #c82333;
}
</style>
