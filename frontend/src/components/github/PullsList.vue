<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useGithubStore, type GithubPrFilter } from '../../stores/github'
import type { GithubPull } from '../../types'
import { relativeTime } from '../../lib/relativeTime'

const props = defineProps<{
  projectId: string
}>()

const emit = defineEmits<{
  select: [pull: GithubPull]
}>()

const githubStore = useGithubStore()
const { filteredPulls, prFilter, hasLogin, snapshot } = storeToRefs(githubStore)

const PR_TABS: { id: GithubPrFilter; label: string; loginRequired: boolean }[] = [
  { id: 'all', label: 'All open', loginRequired: false },
  { id: 'mine', label: 'Mine', loginRequired: true },
  { id: 'review', label: 'Review requested', loginRequired: true },
]

function visibleTabs() {
  return PR_TABS.filter((t) => !t.loginRequired || hasLogin.value)
}

function onTab(filter: GithubPrFilter) {
  githubStore.setPrFilter(props.projectId, filter)
}

function labelBackground(color: string | undefined | null): string {
  if (!color || !color.trim()) {
    return '#e5e7eb'
  }
  const trimmed = color.trim()
  return trimmed.startsWith('#') ? trimmed : `#${trimmed}`
}

function onSelectPull(pull: GithubPull) {
  emit('select', pull)
}
</script>

<template>
  <section class="pulls-list" data-testid="pulls-list">
    <div class="pulls-list__tabs" role="tablist" aria-label="Pull request filters">
      <button
        v-for="tab in visibleTabs()"
        :key="tab.id"
        type="button"
        role="tab"
        class="pulls-list__tab"
        :class="{ 'pulls-list__tab--active': prFilter === tab.id }"
        :aria-selected="prFilter === tab.id"
        :data-testid="`pr-filter-${tab.id}`"
        @click="onTab(tab.id)"
      >
        {{ tab.label }}
      </button>
    </div>

    <p
      v-if="snapshot?.pulls_truncated"
      class="pulls-list__truncate"
      data-testid="pulls-truncated-notice"
    >
      Showing the 500 most recently updated open items.
    </p>

    <div v-if="filteredPulls.length === 0" class="pulls-list__empty" data-testid="pulls-empty">
      No open pull requests
    </div>

    <ul v-else class="pulls-list__rows">
      <li v-for="pull in filteredPulls" :key="pull.number">
        <button
          type="button"
          class="pulls-list__row"
          :data-testid="`pr-row-${pull.number}`"
          @click="onSelectPull(pull)"
        >
          <div class="pulls-list__title">
            #{{ pull.number }} {{ pull.title }}
            <span
              v-if="pull.draft"
              class="pulls-list__draft"
              data-testid="pr-draft-badge"
            >
              Draft
            </span>
          </div>
          <div v-if="pull.labels.length" class="pulls-list__labels">
            <span
              v-for="label in pull.labels"
              :key="label.name"
              class="pulls-list__label"
              :style="{ backgroundColor: labelBackground(label.color) }"
            >
              {{ label.name }}
            </span>
          </div>
          <div class="pulls-list__meta">
            <span class="pulls-list__author">{{ pull.author_login }}</span>
            <span class="pulls-list__updated">{{ relativeTime(pull.updated_at) }}</span>
          </div>
        </button>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.pulls-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.pulls-list__tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.pulls-list__tab {
  padding: 6px 12px;
  font-size: 13px;
  border: 1px solid var(--color-border, #d1d5db);
  border-radius: 6px;
  background: var(--color-bg, #fff);
  cursor: pointer;
}

.pulls-list__tab--active {
  border-color: var(--color-accent, #2563eb);
  background: var(--color-bg-subtle, #eff6ff);
  font-weight: 600;
}

.pulls-list__truncate {
  margin: 0;
  font-size: 12px;
  color: var(--color-text-secondary, #6b7280);
}

.pulls-list__empty {
  padding: 16px;
  font-size: 14px;
  color: var(--color-text-secondary, #6b7280);
}

.pulls-list__rows {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pulls-list__row {
  width: 100%;
  text-align: left;
  padding: 10px 12px;
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  background: var(--color-bg, #fff);
  cursor: pointer;
}

.pulls-list__row:hover {
  background: var(--color-bg-hover, #f9fafb);
}

.pulls-list__title {
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 6px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.pulls-list__draft {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--color-text-secondary, #9ca3af);
  color: #fff;
}

.pulls-list__labels {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-bottom: 6px;
}

.pulls-list__label {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 12px;
  color: #111;
}

.pulls-list__meta {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  font-size: 12px;
  color: var(--color-text-secondary, #6b7280);
}

.pulls-list__updated {
  margin-left: auto;
}
</style>