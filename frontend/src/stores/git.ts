import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GitStatus, CommitInfo, CommitDetail, DiffInfo, RemoteInfo } from '../types'
import { gitApi } from '../api/client'

export const useGitStore = defineStore('git', () => {
  // State
  const status = ref<GitStatus | null>(null)
  const loading = ref(false)
  const committing = ref(false)
  const pushing = ref(false)
  const fetching = ref(false)
  const error = ref<string | null>(null)
  const conflicts = ref<string[]>([])
  
  // New state for expanded git features
  const history = ref<CommitDetail[]>([])
  const historyLoading = ref(false)
  const workingDiff = ref<DiffInfo | null>(null)
  const diffLoading = ref(false)
  const selectedCommitDiff = ref<DiffInfo | null>(null)
  const selectedCommitId = ref<string | null>(null)
  const remote = ref<RemoteInfo | null>(null)
  const panelOpen = ref(false)

  // Getters
  const hasChanges = computed(() => status.value?.has_changes ?? false)
  const hasConflicts = computed(() => conflicts.value.length > 0)
  const branch = computed(() => status.value?.branch ?? 'main')
  const isRepo = computed(() => status.value?.is_repo ?? false)
  const changedFilesCount = computed(() => status.value?.files.length ?? 0)
  const hasRemote = computed(() => remote.value !== null)
  const canPush = computed(() => (remote.value?.ahead ?? 0) > 0)
  const canPull = computed(() => (remote.value?.behind ?? 0) > 0)

  // Actions
  async function loadStatus() {
    try {
      loading.value = true
      error.value = null
      status.value = await gitApi.status()
    } catch (err) {
      error.value = `Failed to load git status: ${err}`
    } finally {
      loading.value = false
    }
  }

  async function commit(message?: string): Promise<CommitInfo | null> {
    try {
      committing.value = true
      error.value = null
      const result = await gitApi.commit(message)
      await loadStatus()
      // Refresh history and diff after commit
      await Promise.all([loadHistory(), loadWorkingDiff(), loadRemote()])
      return result
    } catch (err) {
      error.value = `Commit failed: ${err}`
      return null
    } finally {
      committing.value = false
    }
  }

  async function push() {
    try {
      pushing.value = true
      error.value = null
      const result = await gitApi.push()
      if (!result.success) {
        throw new Error(result.message)
      }
      await Promise.all([loadStatus(), loadRemote()])
    } catch (err) {
      error.value = `Push failed: ${err}`
      throw err
    } finally {
      pushing.value = false
    }
  }

  async function fetchRemote() {
    try {
      fetching.value = true
      error.value = null
      const result = await gitApi.fetch()
      if (!result.success) {
        throw new Error(result.message)
      }
      await loadRemote()
    } catch (err) {
      error.value = `Fetch failed: ${err}`
      throw err
    } finally {
      fetching.value = false
    }
  }

  async function checkConflicts() {
    try {
      error.value = null
      conflicts.value = await gitApi.conflicts()
    } catch (err) {
      // Conflicts endpoint might not exist yet, ignore error
      conflicts.value = []
    }
  }

  async function loadHistory(limit?: number) {
    try {
      historyLoading.value = true
      history.value = await gitApi.log(limit)
    } catch (err) {
      console.error('Failed to load git history:', err)
      history.value = []
    } finally {
      historyLoading.value = false
    }
  }

  async function loadWorkingDiff() {
    try {
      diffLoading.value = true
      workingDiff.value = await gitApi.diff()
    } catch (err) {
      console.error('Failed to load working diff:', err)
      workingDiff.value = null
    } finally {
      diffLoading.value = false
    }
  }

  async function loadCommitDiff(commitId: string) {
    try {
      diffLoading.value = true
      selectedCommitId.value = commitId
      selectedCommitDiff.value = await gitApi.commitDiff(commitId)
    } catch (err) {
      console.error('Failed to load commit diff:', err)
      selectedCommitDiff.value = null
    } finally {
      diffLoading.value = false
    }
  }

  async function loadRemote() {
    try {
      remote.value = await gitApi.remote()
    } catch (err) {
      console.error('Failed to load remote info:', err)
      remote.value = null
    }
  }

  function clearSelectedCommit() {
    selectedCommitId.value = null
    selectedCommitDiff.value = null
  }

  function togglePanel() {
    panelOpen.value = !panelOpen.value
    if (panelOpen.value) {
      // Load all data when opening panel
      Promise.all([
        loadStatus(),
        loadHistory(),
        loadWorkingDiff(),
        loadRemote()
      ])
    }
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    status,
    loading,
    committing,
    pushing,
    fetching,
    error,
    conflicts,
    history,
    historyLoading,
    workingDiff,
    diffLoading,
    selectedCommitDiff,
    selectedCommitId,
    remote,
    panelOpen,
    // Getters
    hasChanges,
    hasConflicts,
    branch,
    isRepo,
    changedFilesCount,
    hasRemote,
    canPush,
    canPull,
    // Actions
    loadStatus,
    commit,
    push,
    fetchRemote,
    checkConflicts,
    loadHistory,
    loadWorkingDiff,
    loadCommitDiff,
    loadRemote,
    clearSelectedCommit,
    togglePanel,
    clearError
  }
})
