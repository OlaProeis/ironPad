export type ReadingMode = 'raw' | 'rendered'

export interface ReadingModeItem {
  kind?: string
  supports_rendered?: boolean
}

function storageKey(itemId: string): string {
  return `ironpad-library-mode:${itemId}`
}

export function defaultReadingMode(item: ReadingModeItem): ReadingMode {
  if (item.kind === 'link' && item.supports_rendered) return 'rendered'
  return 'raw'
}

export function getReadingMode(itemId: string, fallback: ReadingMode): ReadingMode {
  try {
    if (typeof localStorage === 'undefined') return fallback
    const stored = localStorage.getItem(storageKey(itemId))
    if (stored === 'raw' || stored === 'rendered') return stored
    return fallback
  } catch {
    return fallback
  }
}

export function setReadingMode(itemId: string, mode: ReadingMode): void {
  try {
    if (typeof localStorage === 'undefined') return
    localStorage.setItem(storageKey(itemId), mode)
  } catch {
    // private mode / blocked storage
  }
}
