import { describe, expect, it } from 'vitest'
import { buildBundle } from './copyBundle'

describe('buildBundle', () => {
  it('emits headings, path lines, and unsubstituted text in order', () => {
    const text = buildBundle([
      { title: 'Alpha', path: 'data/prompts/a.md', text: 'Hello {{name}}' },
      { title: 'Beta', path: 'C:/repo/docs/b.md', text: 'Second' },
    ])

    expect(text).toBe(
      '## Alpha\n\ndata/prompts/a.md\n\nHello {{name}}\n\n## Beta\n\nC:/repo/docs/b.md\n\nSecond\n'
    )
    const headings = text.match(/^## /gm)
    expect(headings).toEqual(['## ', '## '])
    expect(text.indexOf('## Alpha')).toBeLessThan(text.indexOf('## Beta'))
    expect(text).toContain('{{name}}')
  })
})
