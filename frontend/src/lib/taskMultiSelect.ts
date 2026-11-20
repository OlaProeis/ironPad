/** Toggle bulk-selection for a top-level task id (immutable Set return). */
export function toggleMultiSelect(selectedIds: Set<string>, taskId: string): Set<string> {
  const next = new Set(selectedIds)
  if (next.has(taskId)) {
    next.delete(taskId)
  } else {
    next.add(taskId)
  }
  return next
}

export function clearMultiSelect(): Set<string> {
  return new Set()
}

/** Drop ids that are not currently visible (e.g. hidden by tag filter). */
export function pruneMultiSelect(
  selectedIds: Set<string>,
  visibleTopLevelIds: Iterable<string>,
): Set<string> {
  const visible = new Set(visibleTopLevelIds)
  const next = new Set<string>()
  for (const id of selectedIds) {
    if (visible.has(id)) {
      next.add(id)
    }
  }
  return next
}

export function isMultiSelected(selectedIds: Set<string>, taskId: string): boolean {
  return selectedIds.has(taskId)
}