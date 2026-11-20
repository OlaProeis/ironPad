// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import type { BrowseResponse, Prompt } from '../../../types'
import SuggestFilesPanel from '../SuggestFilesPanel.vue'
import { libraryApi } from '../../../api/client'
import { setLibraryRoot } from '../../../lib/libraryRoot'
import { usePromptsStore } from '../../../stores/prompts'

vi.mock('../../../api/client', () => ({
  libraryApi: {
    browseLibrary: vi.fn(),
  },
  promptsApi: {
    list: vi.fn().mockResolvedValue([]),
    folders: vi.fn().mockResolvedValue([]),
    get: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    semanticSearch: vi.fn().mockResolvedValue([]),
  },
}))

const rootListing: BrowseResponse = {
  path: '/repo',
  parent: null,
  entries: [
    { name: 'ai-context.md', path: '/repo/ai-context.md', is_dir: false, size: 1, linkable: true },
    { name: 'ROADMAP.md', path: '/repo/ROADMAP.md', is_dir: false, size: 1, linkable: true },
  ],
}

function createdLink(title: string, source: string): Prompt {
  return {
    id: 'new',
    title,
    path: 'data/prompts/new.md',
    scope: 'project',
    folder: 'root',
    tags: [],
    kind: 'link',
    source,
    template: false,
    frontmatter: {},
    content: '',
    read_only: true,
    source_missing: false,
  }
}

describe('SuggestFilesPanel', () => {
  beforeEach(() => {
    localStorage.clear()
    setActivePinia(createPinia())
    vi.mocked(libraryApi.browseLibrary).mockReset()
    vi.mocked(libraryApi.browseLibrary).mockImplementation(async (path: string) => {
      if (path === '/repo') return rootListing
      if (path === '/repo/docs') {
        return { path: '/repo/docs', parent: '/repo', entries: [] }
      }
      throw new Error(path)
    })
  })

  async function mountPanel() {
    setLibraryRoot('p1', '/repo')
    const pinia = createPinia()
    setActivePinia(pinia)
    const wrapper = mount(SuggestFilesPanel, {
      props: { projectId: 'p1' },
      global: { plugins: [pinia] },
    })
    await flushPromises()
    return wrapper
  }

  it('renders suggestions and adds selected links', async () => {
    const wrapper = await mountPanel()
    expect(wrapper.find('[data-testid="suggest-files-panel"]').exists()).toBe(true)
    const boxes = wrapper.findAll('[data-testid="suggest-file-checkbox"]')
    expect(boxes).toHaveLength(2)

    const create = vi
      .spyOn(usePromptsStore(), 'createPrompt')
      .mockImplementation(async (payload) =>
        createdLink(payload.title ?? '', payload.source ?? '')
      )

    await wrapper.get('[data-testid="suggest-add-selected"]').trigger('click')
    await flushPromises()

    expect(create).toHaveBeenCalledTimes(2)
    expect(create).toHaveBeenCalledWith({
      kind: 'link',
      title: 'ai-context.md',
      folder: undefined,
      tags: [],
      template: false,
      source: 'ai-context.md',
    })
    expect(create).toHaveBeenCalledWith({
      kind: 'link',
      title: 'ROADMAP.md',
      folder: undefined,
      tags: [],
      template: false,
      source: 'ROADMAP.md',
    })
  })

  it('dismiss hides the panel and records dismissal', async () => {
    const wrapper = await mountPanel()
    await wrapper.get('[data-testid="suggest-dismiss"]').trigger('click')
    await flushPromises()

    expect(wrapper.find('[data-testid="suggest-files-panel"]').exists()).toBe(false)
    expect(localStorage.getItem('ironpad-library-suggest-dismissed:p1')).toBe('1')
  })
})