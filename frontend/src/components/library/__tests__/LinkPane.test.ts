// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import type { Prompt } from '../../../types'
import LinkPane from '../LinkPane.vue'
import { libraryApi } from '../../../api/client'
import { setLibraryRoot } from '../../../lib/libraryRoot'
import { usePromptsStore } from '../../../stores/prompts'

vi.mock('../../../api/client', () => ({
  libraryApi: {
    openLibraryItem: vi.fn(),
  },
}))

function link(overrides: Partial<Prompt> = {}): Prompt {
  return {
    id: 'link1',
    title: 'Readme',
    path: 'data/prompts/link1.md',
    scope: 'project',
    project_id: 'p1',
    folder: 'root',
    tags: [],
    kind: 'link',
    source: 'docs/readme.md',
    template: false,
    frontmatter: {},
    content: '',
    read_only: true,
    source_missing: false,
    resolved_path: 'C:/repo/docs/readme.md',
    ...overrides,
  }
}

describe('LinkPane', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(libraryApi.openLibraryItem).mockReset()
    vi.mocked(libraryApi.openLibraryItem).mockResolvedValue({
      ok: true,
      resolved_path: 'C:/repo/docs/readme.md',
    })
  })

  function mountPane(prompt: Prompt) {
    return mount(LinkPane, {
      props: { prompt },
      global: { plugins: [createPinia()] },
    })
  }

  it('renders the stored path and Relink, and submits a new source', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const store = usePromptsStore()
    const update = vi.spyOn(store, 'updatePrompt').mockResolvedValue(link())

    const wrapper = mount(LinkPane, {
      props: { prompt: link({ source_missing: true, source: 'docs/old.md' }) },
      global: { plugins: [pinia] },
    })

    expect(wrapper.get('[data-testid="link-stored-path"]').text()).toBe('docs/old.md')
    expect(wrapper.text()).toContain('File not found')
    expect(wrapper.get('[data-testid="link-relink"]').exists()).toBe(true)

    await wrapper.get('[data-testid="link-relink-input"]').setValue('docs/new.md')
    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(update).toHaveBeenCalledWith('link1', { source: 'docs/new.md' })
  })

  it('renders the root prompt when root_required', () => {
    const wrapper = mountPane(link({ root_required: true }))
    expect(wrapper.text()).toContain('Set a folder root for this project to open this link')
  })

  it('renders the source_error message verbatim', () => {
    const wrapper = mountPane(link({ source_error: 'File is 2000000 bytes...' }))
    expect(wrapper.text()).toContain('File is 2000000 bytes...')
  })

  it('renders content in a read-only pre', () => {
    const wrapper = mountPane(link({ content: '# Hello from disk' }))
    const pre = wrapper.get('pre')
    expect(pre.text()).toContain('# Hello from disk')
    expect(wrapper.find('textarea').exists()).toBe(false)
  })

  it('Copy path writes resolved_path to the clipboard', async () => {
    const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue(undefined)
    const wrapper = mountPane(
      link({ resolved_path: 'C:/repo/ai-context.md', content: '# ctx' }),
    )
    await wrapper.get('[data-testid="link-copy-path"]').trigger('click')
    expect(write).toHaveBeenCalledWith('C:/repo/ai-context.md')
  })

  it('Open externally calls openLibraryItem with id and root', async () => {
    setLibraryRoot('p1', 'C:/repo')
    const wrapper = mountPane(
      link({ resolved_path: 'C:/repo/ai-context.md', content: '# ctx' }),
    )
    await wrapper.get('[data-testid="link-open-externally"]').trigger('click')
    await flushPromises()
    expect(libraryApi.openLibraryItem).toHaveBeenCalledWith('link1', 'C:/repo')
  })

  it('disables Copy path and Open externally when source_missing', async () => {
    const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue(undefined)
    const wrapper = mountPane(link({ source_missing: true, source: 'docs/old.md' }))
    const copyBtn = wrapper.get('[data-testid="link-copy-path"]').element as HTMLButtonElement
    const openBtn = wrapper.get('[data-testid="link-open-externally"]').element as HTMLButtonElement
    expect(copyBtn.disabled).toBe(true)
    expect(openBtn.disabled).toBe(true)
    await wrapper.get('[data-testid="link-copy-path"]').trigger('click')
    await wrapper.get('[data-testid="link-open-externally"]').trigger('click')
    expect(write).not.toHaveBeenCalled()
    expect(libraryApi.openLibraryItem).not.toHaveBeenCalled()
  })

  it('Reload calls reloadOpen', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const store = usePromptsStore()
    const reload = vi.spyOn(store, 'reloadOpen').mockResolvedValue()

    const wrapper = mount(LinkPane, {
      props: { prompt: link({ content: 'body' }) },
      global: { plugins: [pinia] },
    })
    await wrapper.get('[data-testid="link-reload"]').trigger('click')
    expect(reload).toHaveBeenCalled()
  })
})
