import { describe, expect, it } from 'vitest'
import {
  clearSelectedDashboardTag,
  computeProjectCardStats,
  filterTasksByTag,
  shouldShowTagFilterEmptyState,
} from './dashboardTagFilter'

const todayLocal = new Date(2024, 5, 20, 12, 0, 0)

describe('filterTasksByTag', () => {
  it('returns only incomplete tasks whose tags include the filter (exact match)', () => {
    const tasks = [
      { completed: false, tags: ['work'] as string[], is_active: true, due_date: undefined },
      { completed: false, tags: ['other'], is_active: true, due_date: undefined },
      { completed: true, tags: ['work'], is_active: true, due_date: undefined },
    ]
    expect(filterTasksByTag(tasks, 'work')).toHaveLength(1)
    expect(filterTasksByTag(tasks, 'work')[0].tags).toEqual(['work'])
  })

  it('is case-sensitive', () => {
    const tasks = [
      { completed: false, tags: ['Bug'] },
      { completed: false, tags: ['bug'] },
    ]
    expect(filterTasksByTag(tasks, 'Bug')).toHaveLength(1)
    expect(filterTasksByTag(tasks, 'bug')).toHaveLength(1)
    expect(filterTasksByTag(tasks, 'BUG')).toHaveLength(0)
  })

  it('treats missing tags as no match', () => {
    const tasks = [{ completed: false }]
    expect(filterTasksByTag(tasks, 'any')).toEqual([])
  })
})

describe('computeProjectCardStats', () => {
  it('splits active, backlog, and completed using existing section rules', () => {
    const tasks = [
      { completed: false, is_active: true, due_date: '2024-06-25' },
      { completed: false, is_active: true, due_date: '2024-06-25' },
      { completed: false, is_active: false, due_date: undefined },
      { completed: true, is_active: false, due_date: undefined },
    ]
    const stats = computeProjectCardStats(tasks, todayLocal)
    expect(stats.activeTasks).toHaveLength(2)
    expect(stats.backlogCount).toBe(1)
    expect(stats.completedCount).toBe(1)
    expect(stats.totalCount).toBe(4)
    expect(stats.overdueCount).toBe(0)
  })

  it('recomputes overdue from the provided list only', () => {
    const tasks = [
      { completed: false, is_active: true, due_date: '2024-06-19' },
      { completed: false, is_active: true, due_date: '2024-06-21' },
    ]
    expect(computeProjectCardStats(tasks, todayLocal).overdueCount).toBe(1)
  })

  it('reflects filtered subset after tag filter', () => {
    const all = [
      {
        completed: false,
        tags: ['work'],
        is_active: true,
        due_date: '2024-06-19',
      },
      {
        completed: false,
        tags: ['home'],
        is_active: false,
        due_date: undefined,
      },
    ]
    const filtered = filterTasksByTag(all, 'work')
    const stats = computeProjectCardStats(filtered, todayLocal)
    expect(stats.activeTasks).toHaveLength(1)
    expect(stats.backlogCount).toBe(0)
    expect(stats.completedCount).toBe(0)
    expect(stats.overdueCount).toBe(1)
    expect(stats.totalCount).toBe(1)
  })
})

describe('shouldShowTagFilterEmptyState', () => {
  it('is true when a tag is selected and no project cards are visible', () => {
    expect(shouldShowTagFilterEmptyState('work', 0)).toBe(true)
  })

  it('is false when no tag is selected', () => {
    expect(shouldShowTagFilterEmptyState(null, 0)).toBe(false)
  })

  it('is false when cards remain visible', () => {
    expect(shouldShowTagFilterEmptyState('work', 2)).toBe(false)
  })
})

describe('clearSelectedDashboardTag', () => {
  it('clears the selected tag', () => {
    expect(clearSelectedDashboardTag('work')).toBeNull()
    expect(clearSelectedDashboardTag(null)).toBeNull()
  })
})