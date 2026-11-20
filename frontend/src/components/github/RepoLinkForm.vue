<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { githubApi } from '../../api/client'
import { useGithubStore } from '../../stores/github'

const REPO_VALIDATION_MSG =
  'Use owner/name, with no URL and no .git suffix.'

const REPO_PATTERN = /^[A-Za-z0-9._-]+\/[A-Za-z0-9._-]+$/
const ALLOWED_CHARS = /^[A-Za-z0-9._\-/]+$/

function validateRepoInput(raw: string): string | null {
  const trimmed = raw.trim()
  if (!trimmed) {
    return REPO_VALIDATION_MSG
  }
  if (trimmed.includes('://')) {
    return REPO_VALIDATION_MSG
  }
  if (trimmed.endsWith('.git')) {
    return REPO_VALIDATION_MSG
  }
  if (trimmed.includes('..')) {
    return REPO_VALIDATION_MSG
  }
  const slashCount = (trimmed.match(/\//g) ?? []).length
  if (slashCount !== 1) {
    return REPO_VALIDATION_MSG
  }
  if (!ALLOWED_CHARS.test(trimmed)) {
    return REPO_VALIDATION_MSG
  }
  if (!REPO_PATTERN.test(trimmed)) {
    return REPO_VALIDATION_MSG
  }
  return null
}

const props = defineProps<{
  projectId: string
}>()

const emit = defineEmits<{
  saved: []
  removed: []
}>()

const githubStore = useGithubStore()
const { snapshot } = storeToRefs(githubStore)

const repoInput = ref('')
const localError = ref<string | null>(null)
const confirmingRemove = ref(false)
const saving = ref(false)

const showRemove = computed(() => Boolean(snapshot.value?.repo))

watch(
  () => snapshot.value?.repo,
  (repo) => {
    if (repo) {
      repoInput.value = repo
    }
  },
  { immediate: true },
)

async function onSave() {
  localError.value = null
  const validationError = validateRepoInput(repoInput.value)
  if (validationError) {
    localError.value = validationError
    return
  }
  const trimmed = repoInput.value.trim()
  saving.value = true
  try {
    await githubApi.putRepo(props.projectId, trimmed)
    await githubStore.refresh(props.projectId)
    emit('saved')
  } catch (err) {
    localError.value = err instanceof Error ? err.message : String(err)
  } finally {
    saving.value = false
  }
}

function onRemoveClick() {
  confirmingRemove.value = true
}

function onRemoveCancel() {
  confirmingRemove.value = false
}

async function onRemoveConfirm() {
  localError.value = null
  try {
    await githubApi.putRepo(props.projectId, null)
    githubStore.snapshot = null
    confirmingRemove.value = false
    emit('removed')
  } catch (err) {
    localError.value = err instanceof Error ? err.message : String(err)
  }
}
</script>

<template>
  <div class="repo-link-form" data-testid="repo-link-form">
    <label class="field">
      <span class="label">Repository</span>
      <input
        v-model="repoInput"
        type="text"
        class="repo-input"
        placeholder="owner/name"
        data-testid="repo-link-input"
        :disabled="saving"
      />
    </label>

    <p class="helper">
      A github.com repository, separate from Remote backup. Ironpad does not store a token.
    </p>

    <div class="setup" data-testid="repo-link-setup">
      <p class="setup-title">How to connect</p>
      <ol>
        <li>
          Enter the repository as <code>owner/name</code>. Do not paste a browser URL or a
          <code>.git</code> address.
        </li>
        <li>
          Sign in on this computer before you save. Run <code>gh auth login</code>, or set
          <code>GH_TOKEN</code> (or <code>GITHUB_TOKEN</code>) in the environment that starts
          Ironpad. There is no token field.
        </li>
        <li>
          A git SSH key used for Remote backup is not a GitHub API login. Ironpad only reads
          issues and pull requests.
        </li>
      </ol>
    </div>

    <p v-if="localError" class="error" data-testid="repo-link-error">
      {{ localError }}
    </p>

    <div class="actions">
      <button
        type="button"
        class="primary"
        data-testid="repo-link-save"
        :disabled="saving"
        @click="onSave"
      >
        Save
      </button>

      <template v-if="showRemove && !confirmingRemove">
        <button
          type="button"
          class="danger"
          data-testid="repo-link-remove"
          @click="onRemoveClick"
        >
          Remove
        </button>
      </template>
    </div>

    <div v-if="confirmingRemove" class="remove-confirm" data-testid="repo-link-remove-confirm">
      <p class="confirm-text">
        Stop showing this repository. Tasks you already created stay.
      </p>
      <div class="confirm-actions">
        <button
          type="button"
          class="danger"
          data-testid="repo-link-remove-confirm-btn"
          @click="onRemoveConfirm"
        >
          Confirm
        </button>
        <button
          type="button"
          data-testid="repo-link-remove-cancel"
          @click="onRemoveCancel"
        >
          Cancel
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.repo-link-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 480px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.label {
  font-size: 13px;
  font-weight: 600;
}

.repo-input {
  padding: 8px 10px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-size: 14px;
  background: var(--color-bg);
  color: var(--color-text);
}

.helper {
  margin: 0;
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.4;
}

.setup {
  margin: 0;
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-bg);
}

.setup-title {
  margin: 0 0 6px;
  font-size: 13px;
  font-weight: 600;
}

.setup ol {
  margin: 0;
  padding-left: 18px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
  line-height: 1.4;
  color: var(--color-text-secondary);
}

.setup code {
  font-family: var(--font-mono, ui-monospace, monospace);
  font-size: 12px;
  color: var(--color-text);
}

.error {
  margin: 0;
  font-size: 13px;
  color: var(--color-danger, #c62828);
}

.actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

button {
  padding: 6px 14px;
  font-size: 13px;
  border-radius: 4px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-secondary, var(--color-bg));
  cursor: pointer;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

button.primary {
  background: var(--color-accent, #2563eb);
  color: #fff;
  border-color: transparent;
}

button.danger {
  color: var(--color-danger, #c62828);
  border-color: var(--color-danger, #c62828);
}

.remove-confirm {
  padding: 12px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.confirm-text {
  margin: 0;
  font-size: 13px;
}

.confirm-actions {
  display: flex;
  gap: 8px;
}
</style>