// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import IssuesList from './IssuesList.vue'
import { useGithubStore } from '../../stores/github'
import type { GithubIssue, GithubSnapshotResponse } from '../../types'
import { relativeTime } from '../../lib/relativeTime'

const PROJECT_ID = 'proj-1'

function baseSnapshot(
  overrides: Partial<GithubSnapshotResponse> = {},
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

function issue(overrides: Partial<GithubIssue> & Pick<GithubIssue, 'number' | 'title'>): GithubIssue {
  return {
    html_url: `https://github.com/o/r/issues/${overrides.number}`,
    updated_at: '2026-01-01T00:00:00Z',
    author_login: 'bob',
    labels: [],
    comments_count: 0,
    assignee_logins: [],
    ...overrides,
  }
}

function mountList() {
  return mount(IssuesList, {
    props: { projectId: PROJECT_ID },
  })
}

describe('IssuesList', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it("with two issues and filter='all', both rows are rendered", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      login: 'alice',
      issues: [
        issue({ number: 1, title: 'Alpha' }),
        issue({ number: 2, title: 'Beta' }),
      ],
    })
    store.issueFilter = 'all'

    const wrapper = mountList()

    expect(wrapper.find('[data-testid="issue-row-1"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="issue-row-2"]').exists()).toBe(true)
  })

  it("with filter='assigned' and login='alice', only assigned issue is shown", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      login: 'alice',
      issues: [
        issue({ number: 1, title: 'Mine', assignee_logins: ['alice'] }),
        issue({ number: 2, title: 'Other', assignee_logins: ['bob'] }),
      ],
    })
    store.issueFilter = 'assigned'

    const wrapper = mountList()

    expect(wrapper.find('[data-testid="issue-row-1"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="issue-row-2"]').exists()).toBe(false)
  })

  it("with hasLogin=false, 'assigned' and 'created' tab buttons are absent", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({ issues: [issue({ number: 1, title: 'Only' })] })
    store.issueFilter = 'all'

    const wrapper = mountList()

    expect(wrapper.find('[data-testid="issue-filter-all"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="issue-filter-assigned"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="issue-filter-created"]').exists()).toBe(false)
  })

  it("empty filtered list shows 'No open issues'", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({ login: 'alice', issues: [] })
    store.issueFilter = 'all'

    const wrapper = mountList()

    expect(wrapper.get('[data-testid="issues-empty"]').text()).toBe('No open issues')
  })

  it('issues_truncated=true shows the 500-item notice', () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      issues_truncated: true,
      issues: [issue({ number: 1, title: 'One' })],
    })

    const wrapper = mountList()

    expect(wrapper.get('[data-testid="issues-truncated-notice"]').text()).toBe(
      'Showing the 500 most recently updated open items.',
    )
  })

  it("relativeTime('2026-11-10T10:00:00Z') returns a non-empty string without throwing", () => {
    expect(() => relativeTime('2026-11-10T10:00:00Z')).not.toThrow()
    expect(relativeTime('2026-11-10T10:00:00Z').length).toBeGreaterThan(0)
  })
})