// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { libraryApi } from '../../api/client'
import {
  clearLibraryRoot,
  getLibraryRoot,
  isInsideRoot,
  normalizeAbsolutePath,
  setLibraryRoot,
  toRelativeSource,
} from '../libraryRoot'
import LibraryRootBar from '../../components/library/LibraryRootBar.vue'

vi.mock('../../api/client', () => ({
  libraryApi: {
    pickLibrary: vi.fn(),
  },
}))

describe('libraryRoot storage', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('set/get/clear round-trip uses ironpad-library-root:p1', () => {
    expect(getLibraryRoot('p1')).toBeNull()
    setLibraryRoot('p1', 'C:/my/repo')
    expect(localStorage.getItem('ironpad-library-root:p1')).toBe('C:/my/repo')
    expect(getLibraryRoot('p1')).toBe('C:/my/repo')
    clearLibraryRoot('p1')
    expect(getLibraryRoot('p1')).toBeNull()
    expect(localStorage.getItem('ironpad-library-root:p1')).toBeNull()
  })

  it('strips a Windows verbatim prefix from a canonical path', () => {
    expect(normalizeAbsolutePath('\\\\?\\G:\\DEV\\proman2k\\ai-context.md')).toBe(
      'g:/DEV/proman2k/ai-context.md'
    )
  })

  it('strips a Windows bidi mark, wrapping quotes, and a trailing slash', () => {
    expect(normalizeAbsolutePath('\u202A"G:\\DEV\\proman2k\\"')).toBe('g:/DEV/proman2k')
    setLibraryRoot('p1', '\u202AG:\\DEV\\proman2k\\')
    expect(getLibraryRoot('p1')).toBe('G:\\DEV\\proman2k')
    expect(localStorage.getItem('ironpad-library-root:p1')).toBe('G:\\DEV\\proman2k')
    expect(toRelativeSource('\u202AG:\\DEV\\proman2k\\', '\u202AG:\\DEV\\proman2k\\README.md')).toBe(
      'README.md'
    )
  })
})

describe('isInsideRoot', () => {
  it('detects paths under the root', () => {
    expect(isInsideRoot('C:/repo', 'C:/repo/docs/index.md')).toBe(true)
    expect(isInsideRoot('C:/repo', 'C:/other/x.md')).toBe(false)
    expect(isInsideRoot('C:/repo', 'C:/repo2/x.md')).toBe(false)
  })
})

describe('toRelativeSource', () => {
  it('strips root prefix with normalized slashes', () => {
    expect(toRelativeSource('C:\\repo', 'C:\\repo\\docs\\index.md')).toBe('docs/index.md')
  })
})

describe('LibraryRootBar', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('Set stores the path and renders it; Clear removes the key', async () => {
    const wrapper = mount(LibraryRootBar, {
      props: { projectId: 'p1' },
    })

    expect(wrapper.get('[data-testid="library-root-current"]').text()).toBe('No folder root')

    const input = wrapper.get('[data-testid="library-root-input"]')
    await input.setValue('D:/projects/ironpad')

    await wrapper.get('[data-testid="library-root-set"]').trigger('click')

    expect(localStorage.getItem('ironpad-library-root:p1')).toBe('D:/projects/ironpad')
    expect(wrapper.get('[data-testid="library-root-current"]').text()).toBe('D:/projects/ironpad')

    await wrapper.get('[data-testid="library-root-clear"]').trigger('click')

    expect(localStorage.getItem('ironpad-library-root:p1')).toBeNull()
    expect(wrapper.get('[data-testid="library-root-current"]').text()).toBe('No folder root')
  })

  it('Browse stores the folder chosen in the dialog', async () => {
    vi.mocked(libraryApi.pickLibrary).mockResolvedValue({ paths: ['D:/projects/ironpad'] })
    const wrapper = mount(LibraryRootBar, {
      props: { projectId: 'p1' },
    })

    await wrapper.get('[data-testid="library-root-browse"]').trigger('click')
    await flushPromises()

    expect(libraryApi.pickLibrary).toHaveBeenCalledWith('directory', undefined)
    expect(localStorage.getItem('ironpad-library-root:p1')).toBe('D:/projects/ironpad')
    expect(wrapper.get('[data-testid="library-root-current"]').text()).toBe('D:/projects/ironpad')
  })
})