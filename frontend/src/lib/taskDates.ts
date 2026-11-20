import type { Task } from '../types'

const ISO_DATE_ONLY = /^(\d{4})-(\d{2})-(\d{2})$/

/** Parse a calendar `YYYY-MM-DD` due date without interpreting it as UTC midnight. */
export function parseDueDateOnly(dueDate: string): string | null {
  const match = ISO_DATE_ONLY.exec(dueDate.trim())
  if (!match) return null

  const year = Number(match[1])
  const month = Number(match[2])
  const day = Number(match[3])
  const probe = new Date(year, month - 1, day)
  if (
    probe.getFullYear() !== year ||
    probe.getMonth() !== month - 1 ||
    probe.getDate() !== day
  ) {
    return null
  }

  return `${match[1]}-${match[2]}-${match[3]}`
}

export function formatLocalCalendarDate(date: Date): string {
  const y = date.getFullYear()
  const m = String(date.getMonth() + 1).padStart(2, '0')
  const d = String(date.getDate()).padStart(2, '0')
  return `${y}-${m}-${d}`
}

/**
 * True when an incomplete task's due calendar date is strictly before `todayLocal`
 * in the local timezone. Uses calendar-date comparison, not `Date` parsing of due strings.
 */
export function isTaskOverdue(
  task: Pick<Task, 'completed' | 'due_date'>,
  todayLocal: Date,
): boolean {
  if (task.completed) return false
  if (!task.due_date) return false

  const dueCalendar = parseDueDateOnly(task.due_date)
  if (!dueCalendar) return false

  const todayCalendar = formatLocalCalendarDate(todayLocal)
  return dueCalendar < todayCalendar
}