// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import type { Prompt, PromptSummary } from '../types'
import PromptsView from './PromptsView.vue'
import { usePromptsStore } from '../stores/prompts'
import { gitApi, promptsApi } from '../api/client'
import type { CommitDetail } from '../types'

vi.mock('../api/client', () => ({
  promptsApi: {
    list: vi.fn().mockResolvedValue([]),
    folders: vi.fn().mockResolvedValue([]),
    get: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    semanticSearch: vi.fn().mockResolvedValue([]),
  },
  gitApi: {
    logByPath: vi.fn().mockResolvedValue([]),
    commitDiff: vi.fn().mockResolvedValue({ files: [] }),
  },
}))

function summary(overrides: Partial<PromptSummary>): PromptSummary {
  return {
    id: 'id',
    title: 'Title',
    path: 'data/prompts/x.md',
    scope: 'global',
    folder: 'root',
    tags: [],
    kind: 'prompt',
    ...overrides,
  }
}

function fullPrompt(overrides: Partial<Prompt>): Prompt {
  return {
    ...summary({ id: 'link1', title: 'My Link', kind: 'link', source: 'docs/readme.md' }),
    template: false,
    frontmatter: {},
    content: '# Hi',
    read_only: true,
    source_missing: false,
    ...overrides,
  }
}

async function mountPromptsView() {
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      {
        path: '/prompts/:promptId?',
        name: 'prompts',
        component: PromptsView,
        props: true,
      },
    ],
  })
  await router.push('/prompts')
  await router.isReady()

  const wrapper = mount(PromptsView, {
    global: {
      plugins: [router],
      stubs: { LibraryRootBar: true },
    },
  })
  await flushPromises()
  return { wrapper, router }
}

describe('PromptsView — kind badges and link actions', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('shows Prompt and Link badges in the library list', async () => {
    const items = [
      summary({ id: 'p1', title: 'Regular', kind: 'prompt' }),
      summary({ id: 'l1', title: 'Pointer', kind: 'link', source: 'a.md' }),
    ]
    vi.mocked(promptsApi.list).mockResolvedValue(items)

    const store = usePromptsStore()
    store.semanticMode = false

    const { wrapper } = await mountPromptsView()
    const badges = wrapper.findAll('.kind-badge')
    const texts = badges.map((b) => b.text())
    expect(texts).toContain('Prompt')
    expect(texts).toContain('Link')
  })

  it('duplicate link calls create with kind link, same source, and no content', async () => {
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    const link = fullPrompt({
      id: 'link1',
      title: 'Readme',
      kind: 'link',
      source: 'docs/readme.md',
    })
    store.selectedPrompt = link

    const created = fullPrompt({ id: 'link2', title: 'Readme (Copy)' })
    const createSpy = vi.spyOn(store, 'createPrompt').mockResolvedValue(created)
    vi.spyOn(store, 'loadPrompt').mockResolvedValue()

    const { wrapper, router } = await mountPromptsView()
    await router.push({ name: 'prompts', params: { promptId: 'link1' } })
    await flushPromises()

    const dupButton = wrapper.findAll('button').find((b) => b.text() === 'Duplicate link')
    expect(dupButton).toBeDefined()
    await dupButton!.trigger('click')
    await flushPromises()

    expect(createSpy).toHaveBeenCalledWith({
      title: 'Readme (Copy)',
      folder: link.folder,
      tags: link.tags,
      description: link.description,
      kind: 'link',
      source: 'docs/readme.md',
      template: false,
    })
    const payload = createSpy.mock.calls[0]![0]
    expect(payload).not.toHaveProperty('content')
  })

  it('delete link confirm mentions title and that the file on disk stays', async () => {
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    store.selectedPrompt = fullPrompt({
      id: 'link1',
      title: 'Important Doc',
      kind: 'link',
      source: 'notes/doc.md',
    })

    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false)

    const { wrapper } = await mountPromptsView()
    const deleteButton = wrapper.findAll('button').find((b) => b.text() === 'Delete')
    expect(deleteButton).toBeDefined()
    await deleteButton!.trigger('click')

    expect(confirmSpy).toHaveBeenCalled()
    const message = String(confirmSpy.mock.calls[0]![0])
    expect(message).toContain('Important Doc')
    expect(message.toLowerCase()).toContain('file on disk stays')
  })
})

