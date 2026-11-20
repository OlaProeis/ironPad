/** Bidi marks, zero-width characters, and a BOM. Windows paste often prefixes a path with U+202A. */
const PATH_NOISE = /[\u202A-\u202E\u2066-\u2069\u200B-\u200F\uFEFF\u061C]/g

function storageKey(projectId: string): string {
  return `ironpad-library-root:${projectId}`
}

function stripTrailingSeparators(path: string): string {
  if (/^[A-Za-z]:[\\/]?$/.test(path) || path === '/' || path === '\\') {
    return path
  }
  return path.replace(/[\\/]+$/, '')
}

/**
 * Drop invisible paste artifacts and one pair of wrapping quotes from Copy as path.
 * Does not change slash direction.
 */
export function sanitizeUserPath(path: string): string {
  let s = path.replace(PATH_NOISE, '').trim()
  if (
    (s.startsWith('"') && s.endsWith('"') && s.length >= 2) ||
    (s.startsWith("'") && s.endsWith("'") && s.length >= 2)
  ) {
    s = s.slice(1, -1).trim()
  }
  return stripTrailingSeparators(s)
}

export function getLibraryRoot(projectId: string): string | null {
  try {
    if (typeof localStorage === 'undefined') return null
    const key = storageKey(projectId)
    const raw = localStorage.getItem(key)
    if (!raw) return null
    const cleaned = sanitizeUserPath(raw)
    if (!cleaned) {
      localStorage.removeItem(key)
      return null
    }
    if (cleaned !== raw) {
      localStorage.setItem(key, cleaned)
    }
    return cleaned
  } catch {
    return null
  }
}

export function setLibraryRoot(projectId: string, path: string): void {
  try {
    if (typeof localStorage === 'undefined') return
    const cleaned = sanitizeUserPath(path)
    if (!cleaned) return
    localStorage.setItem(storageKey(projectId), cleaned)
  } catch {
    // private mode / blocked storage
  }
}

export function clearLibraryRoot(projectId: string): void {
  try {
    if (typeof localStorage === 'undefined') return
    localStorage.removeItem(storageKey(projectId))
  } catch {
    // private mode / blocked storage
  }
}

function normalizeSlashes(path: string): string {
  return path.replace(/\\/g, '/')
}

/** Lowercase drive letter; forward slashes; no trailing slash (except bare `X:/`). */
export function normalizeAbsolutePath(path: string): string {
  let s = normalizeSlashes(sanitizeUserPath(path))
  // Windows canonicalize() prefixes `\\?\`, which becomes `//?/` after slash normalization.
  if (s.toLowerCase().startsWith('//?/unc/')) {
    s = `//${s.slice('//?/unc/'.length)}`
  } else if (s.startsWith('//?/')) {
    s = s.slice(4)
  }
  const drive = /^([A-Za-z]):/.exec(s)
  if (drive?.[1]) {
    s = `${drive[1].toLowerCase()}:${s.slice(drive[0].length)}`
  }
  if (s.length > 3 && s.endsWith('/')) {
    s = s.replace(/\/+$/, '')
  }
  return s
}

export function isInsideRoot(root: string, absPath: string): boolean {
  const normRoot = normalizeAbsolutePath(root)
  const normAbs = normalizeAbsolutePath(absPath)
  if (normAbs === normRoot) return true
  if (!normAbs.startsWith(normRoot)) return false
  const next = normAbs.charAt(normRoot.length)
  return next === '/'
}

export function toRelativeSource(root: string, absPath: string): string {
  const normRoot = normalizeAbsolutePath(root)
  const normAbs = normalizeAbsolutePath(absPath)
  let relative: string
  if (normAbs === normRoot) {
    relative = ''
  } else if (normAbs.startsWith(normRoot) && normAbs.charAt(normRoot.length) === '/') {
    relative = normAbs.slice(normRoot.length + 1)
  } else {
    relative = normalizeSlashes(absPath)
  }
  return relative.replace(/^\/+/, '')
}