import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  readSortMode,
  sortTopLevelTasks,
  writeSortMode,
  type TaskSortFields,
} from './taskSort'

function task(
  overrides: Partial<TaskSortFields> & { created: string },
): TaskSortFields {
  return {
    due_date: undefined,
    priority: undefined,
    ...overrides,
  }
}

describe('sortTopLevelTasks', () => {
  it('sorts by created newest first', () => {
    const tasks = [
      task({ created: '2024-01-01T10:00:00Z' }),
      task({ created: '2024-03-01T10:00:00Z' }),
      task({ created: '2024-02-01T10:00:00Z' }),
    ]
    const sorted = sortTopLevelTasks(tasks, 'created')
    expect(sorted.map((t) => t.created)).toEqual([
      '2024-03-01T10:00:00Z',
      '2024-02-01T10:00:00Z',
      '2024-01-01T10:00:00Z',
    ])
  })

  it('sorts by due_date earliest first with missing dates last', () => {
    const tasks = [
      task({ created: 'c3', due_date: '2024-06-10' }),
      task({ created: 'c1' }),
      task({ created: 'c2', due_date: '2024-06-01' }),
      task({ created: 'c4', due_date: '2024-06-05' }),
    ]
    const sorted = sortTopLevelTasks(tasks, 'due_date')
    expect(sorted.map((t) => t.created)).toEqual(['c2', 'c4', 'c3', 'c1'])
  })

  it('tie-breaks due_date mode by created descending', () => {
    const tasks = [
      task({ created: '2024-01-02', due_date: '2024-06-01' }),
      task({ created: '2024-01-03', due_date: '2024-06-01' }),
      task({ created: '2024-01-01', due_date: '2024-06-01' }),
    ]
    const sorted = sortTopLevelTasks(tasks, 'due_date')
    expect(sorted.map((t) => t.created)).toEqual([
      '2024-01-03',
      '2024-01-02',
      '2024-01-01',
    ])
  })

  it('treats unparseable due_date like missing (sorted last)', () => {
    const tasks = [
      task({ created: 'old', due_date: 'not-a-date' }),
      task({ created: 'due', due_date: '2024-01-01' }),
    ]
    const sorted = sortTopLevelTasks(tasks, 'due_date')
    expect(sorted.map((t) => t.created)).toEqual(['due', 'old'])
  })

  it('sorts by priority high, medium, low, then unset', () => {
    const tasks = [
      task({ created: 'c-low', priority: 'low' }),
      task({ created: 'c-high', priority: 'high' }),
      task({ created: 'c-none' }),
      task({ created: 'c-med', priority: 'medium' }),
    ]
    const sorted = sortTopLevelTasks(tasks, 'priority')
    expect(sorted.map((t) => t.created)).toEqual([
      'c-high',
      'c-med',
      'c-low',
      'c-none',
    ])
  })

  it('compares priority case-insensitively and treats unknown as unset', () => {
    const tasks = [
      task({ created: 'c-unknown', priority: 'urgent' }),
      task({ created: 'c-HIGH', priority: 'HIGH' }),
      task({ created: 'c-none' }),
    ]
    const sorted = sortTopLevelTasks(tasks, 'priority')
    expect(sorted.map((t) => t.created)).toEqual(['c-HIGH', 'c-unknown', 'c-none'])
  })

  it('tie-breaks priority mode by created descending', () => {
    const tasks = [
      task({ created: '2024-02-01', priority: 'high' }),
      task({ created: '2024-03-01', priority: 'high' }),
    ]
    const sorted = sortTopLevelTasks(tasks, 'priority')
    expect(sorted.map((t) => t.created)).toEqual(['2024-03-01', '2024-02-01'])
  })

  it('does not mutate the input array', () => {
    const tasks = [
      task({ created: 'b' }),
      task({ created: 'a' }),
    ]
    const copy = [...tasks]
    sortTopLevelTasks(tasks, 'created')
    expect(tasks).toEqual(copy)
  })
})

describe('sort mode persistence', () => {
  const store = new Map<string, string>()

  beforeEach(() => {
    store.clear()
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value)
      },
      removeItem: (key: string) => {
        store.delete(key)
      },
    })
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('defaults to created when nothing is stored', () => {
    expect(readSortMode()).toBe('created')
  })

  it('round-trips valid modes via localStorage', () => {
    writeSortMode('due_date')
    expect(readSortMode()).toBe('due_date')
    writeSortMode('priority')
    expect(readSortMode()).toBe('priority')
  })

  it('falls back to created for invalid stored values', () => {
    store.set('ironpad-task-sort', 'bogus')
    expect(readSortMode()).toBe('created')
  })
})