describe('PromptsView — reading mode', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    localStorage.clear()
  })

  it('keeps the raw draft when switching to Rendered and back', async () => {
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    store.selectedPrompt = fullPrompt({
      id: 'p1',
      title: 'Draft',
      kind: 'prompt',
      content: 'start',
      read_only: false,
    })

    const { wrapper } = await mountPromptsView()
    const editor = wrapper.get('[data-testid="raw-editor"]')
    await editor.setValue('start plus edit')
    await wrapper.get('[data-testid="reading-mode-rendered"]').trigger('click')
    expect(wrapper.find('[data-testid="raw-editor"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="rendered-markdown"]').exists()).toBe(true)
    await wrapper.get('[data-testid="reading-mode-raw"]').trigger('click')
    expect(wrapper.get('[data-testid="raw-editor"]').element).toHaveProperty('value', 'start plus edit')
  })

  it('saves the raw draft while Rendered mode is showing', async () => {
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    const prompt = fullPrompt({
      id: 'p1',
      title: 'Draft',
      kind: 'prompt',
      content: 'alpha {{name}}',
      read_only: false,
      template: false,
    })
    store.selectedPrompt = prompt
    const updateSpy = vi.spyOn(store, 'updatePrompt').mockResolvedValue(prompt)

    const { wrapper } = await mountPromptsView()
    await wrapper.get('[data-testid="raw-editor"]').setValue('alpha {{name}} edited')
    await wrapper.get('[data-testid="reading-mode-rendered"]').trigger('click')
    const save = wrapper.findAll('button').find((b) => b.text() === 'Save')
    await save!.trigger('click')
    await flushPromises()

    expect(updateSpy).toHaveBeenCalled()
    const payload = updateSpy.mock.calls[0]![1] as { content?: string }
    expect(payload.content).toBe('alpha {{name}} edited')
  })

  it('hides Rendered for a .txt link', async () => {
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    store.selectedPrompt = fullPrompt({
      id: 'txt1',
      title: 'Plain',
      kind: 'link',
      source: 'notes/plain.txt',
      supports_rendered: false,
      content: 'plain text',
    })

    const { wrapper } = await mountPromptsView()
    expect(wrapper.find('[data-testid="reading-mode-raw"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="reading-mode-rendered"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="link-content"]').exists()).toBe(true)
  })

  it('substitutes closed variables in rendered mode and leaves an unclosed {{ literal', async () => {
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    store.selectedPrompt = fullPrompt({
      id: 'p1',
      title: 'Vars',
      kind: 'prompt',
      content: 'Hello {{name}} and {{',
      read_only: false,
    })

    const { wrapper } = await mountPromptsView()
    const nameInput = wrapper.get('input[placeholder="Value for name"]')
    await nameInput.setValue('X')
    await wrapper.get('[data-testid="reading-mode-rendered"]').trigger('click')
    const text = wrapper.get('[data-testid="rendered-markdown"]').text()
    expect(text).toContain('X')
    expect(text).toContain('{{')
    expect(text).not.toContain('{{name}}')
  })
})

describe('PromptsView — link template toggle', () => {
  let clipboardWrite: ReturnType<typeof vi.spyOn>

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    clipboardWrite = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue(undefined)
  })

  async function mountWithLink(overrides: Partial<Prompt>) {
    const link = fullPrompt({
      id: 'link1',
      title: 'Templated doc',
      kind: 'link',
      source: 'docs/msg.md',
      content: 'Hi {{name}}',
      read_only: true,
      ...overrides,
    })
    vi.mocked(promptsApi.get).mockResolvedValue(link)

    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false

    const { wrapper, router } = await mountPromptsView()
    await router.push({ name: 'prompts', params: { promptId: 'link1' } })
    await flushPromises()
    return { wrapper, store, link }
  }

  it('with template false hides variable form and Copy copies source verbatim', async () => {
    const { wrapper } = await mountWithLink({ template: false })
    expect(wrapper.find('input[placeholder="Value for name"]').exists()).toBe(false)
    const copy = wrapper.findAll('button').find((b) => b.text() === 'Copy')
    await copy!.trigger('click')
    await flushPromises()
    expect(clipboardWrite).toHaveBeenCalledWith('Hi {{name}}')
  })

  it('toggling Template calls update with only template true', async () => {
    const { wrapper, store } = await mountWithLink({ template: false })
    const updateSpy = vi.spyOn(store, 'updatePrompt').mockImplementation(async (_id, payload) => {
      return fullPrompt({ ...store.selectedPrompt!, template: payload.template ?? false })
    })
    await wrapper.get('[data-testid="link-template"]').setValue(true)
    await flushPromises()
    expect(updateSpy).toHaveBeenCalledWith('link1', { template: true })
    for (const call of updateSpy.mock.calls) {
      expect(call[1]).not.toHaveProperty('content')
    }
  })

  it('with template true shows variable field and Copy Rendered substitutes', async () => {
    const { wrapper } = await mountWithLink({ template: true })
    expect(wrapper.get('input[placeholder="Value for name"]').exists()).toBe(true)
    await wrapper.get('input[placeholder="Value for name"]').setValue('Ada')
    const copyRendered = wrapper.findAll('button').find((b) => b.text() === 'Copy Rendered')
    expect(copyRendered).toBeDefined()
    await copyRendered!.trigger('click')
    await flushPromises()
    expect(clipboardWrite).toHaveBeenCalledWith('Hi Ada')
  })
})

