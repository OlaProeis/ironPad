// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import GitHubView from './GitHubView.vue'
import RepoLinkForm from '../components/github/RepoLinkForm.vue'
import { githubApi } from '../api/client'
import { useGithubStore } from '../stores/github'
import type { GithubSnapshotResponse } from '../types'

vi.mock('../api/client', () => ({
  githubApi: {
    getSnapshot: vi.fn(),
    refresh: vi.fn(),
  },
}))

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

const FIFTEEN_MINUTES_MS = 15 * 60 * 1000

describe('GitHubView scaffold', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.useRealTimers()
  })

  it('renders RepoLinkForm when snapshot has no repo', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(baseSnapshot())
    vi.mocked(githubApi.refresh).mockResolvedValue(baseSnapshot())

    const wrapper = mount(GitHubView, {
      props: { id: 'proj-1' },
    })
    await flushPromises()

    expect(wrapper.find('[data-testid="repo-link-form"]').exists()).toBe(true)
    expect(wrapper.getComponent(RepoLinkForm).props('projectId')).toBe('proj-1')
    expect(wrapper.find('[data-testid="github-refresh-btn"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="issues-list"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="pulls-list"]').exists()).toBe(false)
  })

  it('does not refresh a stale snapshot when no repository is linked', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(baseSnapshot({ stale: true }))
    vi.mocked(githubApi.refresh).mockResolvedValue(baseSnapshot({ stale: true }))

    mount(GitHubView, { props: { id: 'proj-1' } })
    await flushPromises()

    expect(githubApi.refresh).not.toHaveBeenCalled()
  })

  it('renders list placeholders when snapshot has a repo', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo' }),
    )
    vi.mocked(githubApi.refresh).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo' }),
    )

    const wrapper = mount(GitHubView, {
      props: { id: 'proj-1' },
    })
    await flushPromises()

    expect(wrapper.find('[data-testid="repo-link-form"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="issues-list"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="pulls-list"]').exists()).toBe(true)
  })
})

describe('GitHubView refresh triggers', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.useFakeTimers()
    vi.mocked(githubApi.refresh).mockImplementation(async () =>
      baseSnapshot({ repo: 'org/repo', stale: false }),
    )
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('on mount with stale snapshot calls refresh once', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo', stale: true }),
    )

    mount(GitHubView, { props: { id: 'proj-1' } })
    await flushPromises()

    expect(githubApi.getSnapshot).toHaveBeenCalledWith('proj-1')
    expect(githubApi.refresh).toHaveBeenCalledTimes(1)
    expect(githubApi.refresh).toHaveBeenCalledWith('proj-1')
  })

  it('on mount with fresh snapshot does not call refresh', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo', stale: false }),
    )

    mount(GitHubView, { props: { id: 'proj-1' } })
    await flushPromises()

    expect(githubApi.refresh).not.toHaveBeenCalled()
  })

  it('clicking Refresh calls refresh', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo', stale: false }),
    )

    const wrapper = mount(GitHubView, { props: { id: 'proj-1' } })
    await flushPromises()
    vi.mocked(githubApi.refresh).mockClear()

    await wrapper.get('[data-testid="github-refresh-btn"]').trigger('click')
    await flushPromises()

    expect(githubApi.refresh).toHaveBeenCalledTimes(1)
    expect(githubApi.refresh).toHaveBeenCalledWith('proj-1')
  })

  it('advancing fake timer by 15 minutes calls refresh again', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo', stale: false }),
    )

    mount(GitHubView, { props: { id: 'proj-1' } })
    await flushPromises()
    vi.mocked(githubApi.refresh).mockClear()

    await vi.advanceTimersByTimeAsync(FIFTEEN_MINUTES_MS)

    expect(githubApi.refresh).toHaveBeenCalledTimes(1)
    expect(githubApi.refresh).toHaveBeenCalledWith('proj-1')
  })

  it('onUnmounted clears the interval so timer advance does not refresh', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo', stale: false }),
    )

    const wrapper = mount(GitHubView, { props: { id: 'proj-1' } })
    await flushPromises()
    vi.mocked(githubApi.refresh).mockClear()

    wrapper.unmount()
    await vi.advanceTimersByTimeAsync(FIFTEEN_MINUTES_MS)

    expect(githubApi.refresh).not.toHaveBeenCalled()
  })

  it('shows error banner when githubStore.error is set', async () => {
    vi.mocked(githubApi.getSnapshot).mockResolvedValue(
      baseSnapshot({ repo: 'org/repo', stale: false }),
    )

    const wrapper = mount(GitHubView, { props: { id: 'proj-1' } })
    await flushPromises()

    const store = useGithubStore()
    store.error = 'GitHub API unavailable'
    await wrapper.vm.$nextTick()

    expect(wrapper.find('[data-testid="github-error-banner"]').exists()).toBe(
      true,
    )
    expect(wrapper.find('[data-testid="github-error-banner"]').text()).toContain(
      'GitHub API unavailable',
    )
  })
})