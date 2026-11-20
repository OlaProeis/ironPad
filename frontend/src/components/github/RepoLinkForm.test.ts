// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import RepoLinkForm from './RepoLinkForm.vue'
import { githubApi } from '../../api/client'
import { useGithubStore } from '../../stores/github'
import type { GithubSnapshotResponse } from '../../types'

vi.mock('../../api/client', () => ({
  githubApi: {
    putRepo: vi.fn(),
    refresh: vi.fn(),
    getSnapshot: vi.fn(),
  },
}))

const PROJECT_ID = 'proj-1'
const VALIDATION_MSG = 'Use owner/name, with no URL and no .git suffix.'

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

function mountForm() {
  return mount(RepoLinkForm, {
    props: { projectId: PROJECT_ID },
  })
}

describe('RepoLinkForm', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('explains how to sign in before a repository can be read', () => {
    const wrapper = mountForm()
    const setup = wrapper.get('[data-testid="repo-link-setup"]').text()
    expect(setup).toContain('gh auth login')
    expect(setup).toContain('GH_TOKEN')
    expect(setup).toContain('owner/name')
  })

  it("shows validation error when submitting 'https://github.com/a/b'", async () => {
    const wrapper = mountForm()
    await wrapper.get('[data-testid="repo-link-input"]').setValue('https://github.com/a/b')
    await wrapper.get('[data-testid="repo-link-save"]').trigger('click')

    expect(wrapper.get('[data-testid="repo-link-error"]').text()).toBe(VALIDATION_MSG)
    expect(githubApi.putRepo).not.toHaveBeenCalled()
  })

  it("shows validation error when submitting 'owner/repo.git'", async () => {
    const wrapper = mountForm()
    await wrapper.get('[data-testid="repo-link-input"]').setValue('owner/repo.git')
    await wrapper.get('[data-testid="repo-link-save"]').trigger('click')

    expect(wrapper.get('[data-testid="repo-link-error"]').text()).toBe(VALIDATION_MSG)
    expect(githubApi.putRepo).not.toHaveBeenCalled()
  })

  it("calls putRepo with 'owner/repo' on valid save", async () => {
    vi.mocked(githubApi.putRepo).mockResolvedValue(undefined)
    vi.mocked(githubApi.refresh).mockResolvedValue(baseSnapshot({ repo: 'owner/repo' }))

    const wrapper = mountForm()
    await wrapper.get('[data-testid="repo-link-input"]').setValue('owner/repo')
    await wrapper.get('[data-testid="repo-link-save"]').trigger('click')
    await flushPromises()

    expect(githubApi.putRepo).toHaveBeenCalledWith(PROJECT_ID, 'owner/repo')
  })

  it('refreshes store and emits saved after putRepo resolves', async () => {
    const refreshed = baseSnapshot({ repo: 'owner/repo' })
    vi.mocked(githubApi.putRepo).mockResolvedValue(undefined)
    vi.mocked(githubApi.refresh).mockResolvedValue(refreshed)

    const wrapper = mountForm()
    await wrapper.get('[data-testid="repo-link-input"]').setValue('owner/repo')
    await wrapper.get('[data-testid="repo-link-save"]').trigger('click')
    await flushPromises()

    expect(githubApi.refresh).toHaveBeenCalledWith(PROJECT_ID)
    expect(useGithubStore().snapshot).toEqual(refreshed)
    expect(wrapper.emitted('saved')).toHaveLength(1)
  })

  it('remove flow confirms, clears repo, and emits removed', async () => {
    vi.mocked(githubApi.putRepo).mockResolvedValue(undefined)

    const store = useGithubStore()
    store.snapshot = baseSnapshot({ repo: 'org/repo' })

    const wrapper = mountForm()
    await flushPromises()

    await wrapper.get('[data-testid="repo-link-remove"]').trigger('click')
    expect(wrapper.find('[data-testid="repo-link-remove-confirm"]').exists()).toBe(true)

    await wrapper.get('[data-testid="repo-link-remove-confirm-btn"]').trigger('click')
    await flushPromises()

    expect(githubApi.putRepo).toHaveBeenCalledWith(PROJECT_ID, null)
    expect(store.snapshot).toBeNull()
    expect(wrapper.emitted('removed')).toHaveLength(1)
  })
})