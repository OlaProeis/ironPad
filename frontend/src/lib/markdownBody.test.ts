import { describe, expect, it } from 'vitest'
import { stripFrontmatter } from './markdownBody'

describe('stripFrontmatter', () => {
  it('strips a leading YAML frontmatter block', () => {
    const input = '---\ntitle: Hello\ntags:\n  - a\n---\n\n# Body\n'
    expect(stripFrontmatter(input)).toBe('\n# Body\n')
  })

  it('keeps --- delimiters when they appear in the middle of the document', () => {
    const input = '# Title\n\n---\n\nNot frontmatter\n'
    expect(stripFrontmatter(input)).toBe(input)
  })

  it('returns text unchanged when there is no frontmatter', () => {
    const input = '# Title\n\nParagraph.\n'
    expect(stripFrontmatter(input)).toBe(input)
  })
})