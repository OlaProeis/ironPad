// @vitest-environment happy-dom
import { afterEach, beforeEach, describe, expect, it } from 'vitest'
import { defaultReadingMode, getReadingMode, setReadingMode } from './readingMode'

describe('defaultReadingMode', () => {
  it('is raw for a prompt, rendered for an .md link, and raw for a .txt link', () => {
    expect(defaultReadingMode({ kind: 'prompt', supports_rendered: true })).toBe('raw')
    expect(
      defaultReadingMode({ kind: 'link', supports_rendered: true, source: 'docs/readme.md' } as {
        kind: string
        supports_rendered: boolean
      })
    ).toBe('rendered')
    expect(
      defaultReadingMode({ kind: 'link', supports_rendered: false, source: 'notes/plain.txt' } as {
        kind: string
        supports_rendered: boolean
      })
    ).toBe('raw')
  })
})

describe('reading mode persistence', () => {
  beforeEach(() => {
    localStorage.clear()
  })

  afterEach(() => {
    localStorage.clear()
  })

  it('set/get persists under ironpad-library-mode:<itemId>', () => {
    expect(getReadingMode('item-1', 'raw')).toBe('raw')
    setReadingMode('item-1', 'rendered')
    expect(localStorage.getItem('ironpad-library-mode:item-1')).toBe('rendered')
    expect(getReadingMode('item-1', 'raw')).toBe('rendered')
    setReadingMode('item-1', 'raw')
    expect(getReadingMode('item-1', 'rendered')).toBe('raw')
  })
})
