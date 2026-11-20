// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import TrackButton from './TrackButton.vue'
import { githubApi } from '../../api/client'

const pushMock = vi.hoisted(() => vi.fn())

vi.mock('vue-router', () => ({
  useRouter: () => ({ push: pushMock }),
}))

vi.mock('../../api/client', () => ({
  githubApi: {
    trackIssue: vi.fn(),
    trackPull: vi.fn(),
  },
}))

const PROJECT_ID = 'proj-1'
const ISSUE_NUMBER = 42
const TASK_ID = 'task-abc'

function mountButton(kind: 'issue' | 'pull' = 'issue') {
  return mount(TrackButton, {
    props: {
      projectId: PROJECT_ID,
      number: ISSUE_NUMBER,
      kind,
    },
  })
}

describe('TrackButton', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('clicking Track as task calls githubApi.trackIssue', async () => {
    vi.mocked(githubApi.trackIssue).mockResolvedValue({
      task_id: TASK_ID,
      created: true,
    })

    const wrapper = mountButton('issue')
    await wrapper.get('[data-testid="track-as-task"]').trigger('click')
    await flushPromises()

    expect(githubApi.trackIssue).toHaveBeenCalledWith(PROJECT_ID, ISSUE_NUMBER)
    expect(githubApi.trackPull).not.toHaveBeenCalled()
  })

  it('on created=true response, button label changes to Open task', async () => {
    vi.mocked(githubApi.trackIssue).mockResolvedValue({
      task_id: TASK_ID,
      created: true,
    })

    const wrapper = mountButton('issue')
    await wrapper.get('[data-testid="track-as-task"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="track-as-task"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="open-task"]').text()).toBe('Open task')
  })

  it('on created=false response, button also shows Open task', async () => {
    vi.mocked(githubApi.trackIssue).mockResolvedValue({
      task_id: TASK_ID,
      created: false,
    })

    const wrapper = mountButton('issue')
    await wrapper.get('[data-testid="track-as-task"]').trigger('click')
    await flushPromises()

    expect(wrapper.get('[data-testid="open-task"]').text()).toBe('Open task')
  })

  it('clicking Open task calls router.push with project-tasks route and task query', async () => {
    vi.mocked(githubApi.trackIssue).mockResolvedValue({
      task_id: TASK_ID,
      created: true,
    })

    const wrapper = mountButton('issue')
    await wrapper.get('[data-testid="track-as-task"]').trigger('click')
    await flushPromises()

    await wrapper.get('[data-testid="open-task"]').trigger('click')

    expect(pushMock).toHaveBeenCalledWith({
      name: 'project-tasks',
      params: { id: PROJECT_ID },
      query: { task: TASK_ID },
    })
  })

  it("kind='pull' calls githubApi.trackPull", async () => {
    vi.mocked(githubApi.trackPull).mockResolvedValue({
      task_id: TASK_ID,
      created: true,
    })

    const wrapper = mountButton('pull')
    await wrapper.get('[data-testid="track-as-task"]').trigger('click')
    await flushPromises()

    expect(githubApi.trackPull).toHaveBeenCalledWith(PROJECT_ID, ISSUE_NUMBER)
    expect(githubApi.trackIssue).not.toHaveBeenCalled()
  })
})