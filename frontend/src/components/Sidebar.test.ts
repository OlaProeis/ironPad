// @vitest-environment happy-dom
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createRouter, createMemoryHistory } from 'vue-router'
import Sidebar from './Sidebar.vue'
import { useWorkspaceStore } from '../stores/workspace'

vi.mock('../api/client', () => ({
  tasksApi: {
    list: vi.fn().mockResolvedValue([]),
  },
}))

describe('Sidebar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('shows Library for project prompts navigation', async () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: '/projects/:id/prompts',
          name: 'project-prompts',
          component: { template: '<div />' },
        },
      ],
    })
    await router.push('/projects/test-project/prompts')
    await router.isReady()

    const workspace = useWorkspaceStore()
    workspace.activeProjectId = 'test-project'
    workspace.activeProject = {
      id: 'test-project',
      name: 'Test',
      path: '/tmp/test',
      created: '2026-01-01T00:00:00Z',
    }

    const wrapper = mount(Sidebar, {
      global: {
        plugins: [router],
        stubs: {
          SearchPanel: true,
          TaskPanel: true,
          GitStatus: true,
        },
      },
    })
    await flushPromises()

    const navButtons = wrapper.findAll('.project-nav .nav-item')
    const libraryButton = navButtons.find((btn) => btn.text() === 'Library')
    expect(libraryButton).toBeDefined()
    expect(libraryButton?.classes()).toContain('nav-item')
  })

  it('shows GitHub navigation when a project is active', async () => {
    const router = createRouter({
      history: createMemoryHistory(),
      routes: [
        {
          path: '/projects/:id/github',
          name: 'project-github',
          component: { template: '<div />' },
        },
      ],
    })
    await router.push('/projects/test-project/github')
    await router.isReady()

    const workspace = useWorkspaceStore()
    workspace.activeProjectId = 'test-project'
    workspace.activeProject = {
      id: 'test-project',
      name: 'Test',
      path: '/tmp/test',
      created: '2026-01-01T00:00:00Z',
    }

    const wrapper = mount(Sidebar, {
      global: {
        plugins: [router],
        stubs: {
          SearchPanel: true,
          TaskPanel: true,
          GitStatus: true,
        },
      },
    })
    await flushPromises()

    expect(wrapper.html()).toContain('★ GitHub')
    const githubButton = wrapper
      .findAll('.project-nav .nav-item')
      .find((btn) => btn.text().includes('GitHub'))
    expect(githubButton).toBeDefined()
  })
})