// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import type { BrowseResponse } from '../../types'
import {
  dismissSuggest,
  findSuggestions,
  isSuggestDismissed,
  type BrowseFn,
} from '../librarySuggest'

describe('librarySuggest', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('findSuggestions returns paths that exist and are not already linked', async () => {
    const rootListing: BrowseResponse = {
      path: '/repo',
      parent: null,
      entries: [
        { name: 'ai-context.md', path: '/repo/ai-context.md', is_dir: false, size: 1, linkable: true },
        { name: 'ROADMAP.md', path: '/repo/ROADMAP.md', is_dir: false, size: 1, linkable: false },
      ],
    }
    const docsListing: BrowseResponse = {
      path: '/repo/docs',
      parent: '/repo',
      entries: [
        { name: 'index.md', path: '/repo/docs/index.md', is_dir: false, size: 1, linkable: true },
      ],
    }

    const browse = vi.fn<BrowseFn>(async (path: string) => {
      if (path === '/repo') return rootListing
      if (path === '/repo/docs') return docsListing
      throw new Error('unexpected path')
    })

    const result = await findSuggestions('/repo', ['ai-context.md'], browse)
    expect(result).toEqual(['docs/index.md'])
    expect(browse).toHaveBeenCalledWith('/repo')
    expect(browse).toHaveBeenCalledWith('/repo/docs')
  })

  it('dismissSuggest stores dismissal per project', () => {
    expect(isSuggestDismissed('p1')).toBe(false)
    dismissSuggest('p1')
    expect(isSuggestDismissed('p1')).toBe(true)
    expect(localStorage.getItem('ironpad-library-suggest-dismissed:p1')).toBe('1')
    expect(isSuggestDismissed('p2')).toBe(false)
  })
})