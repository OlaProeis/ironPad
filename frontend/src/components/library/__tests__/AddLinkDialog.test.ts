// @vitest-environment happy-dom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import type { BrowseResponse, Prompt } from '../../../types'
import AddLinkDialog from '../AddLinkDialog.vue'
import { libraryApi } from '../../../api/client'
import { setLibraryRoot } from '../../../lib/libraryRoot'
import { usePromptsStore } from '../../../stores/prompts'

vi.mock('../../../api/client', () => ({
  libraryApi: {
    browseLibrary: vi.fn(),
    pickLibrary: vi.fn(),
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

const listing: BrowseResponse = {
  path: '/repo',
  parent: '/repo-parent',
  entries: [
    { name: 'docs', path: '/repo/docs', is_dir: true, size: 0, linkable: false },
    { name: 'ai-context.md', path: '/repo/ai-context.md', is_dir: false, size: 12, linkable: true },
    { name: 'app.exe', path: '/repo/app.exe', is_dir: false, size: 20, linkable: false },
  ],
}

function createdLink(): Prompt {
  return {
    id: 'new',
    title: 'ai-context.md',
    path: 'data/prompts/new.md',
    scope: 'project',
    folder: 'root',
    tags: [],
    kind: 'link',
    source: 'ai-context.md',
    template: false,
    frontmatter: {},
    content: '',
    read_only: true,
    source_missing: false,
  }
}

describe('AddLinkDialog', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.mocked(libraryApi.browseLibrary).mockReset()
    vi.mocked(libraryApi.browseLibrary).mockResolvedValue(listing)
    vi.mocked(libraryApi.pickLibrary).mockReset()
    localStorage.clear()
  })

  async function mountDialog(props: { mode: 'global' | 'project'; projectId: string | null }) {
    const pinia = createPinia()
    setActivePinia(pinia)
    const wrapper = mount(AddLinkDialog, {
      props: { open: true, ...props },
      global: { plugins: [pinia] },
    })
    await flushPromises()
    return wrapper
  }

  it('navigates directories and disables non-linkable files', async () => {
    setLibraryRoot('p1', '/repo')
    const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })

    expect(libraryApi.browseLibrary).toHaveBeenCalledWith('/repo')
    const dir = wrapper.get('[data-testid="browse-dir"]')
    expect(dir.text()).toContain('docs')
    const disabled = wrapper.get('[data-testid="browse-file-disabled"] input')
    expect((disabled.element as HTMLInputElement).disabled).toBe(true)
    expect(wrapper.get('[data-testid="browse-file-disabled"]').text()).toContain('not allowed')

    await dir.trigger('click')
    expect(libraryApi.browseLibrary).toHaveBeenCalledWith('/repo/docs')

    await wrapper.get('[data-testid="browse-up"]').trigger('click')
    expect(libraryApi.browseLibrary).toHaveBeenCalledWith('/repo-parent')
  })

  it('creates a project link with a root-relative source', async () => {
    setLibraryRoot('p1', '/repo')
    const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })
    const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())
    await wrapper.get('[data-testid="browse-file"] input').setValue(true)
    await wrapper.get('[data-testid="add-link-confirm"]').trigger('click')
    await flushPromises()

    expect(create).toHaveBeenCalledTimes(1)
    expect(create).toHaveBeenCalledWith({
      kind: 'link',
      title: 'ai-context.md',
      folder: undefined,
      tags: [],
      template: false,
      source: 'ai-context.md',
    })
  })

  it('creates a global link with an absolute source', async () => {
    const wrapper = await mountDialog({ mode: 'global', projectId: null })
    const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())
    await wrapper.get('[data-testid="add-link-path"]').setValue('/repo')
    await wrapper.get('[data-testid="add-link-go"]').trigger('click')
    await flushPromises()

    await wrapper.get('[data-testid="browse-file"] input').setValue(true)
    await wrapper.get('[data-testid="add-link-confirm"]').trigger('click')
    await flushPromises()

    expect(create).toHaveBeenCalledTimes(1)
    expect(create).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'link',
        title: 'ai-context.md',
        source: '/repo/ai-context.md',
      })
    )
  })

  it('shows the root prompt when the project has no library root', async () => {
    const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })
    expect(wrapper.get('[data-testid="add-link-root-prompt"]').text()).toContain(
      'Set a folder root for this project'
    )
    expect(libraryApi.browseLibrary).not.toHaveBeenCalled()
  })

  describe('paste path', () => {
    it('creates a project link when the pasted path is under the root', async () => {
      setLibraryRoot('p1', 'C:/repo')
      const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })
      const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())

      await wrapper.get('[data-testid="add-link-paste-path"]').setValue('C:/repo/docs/index.md')
      await wrapper.get('[data-testid="add-link-paste-add"]').trigger('click')
      await flushPromises()

      expect(create).toHaveBeenCalledTimes(1)
      expect(create).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: 'link',
          title: 'index.md',
          source: 'docs/index.md',
        })
      )
    })

    it('strips a Windows bidi mark and stores a root-relative source', async () => {
      setLibraryRoot('p1', 'G:\\DEV\\proman2k')
      const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })
      const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())

      await wrapper
        .get('[data-testid="add-link-paste-path"]')
        .setValue('\u202AG:\\DEV\\proman2k\\README.md')
      await wrapper.get('[data-testid="add-link-paste-add"]').trigger('click')
      await flushPromises()

      expect(create).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: 'link',
          title: 'README.md',
          source: 'README.md',
        })
      )
    })

    it('browses a folder root that was saved with a bidi mark', async () => {
      localStorage.setItem('ironpad-library-root:p1', '\u202AG:\\DEV\\proman2k\\')
      await mountDialog({ mode: 'project', projectId: 'p1' })
      expect(libraryApi.browseLibrary).toHaveBeenCalledWith('G:\\DEV\\proman2k')
    })

    it('stores an absolute source when the pasted file is outside the root', async () => {
      setLibraryRoot('p1', 'C:/repo')
      const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })
      const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())

      await wrapper.get('[data-testid="add-link-paste-path"]').setValue('D:/other/x.md')
      await wrapper.get('[data-testid="add-link-paste-add"]').trigger('click')
      await flushPromises()

      expect(create).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: 'link',
          title: 'x.md',
          source: 'd:/other/x.md',
        })
      )
    })

    it('adds a picked file outside the project root', async () => {
      setLibraryRoot('p1', 'C:/repo')
      vi.mocked(libraryApi.pickLibrary).mockResolvedValue({ paths: ['D:\\notes\\extra.md'] })
      const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })
      const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())

      await wrapper.get('[data-testid="add-link-browse-file"]').trigger('click')
      await flushPromises()

      expect(libraryApi.pickLibrary).toHaveBeenCalledWith('file', 'C:/repo')
      expect(create).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: 'link',
          title: 'extra.md',
          source: 'd:/notes/extra.md',
        })
      )
    })

    it('refuses project pastes with parent segments', async () => {
      setLibraryRoot('p1', 'C:/repo')
      const wrapper = await mountDialog({ mode: 'project', projectId: 'p1' })
      const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())

      await wrapper.get('[data-testid="add-link-paste-path"]').setValue('../x.md')
      await wrapper.get('[data-testid="add-link-paste-add"]').trigger('click')
      await flushPromises()

      expect(create).not.toHaveBeenCalled()
      expect(wrapper.get('[data-testid="add-link-error"]').text()).toContain("'..'")
    })

    it('creates a global link from an absolute pasted path', async () => {
      const wrapper = await mountDialog({ mode: 'global', projectId: null })
      const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())

      await wrapper.get('[data-testid="add-link-paste-path"]').setValue('C:/abs/x.md')
      await wrapper.get('[data-testid="add-link-paste-add"]').trigger('click')
      await flushPromises()

      expect(create).toHaveBeenCalledTimes(1)
      expect(create).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: 'link',
          title: 'x.md',
          source: 'c:/abs/x.md',
        })
      )
    })

    it('refuses a relative global paste', async () => {
      const wrapper = await mountDialog({ mode: 'global', projectId: null })
      const create = vi.spyOn(usePromptsStore(), 'createPrompt').mockResolvedValue(createdLink())

      await wrapper.get('[data-testid="add-link-paste-path"]').setValue('relative.md')
      await wrapper.get('[data-testid="add-link-paste-add"]').trigger('click')
      await flushPromises()

      expect(create).not.toHaveBeenCalled()
      expect(wrapper.get('[data-testid="add-link-error"]').text()).toContain('absolute path')
    })
  })
})
