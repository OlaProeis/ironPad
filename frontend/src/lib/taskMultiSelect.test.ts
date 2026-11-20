import { describe, expect, it } from 'vitest'
import {
  clearMultiSelect,
  isMultiSelected,
  pruneMultiSelect,
  toggleMultiSelect,
} from './taskMultiSelect'

describe('taskMultiSelect', () => {
  it('toggles selection without mutating the original set', () => {
    const initial = new Set(['a'])
    const added = toggleMultiSelect(initial, 'b')
    expect(added.has('a')).toBe(true)
    expect(added.has('b')).toBe(true)
    expect(initial.has('b')).toBe(false)

    const removed = toggleMultiSelect(added, 'a')
    expect(removed.has('a')).toBe(false)
    expect(removed.has('b')).toBe(true)
  })

  it('clears all selections', () => {
    expect(clearMultiSelect().size).toBe(0)
  })

  it('prunes ids not in the visible top-level list', () => {
    const selected = new Set(['visible-1', 'hidden-2', 'visible-3'])
    const pruned = pruneMultiSelect(selected, ['visible-1', 'visible-3'])
    expect([...pruned].sort()).toEqual(['visible-1', 'visible-3'])
  })

  it('reports membership', () => {
    const selected = new Set(['x'])
    expect(isMultiSelected(selected, 'x')).toBe(true)
    expect(isMultiSelected(selected, 'y')).toBe(false)
  })
})