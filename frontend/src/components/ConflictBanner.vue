<script setup lang="ts">
import { useGitStore } from '../stores'

const gitStore = useGitStore()
</script>

<template>
  <div v-if="gitStore.hasConflicts" class="conflict-banner">
    <span class="icon">⚠️</span>
    <span class="message">
      Git conflicts detected in {{ gitStore.conflicts.length }} file(s). 
      Please resolve conflicts manually using git or an external tool.
    </span>
    <details class="conflict-files">
      <summary>Show files</summary>
      <ul>
        <li v-for="file in gitStore.conflicts" :key="file">{{ file }}</li>
      </ul>
    </details>
  </div>
</template>

<style scoped>
.conflict-banner {
  padding: 12px 16px;
  background: var(--color-danger);
  color: white;
  font-size: 13px;
  display: flex;
  align-items: flex-start;
  gap: 8px;
  flex-wrap: wrap;
}

.icon {
  font-size: 14px;
  flex-shrink: 0;
}

.message {
  flex: 1;
  min-width: 200px;
}

.conflict-files {
  width: 100%;
  margin-top: 8px;
}

.conflict-files summary {
  cursor: pointer;
  font-weight: 500;
}

.conflict-files ul {
  margin: 8px 0 0 0;
  padding-left: 20px;
}

.conflict-files li {
  font-family: monospace;
  font-size: 12px;
  margin: 4px 0;
}
</style>
