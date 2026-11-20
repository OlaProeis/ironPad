import { describe, expect, it } from 'vitest'
import { buildTagChipSummary } from './dashboardTagChips'

describe('buildTagChipSummary', () => {
  it('merges duplicate tags across tasks into one chip count', () => {
    const allTasks = [
      { completed: false, tags: ['work'] },
      { completed: false, tags: ['work', 'urgent'] },
      { completed: false, tags: ['work'] },
    ]
    const summary = buildTagChipSummary(allTasks)
    const work = summary.find(s => s.tag === 'work')
    expect(work?.count).toBe(3)
    expect(summary.map(s => s.tag)).toEqual(['urgent', 'work'])
  })

  it('treats tags as case-sensitive distinct labels', () => {
    const allTasks = [
      { completed: false, tags: ['Bug'] },
      { completed: false, tags: ['bug'] },
    ]
    const summary = buildTagChipSummary(allTasks)
    expect(summary).toHaveLength(2)
    expect(summary).toContainEqual({ tag: 'Bug', count: 1 })
    expect(summary).toContainEqual({ tag: 'bug', count: 1 })
  })

  it('excludes completed tasks from chip counts', () => {
    const allTasks = [
      { completed: true, tags: ['done-tag'] },
      { completed: false, tags: ['open-tag'] },
    ]
    expect(buildTagChipSummary(allTasks)).toEqual([{ tag: 'open-tag', count: 1 }])
  })

  it('counts a multi-tag task once on each of its tags', () => {
    const allTasks = [{ completed: false, tags: ['a', 'b', 'c'] }]
    expect(buildTagChipSummary(allTasks)).toEqual([
      { tag: 'a', count: 1 },
      { tag: 'b', count: 1 },
      { tag: 'c', count: 1 },
    ])
  })
})