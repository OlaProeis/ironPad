<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useGithubStore } from '../stores/github'
import RepoLinkForm from '../components/github/RepoLinkForm.vue'
import IssuesList from '../components/github/IssuesList.vue'
import PullsList from '../components/github/PullsList.vue'
import ItemDetailPane from '../components/github/ItemDetailPane.vue'
import type { GithubIssue, GithubPull } from '../types'

const REFRESH_INTERVAL_MS = 15 * 60 * 1000

const props = defineProps<{
  id: string
}>()

const githubStore = useGithubStore()
const { error } = storeToRefs(githubStore)

const hasRepo = computed(() => Boolean(githubStore.snapshot?.repo))

const selectedItem = ref<GithubIssue | GithubPull | null>(null)
const selectedKind = ref<'issue' | 'pull'>('issue')
const refreshing = ref(false)
const errorBannerDismissed = ref(false)
const refreshIntervalId = ref<ReturnType<typeof setInterval> | null>(null)

const updatedTimeLabel = computed(() => {
  const refreshedAt = githubStore.snapshot?.refreshed_at
  if (!refreshedAt) {
    return null
  }
  return new Date(refreshedAt).toLocaleTimeString()
})

const showErrorBanner = computed(
  () => Boolean(error.value) && !errorBannerDismissed.value,
)

function clearRefreshInterval() {
  if (refreshIntervalId.value !== null) {
    clearInterval(refreshIntervalId.value)
    refreshIntervalId.value = null
  }
}

function startRefreshInterval(projectId: string) {
  clearRefreshInterval()
  if (!projectId) {
    return
  }
  refreshIntervalId.value = setInterval(() => {
    if (!githubStore.snapshot?.repo) {
      return
    }
    void githubStore.refresh(projectId)
  }, REFRESH_INTERVAL_MS)
}

async function loadForProject(projectId: string) {
  if (!projectId) {
    return
  }
  await githubStore.loadSnapshot(projectId)
  if (githubStore.snapshot?.stale === true && githubStore.snapshot?.repo) {
    await githubStore.refresh(projectId)
  }
}

async function onRefreshClick() {
  if (refreshing.value) {
    return
  }
  refreshing.value = true
  try {
    await githubStore.refresh(props.id)
  } finally {
    refreshing.value = false
  }
}

function dismissErrorBanner() {
  errorBannerDismissed.value = true
}

const selectedRemoved = computed(() => {
  const item = selectedItem.value
  const snapshot = githubStore.snapshot
  if (!item || !snapshot) {
    return false
  }
  const rows = selectedKind.value === 'issue' ? snapshot.issues : snapshot.pulls
  return !rows.some((row) => row.number === item.number)
})

function onSelectIssue(issue: GithubIssue) {
  selectedKind.value = 'issue'
  selectedItem.value = issue
}

function onSelectPull(pull: GithubPull) {
  selectedKind.value = 'pull'
  selectedItem.value = pull
}

function closeDetail() {
  selectedItem.value = null
}

onMounted(() => {
  void loadForProject(props.id)
  startRefreshInterval(props.id)
})

onUnmounted(() => {
  clearRefreshInterval()
})

watch(
  () => props.id,
  (projectId) => {
    void loadForProject(projectId)
    startRefreshInterval(projectId)
  },
)

watch(error, (value) => {
  if (!value) {
    errorBannerDismissed.value = false
  }
})
</script>

<template>
  <div class="github-view">
    <header class="github-header">
      <h1>GitHub</h1>
      <div v-if="!githubStore.loading && hasRepo" class="github-header-actions">
        <span v-if="updatedTimeLabel" class="updated-time" data-testid="github-updated-time">
          Updated {{ updatedTimeLabel }}
        </span>
        <button
          type="button"
          class="refresh-btn"
          data-testid="github-refresh-btn"
          :disabled="refreshing"
          @click="onRefreshClick"
        >
          <span v-if="refreshing" class="refresh-spinner" aria-hidden="true" />
          Refresh
        </button>
      </div>
    </header>

    <div
      v-if="showErrorBanner"
      class="error-banner"
      data-testid="github-error-banner"
      role="alert"
    >
      <span class="error-banner-text">{{ error }}</span>
      <button
        type="button"
        class="error-banner-dismiss"
        aria-label="Dismiss error"
        @click="dismissErrorBanner"
      >
        ×
      </button>
    </div>

    <div v-if="githubStore.loading" class="loading">Loading GitHub data...</div>

    <template v-else>
      <RepoLinkForm v-if="!hasRepo" :project-id="id" />
      <div v-else class="github-panels">
        <div class="github-lists">
          <IssuesList :project-id="id" @select="onSelectIssue" />
          <PullsList :project-id="id" @select="onSelectPull" />
        </div>
        <ItemDetailPane
          v-if="selectedItem"
          :project-id="id"
          :item="selectedItem"
          :kind="selectedKind"
          :removed="selectedRemoved"
          @close="closeDetail"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.github-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 24px;
}

.github-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}

.github-header h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.github-header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  font-size: 14px;
}

.updated-time {
  color: var(--color-text-secondary, #6b7280);
}

.refresh-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  font-size: 14px;
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 6px;
  background: var(--color-bg, #fff);
  cursor: pointer;
}

.refresh-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.refresh-spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--color-border, #e5e7eb);
  border-top-color: currentColor;
  border-radius: 50%;
  animation: github-refresh-spin 0.8s linear infinite;
}

@keyframes github-refresh-spin {
  to {
    transform: rotate(360deg);
  }
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  padding: 10px 12px;
  font-size: 14px;
  color: #991b1b;
  background: #fef2f2;
  border: 1px solid #fecaca;
  border-radius: 6px;
}

.error-banner-text {
  flex: 1;
  min-width: 0;
}

.error-banner-dismiss {
  flex-shrink: 0;
  padding: 0 4px;
  font-size: 18px;
  line-height: 1;
  border: none;
  background: transparent;
  color: inherit;
  cursor: pointer;
}

.github-panels {
  display: flex;
  flex: 1;
  gap: 24px;
  overflow: hidden;
  min-height: 0;
}

.github-lists {
  display: flex;
  flex-direction: column;
  gap: 24px;
  flex: 1;
  overflow: auto;
  min-width: 0;
}

.loading {
  color: var(--color-text-secondary);
  font-size: 14px;
}
</style>