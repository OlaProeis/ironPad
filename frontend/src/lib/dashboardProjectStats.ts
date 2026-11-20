import type { Task } from '../types'
import { isTaskOverdue } from './taskDates'

/** Active, incomplete tasks whose stored `due_date` is overdue per shared calendar rules. */
export function countActiveOverdueTasks(
  tasks: Pick<Task, 'completed' | 'is_active' | 'due_date'>[],
  todayLocal: Date,
): number {
  return tasks
    .filter(t => !t.completed && t.is_active)
    .filter(t => isTaskOverdue(t, todayLocal)).length
}