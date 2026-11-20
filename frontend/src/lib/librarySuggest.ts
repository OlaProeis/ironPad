import type { BrowseResponse } from '../types'
import { normalizeAbsolutePath } from './libraryRoot'

export const SUGGESTED = ['ai-context.md', 'ROADMAP.md', 'docs/index.md'] as const

function dismissKey(projectId: string): string {
  return `ironpad-library-suggest-dismissed:${projectId}`
}

export function isSuggestDismissed(projectId: string): boolean {
  try {
    if (typeof localStorage === 'undefined') return false
    return localStorage.getItem(dismissKey(projectId)) === '1'
  } catch {
    return false
  }
}

export function dismissSuggest(projectId: string): void {
  try {
    if (typeof localStorage === 'undefined') return
    localStorage.setItem(dismissKey(projectId), '1')
  } catch {
    // private mode / blocked storage
  }
}

export type BrowseFn = (path: string) => Promise<BrowseResponse>

function entryForSuggested(
  relative: (typeof SUGGESTED)[number],
  rootListing: BrowseResponse | null,
  docsListing: BrowseResponse | null
): boolean {
  if (relative === 'docs/index.md') {
    return !!docsListing?.entries.find(
      (e) => !e.is_dir && e.name === 'index.md' && e.linkable
    )
  }
  return !!rootListing?.entries.find((e) => !e.is_dir && e.name === relative && e.linkable)
}

export async function findSuggestions(
  root: string,
  existingSources: string[],
  browse: BrowseFn
): Promise<string[]> {
  const existing = new Set(existingSources)
  const normalized = normalizeAbsolutePath(root)
  if (!normalized) return []

  let rootListing: BrowseResponse | null = null
  let docsListing: BrowseResponse | null = null

  try {
    rootListing = await browse(normalized)
  } catch {
    // ignore browse errors
  }

  try {
    docsListing = await browse(`${normalized}/docs`)
  } catch {
    // ignore browse errors
  }

  const found: string[] = []
  for (const rel of SUGGESTED) {
    if (existing.has(rel)) continue
    if (entryForSuggested(rel, rootListing, docsListing)) {
      found.push(rel)
    }
  }
  return found
}

export function titleForSuggestedSource(relative: string): string {
  const slash = relative.lastIndexOf('/')
  return slash >= 0 ? relative.slice(slash + 1) : relative
}