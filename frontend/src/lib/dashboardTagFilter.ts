import type { Task } from '../types'
import { countActiveOverdueTasks } from './dashboardProjectStats'

/** Incomplete tasks with an exact, case-sensitive tag match (same rule as TasksView tag filter). */
export function filterTasksByTag<T extends Pick<Task, 'completed' | 'tags'>>(
  tasks: T[],
  tag: string,
): T[] {
  return tasks.filter(t => !t.completed && (t.tags ?? []).includes(tag))
}

export interface ProjectCardStats<T extends Pick<Task, 'completed' | 'is_active' | 'due_date'>> {
  activeTasks: T[]
  backlogCount: number
  completedCount: number
  overdueCount: number
  totalCount: number
}

/** Per-card active, backlog, completed, and overdue counts from a project task list. */
export function computeProjectCardStats<T extends Pick<Task, 'completed' | 'is_active' | 'due_date'>>(
  projectTasks: T[],
  todayLocal: Date,
): ProjectCardStats<T> {
  const active = projectTasks.filter(t => !t.completed && t.is_active)
  const backlog = projectTasks.filter(t => !t.completed && !t.is_active)
  const completed = projectTasks.filter(t => t.completed)

  return {
    activeTasks: active,
    backlogCount: backlog.length,
    completedCount: completed.length,
    overdueCount: countActiveOverdueTasks(projectTasks, todayLocal),
    totalCount: projectTasks.length,
  }
}

/** True when a tag filter is active and no project cards remain visible. */
export function shouldShowTagFilterEmptyState(
  selectedTag: string | null,
  visibleProjectCardCount: number,
): boolean {
  return selectedTag != null && selectedTag.length > 0 && visibleProjectCardCount === 0
}

/** Resets tag filter selection (restore full dashboard). */
export function clearSelectedDashboardTag(
  selectedTag: string | null,
): string | null {
  return null
}