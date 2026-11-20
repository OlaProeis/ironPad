import { describe, expect, it } from 'vitest'
import {
  isImagePath,
  libraryMediaUrl,
  resolveLocalMarkdownPath,
  sameLinkedSource,
  sourceForLibraryFile,
} from '../markdownTarget'

describe('resolveLocalMarkdownPath', () => {
  it('resolves a relative image when the file path has a Windows verbatim prefix', () => {
    expect(
      resolveLocalMarkdownPath(
        '\\\\?\\G:\\DEV\\proman2k\\ai-context.md',
        'docs/graphics/tech-stack.png',
        'G:\\DEV\\proman2k'
      )
    ).toBe('g:/DEV/proman2k/docs/graphics/tech-stack.png')
  })

  it('resolves a relative image against the markdown file directory', () => {
    expect(
      resolveLocalMarkdownPath(
        'C:/repo/ai-context.md',
        'docs/graphics/ironpad-banner.png',
        'C:/repo'
      )
    ).toBe('c:/repo/docs/graphics/ironpad-banner.png')
  })

  it('resolves parent segments and a repo-root slash link', () => {
    expect(resolveLocalMarkdownPath('C:/repo/docs/index.md', '../README.md', 'C:/repo')).toBe(
      'c:/repo/README.md'
    )
    expect(resolveLocalMarkdownPath('C:/repo/docs/index.md', '/docs/other.md', 'C:/repo')).toBe(
      'c:/repo/docs/other.md'
    )
  })

  it('ignores anchors and external urls', () => {
    expect(resolveLocalMarkdownPath('C:/repo/a.md', '#title', 'C:/repo')).toBeNull()
    expect(resolveLocalMarkdownPath('C:/repo/a.md', 'https://example.com', 'C:/repo')).toBeNull()
  })

  it('drops a fragment when resolving a file link', () => {
    expect(resolveLocalMarkdownPath('C:/repo/a.md', 'docs/index.md#setup', 'C:/repo')).toBe(
      'c:/repo/docs/index.md'
    )
  })
})

describe('sourceForLibraryFile', () => {
  it('stores a path inside the root as relative and a path outside as absolute', () => {
    expect(sourceForLibraryFile('C:/repo', 'C:/repo/docs/index.md')).toBe('docs/index.md')
    expect(sourceForLibraryFile('C:/repo', 'D:/notes/extra.md')).toBe('d:/notes/extra.md')
    expect(sourceForLibraryFile(null, 'D:\\notes\\extra.md')).toBe('d:/notes/extra.md')
  })
})

describe('sameLinkedSource', () => {
  it('matches a relative source and an absolute source for the same file', () => {
    expect(sameLinkedSource('docs/a.md', 'C:/repo/docs/a.md', 'C:/repo')).toBe(true)
    expect(sameLinkedSource('d:/notes/a.md', 'D:/notes/a.md', 'C:/repo')).toBe(true)
    expect(sameLinkedSource('docs/a.md', 'D:/notes/a.md', 'C:/repo')).toBe(false)
  })
})

describe('libraryMediaUrl', () => {
  it('points at the local media endpoint', () => {
    expect(isImagePath('docs/graphics/ironpad-banner.png')).toBe(true)
    expect(libraryMediaUrl('C:/repo/docs/graphics/ironpad-banner.png')).toBe(
      '/api/library/media?path=' + encodeURIComponent('c:/repo/docs/graphics/ironpad-banner.png')
    )
  })
})
