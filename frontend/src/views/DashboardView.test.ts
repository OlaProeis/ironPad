// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import DashboardView from './DashboardView.vue'
import { githubApi, projectsApi, tasksApi } from '../api/client'
import type { GithubSummaryEntry, Project } from '../types'

vi.mock('../api/client', () => ({
  projectsApi: {
    list: vi.fn(),
    create: vi.fn(),
  },
  tasksApi: {
    listAll: vi.fn(),
  },
  githubApi: {
    getSummary: vi.fn(),
  },
}))

function project(overrides: Partial<Project> = {}): Project {
  return {
    id: 'alpha',
    name: 'Alpha',
    path: 'data/projects/alpha',
    created: '2026-01-01T00:00:00Z',
    ...overrides,
  }
}

async function mountDashboard() {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/', name: 'dashboard', component: DashboardView },
      {
        path: '/projects/:id/github',
        name: 'project-github',
        component: { template: '<div />' },
      },
    ],
  })
  await router.push('/')
  await router.isReady()

  const wrapper = mount(DashboardView, {
    global: { plugins: [router] },
  })
  await flushPromises()
  return wrapper
}

describe('DashboardView', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.mocked(projectsApi.list).mockResolvedValue([project()])
    vi.mocked(tasksApi.listAll).mockResolvedValue([])
  })

  it('shows GitHub summary link when assigned and review counts are positive', async () => {
    const entry: GithubSummaryEntry = {
      project_id: 'alpha',
      repo: 'org/alpha',
      assigned_issues: 2,
      review_pulls: 1,
    }
    vi.mocked(githubApi.getSummary).mockResolvedValue([entry])

    const wrapper = await mountDashboard()
    const link = wrapper.find('a.github-summary-link')
    expect(link.exists()).toBe(true)
    expect(link.text()).toBe('2 assigned · 1 reviews')
  })

  it('hides GitHub summary link when both counts are zero', async () => {
    const entry: GithubSummaryEntry = {
      project_id: 'alpha',
      repo: 'org/alpha',
      assigned_issues: 0,
      review_pulls: 0,
    }
    vi.mocked(githubApi.getSummary).mockResolvedValue([entry])

    const wrapper = await mountDashboard()
    expect(wrapper.find('a.github-summary-link').exists()).toBe(false)
  })

  it('renders without error when getSummary rejects', async () => {
    vi.mocked(githubApi.getSummary).mockRejectedValue(new Error('network'))

    const wrapper = await mountDashboard()
    expect(wrapper.find('.dashboard-grid').exists()).toBe(true)
    expect(wrapper.find('a.github-summary-link').exists()).toBe(false)
  })
})