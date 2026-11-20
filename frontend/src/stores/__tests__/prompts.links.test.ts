// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import type { Prompt } from '../../types'
import { usePromptsStore } from '../prompts'
import { promptsApi } from '../../api/client'
import { setLibraryRoot } from '../../lib/libraryRoot'

vi.mock('../../api/client', () => ({
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

function basePrompt(overrides: Partial<Prompt> = {}): Prompt {
  return {
    id: 'p1',
    title: 'T',
    path: 'data/prompts/p1.md',
    scope: 'project',
    project_id: 'proj1',
    folder: 'root',
    tags: [],
    kind: 'prompt',
    template: false,
    frontmatter: {},
    content: 'body',
    read_only: false,
    source_missing: false,
    ...overrides,
  }
}

describe('prompts store — links', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    localStorage.clear()
  })

  it('loadPrompt in project scope passes library root from localStorage', async () => {
    setLibraryRoot('proj1', 'C:/my/repo')
    const store = usePromptsStore()
    store.activeScope = 'project'
    store.activeProjectId = 'proj1'

    const link = basePrompt({
      id: 'link1',
      kind: 'link',
      source: 'docs/a.md',
      content: '# Hi',
      read_only: true,
      resolved_path: 'C:/my/repo/docs/a.md',
    })
    vi.mocked(promptsApi.get).mockResolvedValue(link)

    await store.loadPrompt('link1')

    expect(promptsApi.get).toHaveBeenCalledWith('link1', 'C:/my/repo')
    expect(store.selectedPrompt).toEqual(link)
  })

  it('reloadOpen refetches and replaces content', async () => {
    const store = usePromptsStore()
    store.selectedPrompt = basePrompt({ id: 'link1', kind: 'link', content: 'v1' })

    vi.mocked(promptsApi.get).mockResolvedValue(
      basePrompt({ id: 'link1', kind: 'link', content: 'v2' })
    )

    await store.reloadOpen()

    expect(promptsApi.get).toHaveBeenCalledWith('link1', undefined)
    expect(store.selectedPrompt?.content).toBe('v2')
  })

  it('window focus refetches an open link, not an open prompt', async () => {
    const store = usePromptsStore()
    store.registerFocusRefresh()

    const link = basePrompt({ id: 'link1', kind: 'link', content: 'from-disk' })
    vi.mocked(promptsApi.get).mockResolvedValue(link)
    store.selectedPrompt = link
    vi.mocked(promptsApi.get).mockClear()
    vi.mocked(promptsApi.get).mockResolvedValue(
      basePrompt({ id: 'link1', kind: 'link', content: 'refreshed' })
    )

    window.dispatchEvent(new Event('focus'))
    await vi.waitFor(() => expect(promptsApi.get).toHaveBeenCalled())

    expect(store.selectedPrompt?.content).toBe('refreshed')

    vi.mocked(promptsApi.get).mockClear()
    store.selectedPrompt = basePrompt({ id: 'p1', kind: 'prompt', content: 'draft' })

    window.dispatchEvent(new Event('focus'))
    await Promise.resolve()

    expect(promptsApi.get).not.toHaveBeenCalled()

    store.unregisterFocusRefresh()
  })

  it('createPrompt forwards kind and source', async () => {
    const store = usePromptsStore()
    store.activeScope = 'project'
    store.activeProjectId = 'proj1'

    const created = basePrompt({
      kind: 'link',
      source: 'a.md',
      content: '',
    })
    vi.mocked(promptsApi.create).mockResolvedValue(created)

    await store.createPrompt({ kind: 'link', source: 'a.md' })

    expect(promptsApi.create).toHaveBeenCalledWith(
      expect.objectContaining({
        scope: 'project',
        project_id: 'proj1',
        kind: 'link',
        source: 'a.md',
      })
    )
  })
})