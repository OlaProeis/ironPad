import { tasksApi } from '../api/client'

export interface BulkTaskRef {
  id: string
  title: string
  completed: boolean
  tags?: string[]
}

export interface BulkMetaUpdate {
  id: string
  title: string
  meta: {
    section?: string
    tags?: string[]
    completed?: boolean
  }
}

export interface BulkRunResult {
  ok: boolean
  /** How many requests finished before stop. */
  applied: number
  /** Task title and error text when a request fails. */
  failure?: { title: string; error: string }
}

function errorText(err: unknown): string {
  return err instanceof Error ? err.message : String(err)
}

/** Incomplete tasks only. Toggle endpoint preserves recurrence on complete. */
export function tasksForBulkComplete<T extends { completed: boolean }>(tasks: T[]): T[] {
  return tasks.filter((task) => !task.completed)
}

export function normalizeTag(raw: string): string | null {
  const tag = raw.trim()
  return tag.length > 0 ? tag : null
}

/** Full tags array with the tag appended once. */
export function tagsAfterAdd(tags: string[] | undefined, tag: string): string[] {
  const next = [...(tags ?? [])]
  if (!next.includes(tag)) {
    next.push(tag)
  }
  return next
}

/** Full tags array with the tag removed. */
export function tagsAfterRemove(tags: string[] | undefined, tag: string): string[] {
  return (tags ?? []).filter((existing) => existing !== tag)
}

/**
 * Section move. Completed tasks also send `completed: false` so they leave Completed.
 */
export function metaForSectionMove(
  task: { completed: boolean },
  section: 'Active' | 'Backlog',
): { section: 'Active' | 'Backlog'; completed?: false } {
  if (task.completed) {
    return { section, completed: false }
  }
  return { section }
}

async function runSequential<T extends { title: string }>(
  items: T[],
  run: (item: T) => Promise<void>,
): Promise<BulkRunResult> {
  let applied = 0
  for (const item of items) {
    try {
      await run(item)
      applied += 1
    } catch (err) {
      return {
        ok: false,
        applied,
        failure: { title: item.title, error: errorText(err) },
      }
    }
  }
  return { ok: true, applied }
}

/** Sequential PUT .../toggle. Already-completed tasks are omitted. */
export async function runBulkComplete(
  projectId: string,
  tasks: BulkTaskRef[],
): Promise<BulkRunResult> {
  const pending = tasksForBulkComplete(tasks)
  return runSequential(pending, (task) => tasksApi.toggle(projectId, task.id).then(() => undefined))
}

/** Sequential PUT .../meta. Stops on the first failure and leaves earlier saves. */
export async function runBulkMetaUpdates(
  projectId: string,
  updates: BulkMetaUpdate[],
): Promise<BulkRunResult> {
  return runSequential(updates, (update) =>
    tasksApi.updateMeta(projectId, update.id, update.meta).then(() => undefined),
  )
}
