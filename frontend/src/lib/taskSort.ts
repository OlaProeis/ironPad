import type { Task } from '../types'
import { parseDueDateOnly } from './taskDates'

export type TaskSortMode = 'created' | 'due_date' | 'priority'

const STORAGE_KEY = 'ironpad-task-sort'

const VALID_MODES = new Set<TaskSortMode>(['created', 'due_date', 'priority'])

export type TaskSortFields = Pick<Task, 'created' | 'due_date' | 'priority'>

function compareCreatedDesc(a: string, b: string): number {
  return b.localeCompare(a)
}

function priorityRank(priority: string | undefined): number {
  if (!priority?.trim()) return 3
  const normalized = priority.trim().toLowerCase()
  if (normalized === 'high') return 0
  if (normalized === 'medium') return 1
  if (normalized === 'low') return 2
  return 3
}

function dueDateSortKey(dueDate: string | undefined): string | null {
  if (!dueDate) return null
  return parseDueDateOnly(dueDate)
}

function compareDueDate(a: TaskSortFields, b: TaskSortFields): number {
  const aKey = dueDateSortKey(a.due_date)
  const bKey = dueDateSortKey(b.due_date)

  if (aKey === null && bKey === null) {
    return compareCreatedDesc(a.created, b.created)
  }
  if (aKey === null) return 1
  if (bKey === null) return -1

  const byDate = aKey.localeCompare(bKey)
  if (byDate !== 0) return byDate
  return compareCreatedDesc(a.created, b.created)
}

function comparePriority(a: TaskSortFields, b: TaskSortFields): number {
  const byRank = priorityRank(a.priority) - priorityRank(b.priority)
  if (byRank !== 0) return byRank
  return compareCreatedDesc(a.created, b.created)
}

/** Sort top-level tasks; returns a new array without mutating the input. */
export function sortTopLevelTasks<T extends TaskSortFields>(
  tasks: T[],
  mode: TaskSortMode,
): T[] {
  const out = [...tasks]
  switch (mode) {
    case 'created':
      out.sort((a, b) => compareCreatedDesc(a.created, b.created))
      break
    case 'due_date':
      out.sort(compareDueDate)
      break
    case 'priority':
      out.sort(comparePriority)
      break
  }
  return out
}

export function readSortMode(): TaskSortMode {
  try {
    if (typeof localStorage === 'undefined') return 'created'
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored && VALID_MODES.has(stored as TaskSortMode)) {
      return stored as TaskSortMode
    }
  } catch {
    // private mode / blocked storage
  }
  return 'created'
}

export function writeSortMode(mode: TaskSortMode): void {
  try {
    if (typeof localStorage === 'undefined') return
    localStorage.setItem(STORAGE_KEY, mode)
  } catch {
    // private mode / blocked storage
  }
}