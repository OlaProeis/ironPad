<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { githubApi } from '../../api/client'

const props = defineProps<{
  projectId: string
  number: number
  kind: 'issue' | 'pull'
}>()

const router = useRouter()
const tracking = ref(false)
const tracked = ref(false)
const existingTaskId = ref<string | null>(null)

async function onTrack() {
  tracking.value = true
  try {
    const result =
      props.kind === 'pull'
        ? await githubApi.trackPull(props.projectId, props.number)
        : await githubApi.trackIssue(props.projectId, props.number)
    tracked.value = true
    existingTaskId.value = result.task_id
  } finally {
    tracking.value = false
  }
}

function onOpenTask() {
  if (!existingTaskId.value) return
  router.push({
    name: 'project-tasks',
    params: { id: props.projectId },
    query: { task: existingTaskId.value },
  })
}
</script>

<template>
  <div class="track-button" data-testid="track-button">
    <button
      v-if="!tracked"
      type="button"
      class="track-button__action"
      data-testid="track-as-task"
      :disabled="tracking"
      @click.stop="onTrack"
    >
      <span v-if="tracking" class="track-button__spinner" data-testid="track-spinner" aria-hidden="true" />
      Track as task
    </button>
    <button
      v-else
      type="button"
      class="track-button__action track-button__action--open"
      data-testid="open-task"
      @click.stop="onOpenTask"
    >
      Open task
    </button>
  </div>
</template>

<style scoped>
.track-button {
  flex-shrink: 0;
}

.track-button__action {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  font-size: 12px;
  border: 1px solid var(--color-border, #d1d5db);
  border-radius: 6px;
  background: var(--color-bg, #fff);
  cursor: pointer;
  white-space: nowrap;
}

.track-button__action:disabled {
  cursor: wait;
  opacity: 0.85;
}

.track-button__action--open {
  border-color: var(--color-accent, #2563eb);
  color: var(--color-accent, #2563eb);
}

.track-button__spinner {
  width: 12px;
  height: 12px;
  border: 2px solid #d1d5db;
  border-top-color: var(--color-accent, #2563eb);
  border-radius: 50%;
  animation: track-button-spin 0.7s linear infinite;
}

@keyframes track-button-spin {
  to {
    transform: rotate(360deg);
  }
}
</style>