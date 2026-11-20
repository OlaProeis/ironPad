import type { Task } from '../types'

export interface TagChipSummaryItem {
  tag: string
  count: number
}

/** Distinct frontmatter tags across incomplete tasks, sorted A–Z with per-tag counts. */
export function buildTagChipSummary(
  allTasks: Pick<Task, 'completed' | 'tags'>[],
): TagChipSummaryItem[] {
  const counts = new Map<string, number>()

  for (const task of allTasks) {
    if (task.completed) continue
    for (const tag of task.tags ?? []) {
      counts.set(tag, (counts.get(tag) ?? 0) + 1)
    }
  }

  return Array.from(counts.entries())
    .map(([tag, count]) => ({ tag, count }))
    .sort((a, b) => a.tag.localeCompare(b.tag))
}