import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { GithubIssue, GithubPull, GithubSnapshotResponse } from '../types'
import { githubApi } from '../api/client'

export type GithubIssueFilter = 'all' | 'assigned' | 'created'
export type GithubPrFilter = 'all' | 'mine' | 'review'

const ISSUE_FILTER_VALUES: GithubIssueFilter[] = ['all', 'assigned', 'created']
const PR_FILTER_VALUES: GithubPrFilter[] = ['all', 'mine', 'review']

function issueFilterKey(projectId: string): string {
  return `ironpad-github-issue-filter:${projectId}`
}

function prFilterKey(projectId: string): string {
  return `ironpad-github-pr-filter:${projectId}`
}

function parseIssueFilter(raw: string | null): GithubIssueFilter {
  if (raw && ISSUE_FILTER_VALUES.includes(raw as GithubIssueFilter)) {
    return raw as GithubIssueFilter
  }
  return 'all'
}

function parsePrFilter(raw: string | null): GithubPrFilter {
  if (raw && PR_FILTER_VALUES.includes(raw as GithubPrFilter)) {
    return raw as GithubPrFilter
  }
  return 'all'
}

function loginsMatch(a: string, b: string): boolean {
  return a.toLowerCase() === b.toLowerCase()
}

function includesLogin(logins: string[], login: string): boolean {
  const lower = login.toLowerCase()
  return logins.some((l) => l.toLowerCase() === lower)
}

export const useGithubStore = defineStore('github', () => {
  const snapshot = ref<GithubSnapshotResponse | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const issueFilter = ref<GithubIssueFilter>('all')
  const prFilter = ref<GithubPrFilter>('all')

  const hasLogin = computed(() => Boolean(snapshot.value?.login))

  const filteredIssues = computed((): GithubIssue[] => {
    const issues = snapshot.value?.issues ?? []
    const login = snapshot.value?.login
    if (!login) {
      return issues
    }
    switch (issueFilter.value) {
      case 'assigned':
        return issues.filter((i) => includesLogin(i.assignee_logins, login))
      case 'created':
        return issues.filter((i) => loginsMatch(i.author_login, login))
      default:
        return issues
    }
  })

  const filteredPulls = computed((): GithubPull[] => {
    const pulls = snapshot.value?.pulls ?? []
    const login = snapshot.value?.login
    if (!login) {
      return pulls
    }
    switch (prFilter.value) {
      case 'mine':
        return pulls.filter((p) => loginsMatch(p.author_login, login))
      case 'review':
        return pulls.filter((p) => includesLogin(p.requested_reviewer_logins, login))
      default:
        return pulls
    }
  })

  function initFilters(projectId: string) {
    try {
      issueFilter.value = parseIssueFilter(localStorage.getItem(issueFilterKey(projectId)))
      prFilter.value = parsePrFilter(localStorage.getItem(prFilterKey(projectId)))
    } catch {
      issueFilter.value = 'all'
      prFilter.value = 'all'
    }
  }

  function setIssueFilter(projectId: string, filter: GithubIssueFilter) {
    issueFilter.value = filter
    try {
      localStorage.setItem(issueFilterKey(projectId), filter)
    } catch {
      // private mode / blocked storage
    }
  }

  function setPrFilter(projectId: string, filter: GithubPrFilter) {
    prFilter.value = filter
    try {
      localStorage.setItem(prFilterKey(projectId), filter)
    } catch {
      // private mode / blocked storage
    }
  }

  function errorText(err: unknown): string {
    return err instanceof Error ? err.message : String(err)
  }

  async function loadSnapshot(projectId: string) {
    try {
      loading.value = true
      error.value = null
      initFilters(projectId)
      snapshot.value = await githubApi.getSnapshot(projectId)
    } catch (err) {
      error.value = `Failed to load GitHub snapshot: ${errorText(err)}`
    } finally {
      loading.value = false
    }
  }

  async function refresh(projectId: string) {
    try {
      loading.value = true
      error.value = null
      snapshot.value = await githubApi.refresh(projectId)
    } catch (err) {
      error.value = `Failed to refresh GitHub snapshot: ${errorText(err)}`
    } finally {
      loading.value = false
    }
  }

  return {
    snapshot,
    loading,
    error,
    issueFilter,
    prFilter,
    hasLogin,
    filteredIssues,
    filteredPulls,
    initFilters,
    setIssueFilter,
    setPrFilter,
    loadSnapshot,
    refresh,
  }
})