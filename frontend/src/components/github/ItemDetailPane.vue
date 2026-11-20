<script setup lang="ts">
import { ref, watch } from 'vue'
import { githubApi } from '../../api/client'
import RenderedMarkdown from '../library/RenderedMarkdown.vue'
import { relativeTime } from '../../lib/relativeTime'
import type { GithubIssue, GithubItemDetail, GithubPull } from '../../types'
import TrackButton from './TrackButton.vue'

const props = defineProps<{
  projectId: string
  item: GithubIssue | GithubPull | null
  kind: 'issue' | 'pull'
  removed?: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const detail = ref<GithubItemDetail | null>(null)
let requestSerial = 0

watch(
  () => [props.projectId, props.kind, props.item?.number] as const,
  async ([projectId, kind, number]) => {
    if (!props.item || number == null) {
      return
    }
    const serial = ++requestSerial
    try {
      const next =
        kind === 'pull'
          ? await githubApi.getPull(projectId, number)
          : await githubApi.getIssue(projectId, number)
      if (serial === requestSerial) {
        detail.value = next
      }
    } catch {
      // Keep the previous detail so a failed refresh still shows the cached body.
    }
  },
  { immediate: true },
)

function statusClass(status: string | undefined): string {
  if (status === 'success') return 'status-dot--success'
  if (status === 'pending') return 'status-dot--pending'
  if (status === 'failure' || status === 'error') return 'status-dot--failure'
  return 'status-dot--none'
}

async function copyLink() {
  if (!props.item) return
  await navigator.clipboard.writeText(props.item.html_url)
}
</script>

<template>
  <aside v-if="item" class="item-detail-pane" data-testid="item-detail-pane">
    <header class="item-detail-pane__header">
      <h2 class="item-detail-pane__title">{{ item.title }}</h2>
      <button
        type="button"
        class="item-detail-pane__close"
        data-testid="detail-close"
        aria-label="Close"
        @click="emit('close')"
      >
        ×
      </button>
    </header>

    <p v-if="removed" class="item-detail-pane__removed" data-testid="item-removed">
      This item is no longer open
    </p>

    <div class="item-detail-pane__actions">
      <a
        :href="item.html_url"
        target="_blank"
        rel="noreferrer"
        data-testid="open-on-github"
      >
        Open on GitHub
      </a>
      <button type="button" data-testid="copy-link" @click="copyLink">
        Copy link
      </button>
    </div>

    <div
      v-if="detail?.error"
      class="error-banner"
      data-testid="detail-error"
      role="alert"
    >
      <span class="error-banner-text">{{ detail.error }}</span>
    </div>

    <template v-if="detail">
      <p
        v-if="kind === 'pull' && detail.commit_status"
        class="commit-status"
        data-testid="commit-status"
      >
        <span
          class="status-dot"
          :class="statusClass(detail.commit_status)"
          aria-hidden="true"
        />
        Status: {{ detail.commit_status }}
      </p>

      <p
        v-if="detail.body_too_large"
        class="body-too-large"
        data-testid="body-too-large"
      >
        The body is too large to display
      </p>
      <RenderedMarkdown v-else-if="detail.body" :source="detail.body" />

      <TrackButton
        :project-id="projectId"
        :number="item.number"
        :kind="kind"
      />

      <ul v-if="detail.comments.length" class="comments" data-testid="comments">
        <li
          v-for="(comment, index) in detail.comments"
          :key="`${comment.author_login}-${comment.created_at}-${index}`"
          class="comment"
        >
          <div class="comment__meta">
            <span class="comment__author">{{ comment.author_login }}</span>
            <span class="comment__time">{{ relativeTime(comment.created_at) }}</span>
          </div>
          <RenderedMarkdown :source="comment.body" />
        </li>
      </ul>

      <p v-if="detail.more_comments" class="more-comments" data-testid="more-comments">
        Earlier comments are on GitHub
      </p>
    </template>
  </aside>
</template>

<style scoped>
.item-detail-pane {
  flex: 0 0 320px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  border: 1px solid var(--color-border, #e5e7eb);
  border-radius: 8px;
  font-size: 14px;
  overflow: auto;
  min-width: 0;
}

.item-detail-pane__header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.item-detail-pane__title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.item-detail-pane__close {
  flex-shrink: 0;
  padding: 0 4px;
  font-size: 18px;
  line-height: 1;
  border: none;
  background: transparent;
  cursor: pointer;
}

.item-detail-pane__removed {
  margin: 0;
  color: var(--color-text-secondary, #6b7280);
}

.item-detail-pane__actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.item-detail-pane__actions a {
  color: var(--color-primary, #2563eb);
}

.error-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
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

.commit-status {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #9ca3af;
}

.status-dot--success {
  background: #16a34a;
}

.status-dot--pending {
  background: #ca8a04;
}

.status-dot--failure {
  background: #dc2626;
}

.status-dot--none {
  background: #9ca3af;
}

.body-too-large,
.more-comments {
  margin: 0;
  color: var(--color-text-secondary, #6b7280);
}

.comments {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.comment__meta {
  display: flex;
  gap: 8px;
  font-size: 13px;
}

.comment__author {
  font-weight: 600;
}

.comment__time {
  color: var(--color-text-secondary, #6b7280);
}
</style>