describe('PromptsView — collapsible Versions (Git)', () => {
  const VERSIONS_KEY = 'ironpad-library-versions-expanded'

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    localStorage.clear()
  })

  async function mountWithPrompt(overrides: Partial<Prompt> = {}) {
    const prompt = fullPrompt({
      id: 'p1',
      title: 'Tracked',
      kind: 'prompt',
      path: 'data/prompts/p1.md',
      content: 'body',
      read_only: false,
      ...overrides,
    })
    vi.mocked(promptsApi.get).mockResolvedValue(prompt)
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    const { wrapper, router } = await mountPromptsView()
    await router.push({ name: 'prompts', params: { promptId: 'p1' } })
    await flushPromises()
    return { wrapper, store, prompt }
  }

  it('defaults collapsed: no versions panel and full-width grid', async () => {
    const { wrapper } = await mountWithPrompt()
    expect(wrapper.find('[data-testid="versions-panel"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="content-grid"]').classes()).toContain('content-grid--full-width')
  })

  it('expanding shows commit log for a prompt and persists true', async () => {
    const commits: CommitDetail[] = [
      {
        id: 'c1',
        short_id: 'abc1234',
        message: 'Save prompt',
        author: 'me',
        timestamp: '2026-01-01T12:00:00Z',
      },
    ]
    vi.mocked(gitApi.logByPath).mockResolvedValue(commits)
    const { wrapper } = await mountWithPrompt()
    await wrapper.get('[data-testid="versions-toggle"]').trigger('click')
    await flushPromises()
    expect(localStorage.getItem(VERSIONS_KEY)).toBe('true')
    expect(wrapper.get('[data-testid="versions-panel"]').exists()).toBe(true)
    expect(wrapper.find('.history-item').exists()).toBe(true)
    expect(gitApi.logByPath).toHaveBeenCalled()
  })

  it('keeps selected commit after collapse and expand', async () => {
    const commits: CommitDetail[] = [
      {
        id: 'c1',
        short_id: 'aaa1111',
        message: 'first',
        author: 'me',
        timestamp: '2026-01-01T12:00:00Z',
      },
      {
        id: 'c2',
        short_id: 'bbb2222',
        message: 'second',
        author: 'me',
        timestamp: '2026-01-02T12:00:00Z',
      },
    ]
    vi.mocked(gitApi.logByPath).mockResolvedValue(commits)
    vi.mocked(gitApi.commitDiff).mockResolvedValue({
      id: 'c1',
      files: [
        {
          path: 'data/prompts/p1.md',
          hunks: [{ header: '@@ -1 +1 @@' }],
        },
      ],
    })
    const { wrapper } = await mountWithPrompt()
    await wrapper.get('[data-testid="versions-toggle"]').trigger('click')
    await flushPromises()
    const items = wrapper.findAll('.history-item')
    await items[0]!.trigger('click')
    await flushPromises()
    expect(wrapper.find('.history-item.active').exists()).toBe(true)
    await wrapper.get('[data-testid="versions-toggle"]').trigger('click')
    await wrapper.get('[data-testid="versions-toggle"]').trigger('click')
    await flushPromises()
    expect(wrapper.find('.history-item.active').exists()).toBe(true)
  })

  it('expanded link shows outside-repository notice and does not fetch git log', async () => {
    const link = fullPrompt({
      id: 'link1',
      title: 'External',
      kind: 'link',
      source: 'outside/readme.md',
      path: 'data/prompts/link1.md',
    })
    vi.mocked(promptsApi.get).mockResolvedValue(link)
    const store = usePromptsStore()
    store.loading = false
    store.semanticMode = false
    localStorage.setItem(VERSIONS_KEY, 'true')
    const { wrapper, router } = await mountPromptsView()
    await router.push({ name: 'prompts', params: { promptId: 'link1' } })
    await flushPromises()
    const notice = wrapper.get('[data-testid="link-versions-notice"]')
    expect(notice.text()).toBe("This file is outside Ironpad's repository.")
    expect(gitApi.logByPath).not.toHaveBeenCalled()
  })
})

