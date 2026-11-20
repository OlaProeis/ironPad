// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import PullsList from './PullsList.vue'
import { useGithubStore } from '../../stores/github'
import type { GithubPull, GithubSnapshotResponse } from '../../types'

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

function pull(
  overrides: Partial<GithubPull> & Pick<GithubPull, 'number' | 'title'>,
): GithubPull {
  return {
    html_url: `https://github.com/o/r/pull/${overrides.number}`,
    updated_at: '2026-01-01T00:00:00Z',
    author_login: 'bob',
    labels: [],
    comments_count: 0,
    draft: false,
    requested_reviewer_logins: [],
    head_sha: 'abc123',
    ...overrides,
  }
}

function mountList() {
  return mount(PullsList, {
    props: { projectId: PROJECT_ID },
  })
}

describe('PullsList', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('draft=true renders the Draft badge', () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      pulls: [pull({ number: 7, title: 'WIP change', draft: true })],
    })
    store.prFilter = 'all'

    const wrapper = mountList()
    const row = wrapper.get('[data-testid="pr-row-7"]')

    expect(row.find('[data-testid="pr-draft-badge"]').exists()).toBe(true)
    expect(row.find('[data-testid="pr-draft-badge"]').text()).toBe('Draft')
  })

  it("filter='review' with login='alice' shows only PRs where alice is a requested reviewer", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      login: 'alice',
      pulls: [
        pull({
          number: 1,
          title: 'Needs alice',
          requested_reviewer_logins: ['alice'],
        }),
        pull({
          number: 2,
          title: 'Needs bob',
          requested_reviewer_logins: ['bob'],
        }),
      ],
    })
    store.prFilter = 'review'

    const wrapper = mountList()

    expect(wrapper.find('[data-testid="pr-row-1"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="pr-row-2"]').exists()).toBe(false)
  })

  it("with hasLogin=false, 'mine' and 'review' tabs absent", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      pulls: [pull({ number: 1, title: 'Only' })],
    })
    store.prFilter = 'all'

    const wrapper = mountList()

    expect(wrapper.find('[data-testid="pr-filter-all"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="pr-filter-mine"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="pr-filter-review"]').exists()).toBe(false)
  })

  it("empty list shows 'No open pull requests'", () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({ login: 'alice', pulls: [] })
    store.prFilter = 'all'

    const wrapper = mountList()

    expect(wrapper.get('[data-testid="pulls-empty"]').text()).toBe('No open pull requests')
  })

  it('pulls_truncated=true shows the notice', () => {
    const store = useGithubStore()
    store.snapshot = baseSnapshot({
      pulls_truncated: true,
      pulls: [pull({ number: 1, title: 'One' })],
    })

    const wrapper = mountList()

    expect(wrapper.get('[data-testid="pulls-truncated-notice"]').text()).toBe(
      'Showing the 500 most recently updated open items.',
    )
  })
})