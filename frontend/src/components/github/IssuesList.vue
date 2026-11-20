<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useGithubStore, type GithubIssueFilter } from '../../stores/github'
import type { GithubIssue } from '../../types'
import { relativeTime } from '../../lib/relativeTime'
import TrackButton from './TrackButton.vue'

const props = defineProps<{
  projectId: string
}>()

const emit = defineEmits<{
  select: [issue: GithubIssue]
}>()

const githubStore = useGithubStore()
const { filteredIssues, issueFilter, hasLogin, snapshot } = storeToRefs(githubStore)

const ISSUE_TABS: { id: GithubIssueFilter; label: string; loginRequired: boolean }[] = [
  { id: 'all', label: 'All open', loginRequired: false },
  { id: 'assigned', label: 'Assigned to me', loginRequired: true },
  { id: 'created', label: 'Created by me', loginRequired: true },
]

function visibleTabs() {
  return ISSUE_TABS.filter((t) => !t.loginRequired || hasLogin.value)
}

function onTab(filter: GithubIssueFilter) {
  githubStore.setIssueFilter(props.projectId, filter)
}

function labelBackground(color: string | undefined | null): string {
  if (!color || !color.trim()) {
    return '#e5e7eb'
  }
  const trimmed = color.trim()
  return trimmed.startsWith('#') ? trimmed : `#${trimmed}`
}

function onSelectIssue(issue: GithubIssue) {
  emit('select', issue)
}
</script>

<template>
  <section class="issues-list" data-testid="issues-list">
    <div class="issues-list__tabs" role="tablist" aria-label="Issue filters">
      <button
        v-for="tab in visibleTabs()"
        :key="tab.id"
        type="button"
        role="tab"
        class="issues-list__tab"
        :class="{ 'issues-list__tab--active': issueFilter === tab.id }"
        :aria-selected="issueFilter === tab.id"
        :data-testid="`issue-filter-${tab.id}`"
        @click="onTab(tab.id)"
      >
        {{ tab.label }}
      </button>
    </div>

    <p
      v-if="snapshot?.issues_truncated"
      class="issues-list__truncate"
      data-testid="issues-truncated-notice"
    >
      Showing the 500 most recently updated open items.
    </p>

    <div v-if="filteredIssues.length === 0" class="issues-list__empty" data-testid="issues-empty">
      No open issues
    </div>

    <ul v-else class="issues-list__rows">
      <li
        v-for="issue in filteredIssues"
        :key="issue.number"
        class="issues-list__item"
      >
        <button
          type="button"
          class="issues-list__row"
          :data-testid="`issue-row-${issue.number}`"
          @click="onSelectIssue(issue)"
        >
          <div class="issues-list__title">
            #{{ issue.number }} {{ issue.title }}
          </div>
          <div v-if="issue.labels.length" class="issues-list__labels">
            <span
              v-for="label in issue.labels"
              :key="label.name"
              class="issues-list__label"
              :style="{ backgroundColor: labelBackground(label.color) }"
            >
              {{ label.name }}
            </span>
          </div>
          <div class="issues-list__meta">
            <span class="issues-list__author">{{ issue.author_login }}</span>
            <span
              v-if="issue.assignee_logins.length"
              class="issues-list__assignees"
            >
              → {{ issue.assignee_logins.join(', ') }}
            </span>
            <span class="issues-list__updated">{{ relativeTime(issue.updated_at) }}</span>
          </div>
        </button>
        <TrackButton
          :project-id="projectId"
          :number="issue.number"
          kind="issue"
        />
      </li>
    </ul>
  </section>
</template>

<style scoped>
.issues-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.issues-list__tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.issues-list__tab {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid var(--color-border, #d1d5db);
  border-radius: 6px;
  background: var(--color-bg, #fff);
  cursor: pointer;
}

.issues-list__tab--active {
  border-color: var(--color-accent, #2563eb);
  background: var(--color-bg-subtle, #eff6ff);
  font-weight: 600;
}

.issues-list__truncate {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary, #6b7280);
}

.issues-list__empty {
  padding: 16px;
  font-size: 14px;
  color: var(--color-text-secondary, #6b7280);
}

.issues-list__rows {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.issues-list__item {
  display: flex;
  align-items: stretch;
  gap: 8px;
}

.issues-list__row {
  flex: 1;
  min-width: 0;
  text-align: left;
  padding: 10px 12px;
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  background: var(--color-bg, #fff);
  cursor: pointer;
}

.issues-list__row:hover {
  background: var(--color-bg-hover, #f9fafb);
}

.issues-list__title {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 6px;
}

.issues-list__labels {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 6px;
}

.issues-list__label {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 12px;
  color: #111;
}

.issues-list__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 12px;
  color: var(--color-text-secondary, #6b7280);
}

.issues-list__updated {
  margin-left: auto;
}
</style>