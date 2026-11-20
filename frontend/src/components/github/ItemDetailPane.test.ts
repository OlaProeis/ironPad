// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import ItemDetailPane from './ItemDetailPane.vue'
import { githubApi } from '../../api/client'
import type { GithubIssue, GithubItemDetail, GithubPull } from '../../types'

vi.mock('../../api/client', () => ({
  githubApi: {
    getIssue: vi.fn(),
    getPull: vi.fn(),
  },
}))

function detail(overrides: Partial<GithubItemDetail> = {}): GithubItemDetail {
  return {
    number: 7,
    title: 'Broken button',
    html_url: 'https://github.com/o/r/issues/7',
    body: 'Hello **world**',
    body_too_large: false,
    comments: [],
    more_comments: false,
    ...overrides,
  }
}

function issue(): GithubIssue {
  return {
    number: 7,
    title: 'Broken button',
    html_url: 'https://github.com/o/r/issues/7',
    updated_at: '2026-01-01T00:00:00Z',
    author_login: 'ada',
    labels: [],
    comments_count: 0,
    assignee_logins: [],
  }
}

function pull(): GithubPull {
  return {
    number: 9,
    title: 'Fix button',
    html_url: 'https://github.com/o/r/pull/9',
    updated_at: '2026-01-02T00:00:00Z',
    author_login: 'ada',
    labels: [],
    comments_count: 1,
    draft: false,
    requested_reviewer_logins: [],
    head_sha: 'abc',
  }
}

describe('ItemDetailPane', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('calls githubApi.getIssue when mounted with an issue', async () => {
    vi.mocked(githubApi.getIssue).mockResolvedValue(detail())

    mount(ItemDetailPane, {
      props: { projectId: 'proj-1', item: issue(), kind: 'issue' },
    })
    await flushPromises()

    expect(githubApi.getIssue).toHaveBeenCalledWith('proj-1', 7)
    expect(githubApi.getPull).not.toHaveBeenCalled()
  })

  it('shows the too-large message and no RenderedMarkdown when body_too_large is true', async () => {
    vi.mocked(githubApi.getIssue).mockResolvedValue(
      detail({ body_too_large: true, body: '<script>evil</script>', comments: [] }),
    )

    const wrapper = mount(ItemDetailPane, {
      props: { projectId: 'proj-1', item: issue(), kind: 'issue' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="body-too-large"]').text()).toBe(
      'The body is too large to display',
    )
    expect(wrapper.find('[data-testid="rendered-markdown"]').exists()).toBe(false)
  })

  it('shows the earlier-comments notice when more_comments is true', async () => {
    vi.mocked(githubApi.getIssue).mockResolvedValue(detail({ more_comments: true }))

    const wrapper = mount(ItemDetailPane, {
      props: { projectId: 'proj-1', item: issue(), kind: 'issue' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="more-comments"]').text()).toBe(
      'Earlier comments are on GitHub',
    )
  })

  it("shows the status line for a pull with commit_status 'failure'", async () => {
    vi.mocked(githubApi.getPull).mockResolvedValue(
      detail({
        number: 9,
        html_url: 'https://github.com/o/r/pull/9',
        commit_status: 'failure',
      }),
    )

    const wrapper = mount(ItemDetailPane, {
      props: { projectId: 'proj-1', item: pull(), kind: 'pull' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="commit-status"]').text()).toContain('Status: failure')
    expect(wrapper.find('.status-dot--failure').exists()).toBe(true)
  })

  it('sets the Open on GitHub href from item.html_url', async () => {
    vi.mocked(githubApi.getIssue).mockResolvedValue(detail())

    const wrapper = mount(ItemDetailPane, {
      props: { projectId: 'proj-1', item: issue(), kind: 'issue' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="open-on-github"]').attributes('href')).toBe(
      'https://github.com/o/r/issues/7',
    )
  })

  it('shows the error line when detail.error is set', async () => {
    vi.mocked(githubApi.getIssue).mockResolvedValue(
      detail({ error: 'GitHub rate limit exceeded' }),
    )

    const wrapper = mount(ItemDetailPane, {
      props: { projectId: 'proj-1', item: issue(), kind: 'issue' },
    })
    await flushPromises()

    expect(wrapper.get('[data-testid="detail-error"]').text()).toContain(
      'GitHub rate limit exceeded',
    )
  })

  it('does not render a script element when the body contains a script tag', async () => {
    vi.mocked(githubApi.getIssue).mockResolvedValue(
      detail({ body: 'See <script>evil</script> please' }),
    )

    const wrapper = mount(ItemDetailPane, {
      props: { projectId: 'proj-1', item: issue(), kind: 'issue' },
    })
    await flushPromises()

    expect(wrapper.html().toLowerCase()).not.toContain('<script')
    expect(wrapper.find('[data-testid="rendered-markdown"]').exists()).toBe(true)
  })
})