describe('PromptsView — copy bundle', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText: vi.fn().mockResolvedValue(undefined) },
    })
  })

  async function checkRow(wrapper: Awaited<ReturnType<typeof mountPromptsView>>['wrapper'], id: string) {
    const box = wrapper.get(`[data-testid="bundle-select-${id}"]`)
    await box.trigger('click')
    await box.trigger('change')
  }

  it('writes both titles when two rows are checked and leaves the open item unchanged', async () => {
    vi.mocked(promptsApi.list).mockResolvedValue([
      summary({ id: 'a', title: 'Alpha', kind: 'prompt', path: 'data/prompts/a.md' }),
      summary({ id: 'b', title: 'Beta', kind: 'prompt', path: 'data/prompts/b.md' }),
    ])
    vi.mocked(promptsApi.get).mockImplementation(async (id: string) =>
      fullPrompt({
        id,
        title: id === 'a' ? 'Alpha' : 'Beta',
        kind: 'prompt',
        path: `data/prompts/${id}.md`,
        content: id === 'a' ? 'body a {{name}}' : 'body b',
        template: false,
        read_only: false,
      })
    )
    const store = usePromptsStore()
    store.semanticMode = false
    const { wrapper, router } = await mountPromptsView()
    await router.push({ name: 'prompts', params: { promptId: 'a' } })
    await flushPromises()
    expect(store.selectedPrompt?.id).toBe('a')

    await checkRow(wrapper, 'a')
    await checkRow(wrapper, 'b')
    await wrapper.get('[data-testid="copy-bundle"]').trigger('click')
    await flushPromises()

    const written = vi.mocked(navigator.clipboard.writeText).mock.calls.at(-1)?.[0] as string
    expect(written).toContain('## Alpha')
    expect(written).toContain('## Beta')
    expect(store.selectedPrompt?.id).toBe('a')
  })

  it('bundles the open item when nothing is selected', async () => {
    vi.mocked(promptsApi.list).mockResolvedValue([
      summary({ id: 'a', title: 'Alpha', kind: 'prompt', path: 'data/prompts/a.md' }),
      summary({ id: 'b', title: 'Beta', kind: 'prompt', path: 'data/prompts/b.md' }),
    ])
    vi.mocked(promptsApi.get).mockImplementation(async (id: string) =>
      fullPrompt({
        id,
        title: id === 'a' ? 'Alpha' : 'Beta',
        kind: 'prompt',
        path: `data/prompts/${id}.md`,
        content: 'only open',
        template: false,
        read_only: false,
      })
    )
    const store = usePromptsStore()
    store.semanticMode = false
    const { wrapper, router } = await mountPromptsView()
    await router.push({ name: 'prompts', params: { promptId: 'a' } })
    await flushPromises()

    await wrapper.get('[data-testid="copy-bundle"]').trigger('click')
    await flushPromises()

    const written = vi.mocked(navigator.clipboard.writeText).mock.calls.at(-1)?.[0] as string
    expect(written).toContain('## Alpha')
    expect(written).not.toContain('## Beta')
    expect(vi.mocked(promptsApi.get).mock.calls.some((call) => call[0] === 'b')).toBe(false)
  })

  it("uses a link's resolved_path in the bundle section", async () => {
    vi.mocked(promptsApi.list).mockResolvedValue([
      summary({ id: 'link1', title: 'Readme', kind: 'link', source: 'docs/readme.md' }),
    ])
    vi.mocked(promptsApi.get).mockResolvedValue(
      fullPrompt({
        id: 'link1',
        title: 'Readme',
        kind: 'link',
        source: 'docs/readme.md',
        resolved_path: 'C:/repo/docs/readme.md',
        content: 'linked body',
      })
    )
    const store = usePromptsStore()
    store.semanticMode = false
    const { wrapper, router } = await mountPromptsView()
    await router.push({ name: 'prompts', params: { promptId: 'link1' } })
    await flushPromises()

    await wrapper.get('[data-testid="copy-bundle"]').trigger('click')
    await flushPromises()

    const written = vi.mocked(navigator.clipboard.writeText).mock.calls.at(-1)?.[0] as string
    expect(written).toContain('C:/repo/docs/readme.md')
    expect(written).toContain('linked body')
  })
})