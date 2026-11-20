// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import type { GithubSnapshotResponse } from '../../types'
import { useGithubStore } from '../github'
import { githubApi } from '../../api/client'

vi.mock('../../api/client', () => ({
  githubApi: {
    getSnapshot: vi.fn(),
    refresh: vi.fn(),
    putRepo: vi.fn(),
    getIssue: vi.fn(),
    getPull: vi.fn(),
    trackIssue: vi.fn(),
    trackPull: vi.fn(),
    getSummary: vi.fn(),
  },
}))

const PROJECT_ID = 'proj-1'

function baseSnapshot(
  overrides: Partial<GithubSnapshotResponse> = {}
): GithubSnapshotResponse {
  return {
    stale: false,
    issues: [],
    pulls: [],
    issues_truncated: false,
    pulls_truncated: false,
    ...overrides,
  }
}

describe('github store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    localStorage.clear()
  })

  it('loadSnapshot stores the response from a mocked githubApi', async () => {
    const stub = baseSnapshot({ repo: 'org/repo', login: 'bob' })
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(stub)

    const store = useGithubStore()
    await store.loadSnapshot(PROJECT_ID)

    expect(githubApi.getSnapshot).toHaveBeenCalledWith(PROJECT_ID)
    expect(store.snapshot).toEqual(stub)
    expect(store.loading).toBe(false)
    expect(store.error).toBeNull()
  })

  it("issueFilter assigned keeps issues where assignee matches login (case-insensitive)", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      login: 'alice',
      issues: [
        {
          number: 1,
          title: 'One',
          html_url: 'https://github.com/o/r/issues/1',
          updated_at: '2020-01-01',
          author_login: 'bob',
          labels: [],
          comments_count: 0,
          assignee_logins: ['Alice'],
        },
        {
          number: 2,
          title: 'Two',
          html_url: 'https://github.com/o/r/issues/2',
          updated_at: '2020-01-01',
          author_login: 'carol',
          labels: [],
          comments_count: 0,
          assignee_logins: [],
        },
      ],
    })
    store.issueFilter = 'assigned'

    expect(store.filteredIssues).toHaveLength(1)
    expect(store.filteredIssues[0]?.number).toBe(1)
  })

  it("issueFilter assigned with no login returns all issues", () => {
    const store = useGithubStore()
    const issues = [
      {
        number: 1,
        title: 'One',
        html_url: 'https://github.com/o/r/issues/1',
        updated_at: '2020-01-01',
        author_login: 'bob',
        labels: [],
        comments_count: 0,
        assignee_logins: ['Alice'],
      },
      {
        number: 2,
        title: 'Two',
        html_url: 'https://github.com/o/r/issues/2',
        updated_at: '2020-01-01',
        author_login: 'carol',
        labels: [],
        comments_count: 0,
        assignee_logins: [],
      },
    ]
    store.snapshot = baseSnapshot({ issues })
    store.issueFilter = 'assigned'

    expect(store.filteredIssues).toHaveLength(2)
    expect(store.filteredIssues).toEqual(issues)
  })

  it('setIssueFilter writes to localStorage and initFilters reads it back', () => {
    const store = useGithubStore()
    store.setIssueFilter(PROJECT_ID, 'created')
    expect(localStorage.getItem(`ironpad-github-issue-filter:${PROJECT_ID}`)).toBe(
      'created'
    )

    store.issueFilter = 'all'
    store.initFilters(PROJECT_ID)
    expect(store.issueFilter).toBe('created')
  })

  it('unknown localStorage filter value falls back to all', () => {
    localStorage.setItem(`ironpad-github-issue-filter:${PROJECT_ID}`, 'bogus')
    localStorage.setItem(`ironpad-github-pr-filter:${PROJECT_ID}`, 'nope')

    const store = useGithubStore()
    store.initFilters(PROJECT_ID)

    expect(store.issueFilter).toBe('all')
    expect(store.prFilter).toBe('all')
  })
})