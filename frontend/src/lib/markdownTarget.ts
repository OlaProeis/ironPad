import { isInsideRoot, normalizeAbsolutePath, toRelativeSource } from './libraryRoot'

const IMAGE_SUFFIXES = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'svg'])

export function isImagePath(path: string): boolean {
  const slash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  const name = (slash >= 0 ? path.slice(slash + 1) : path).split('?')[0].split('#')[0]
  const dot = name.lastIndexOf('.')
  if (dot <= 0 || dot === name.length - 1) return false
  return IMAGE_SUFFIXES.has(name.slice(dot + 1).toLowerCase())
}

export function libraryMediaUrl(absPath: string): string {
  return `/api/library/media?path=${encodeURIComponent(normalizeAbsolutePath(absPath))}`
}

export function isExternalHref(href: string): boolean {
  const value = href.trim()
  return /^(https?:|mailto:|irc:|data:)/i.test(value) || value.startsWith('//')
}

function decodeHref(href: string): string {
  const trimmed = href.trim().replace(/^<|>$/g, '')
  try {
    return decodeURI(trimmed)
  } catch {
    return trimmed
  }
}

function stripFragment(href: string): string {
  const hash = href.indexOf('#')
  const query = href.indexOf('?')
  let end = href.length
  if (hash >= 0) end = Math.min(end, hash)
  if (query >= 0) end = Math.min(end, query)
  return href.slice(0, end)
}

function parentDir(filePath: string): string {
  const norm = normalizeAbsolutePath(filePath)
  const slash = norm.lastIndexOf('/')
  if (slash <= 0) return norm
  if (slash === 2 && /^[a-z]:$/i.test(norm.slice(0, 2))) {
    return norm.slice(0, 3)
  }
  return norm.slice(0, slash)
}

function joinUnder(baseDir: string, rel: string): string {
  const base = normalizeAbsolutePath(baseDir)
  const drive = /^[a-z]:/i.exec(base)?.[0] ?? ''
  const driveKey = drive.toLowerCase()
  const baseParts = base.split('/').filter((part) => part && part.toLowerCase() !== driveKey)
  const stack = [...baseParts]
  for (const part of rel.replace(/\\/g, '/').split('/')) {
    if (!part || part === '.') continue
    if (part === '..') {
      if (stack.length > 0) stack.pop()
      continue
    }
    stack.push(part)
  }
  if (drive) {
    return `${drive.toLowerCase()}/${stack.join('/')}`
  }
  return `/${stack.join('/')}`
}

/**
 * Resolve a markdown link or image href against the file that contains it.
 * Returns null for in-page anchors and external URLs.
 * A leading `/` is resolved from `root` when the current file lives inside that root.
 */
export function resolveLocalMarkdownPath(
  baseFile: string,
  href: string,
  root: string | null
): string | null {
  const decoded = decodeHref(href)
  if (!decoded || decoded.startsWith('#')) return null
  if (isExternalHref(decoded)) return null
  const pathPart = stripFragment(decoded).trim()
  if (!pathPart || isExternalHref(pathPart)) return null

  let fileUrl = pathPart
  if (/^file:/i.test(fileUrl)) {
    fileUrl = fileUrl.replace(/^file:\/\//i, '')
    if (/^\/[A-Za-z]:/.test(fileUrl)) fileUrl = fileUrl.slice(1)
  }

  const slashed = fileUrl.replace(/\\/g, '/')
  if (/^[A-Za-z]:\//.test(slashed) || slashed.startsWith('//')) {
    return normalizeAbsolutePath(slashed)
  }
  if (slashed.startsWith('/')) {
    if (root && isInsideRoot(root, baseFile)) {
      return joinUnder(root, slashed.replace(/^\/+/, ''))
    }
    return normalizeAbsolutePath(slashed)
  }
  return joinUnder(parentDir(baseFile), slashed)
}

/** Relative source when the file is inside the project root; otherwise an absolute path. */
export function sourceForLibraryFile(root: string | null, absPath: string): string {
  if (root && isInsideRoot(root, absPath)) {
    return toRelativeSource(root, absPath)
  }
  return normalizeAbsolutePath(absPath)
}

export function sameLinkedSource(
  stored: string | undefined,
  absPath: string,
  root: string | null
): boolean {
  if (!stored) return false
  const abs = normalizeAbsolutePath(absPath)
  const storedSlashed = stored.replace(/\\/g, '/')
  const storedIsAbsolute = /^[A-Za-z]:\//.test(storedSlashed) || storedSlashed.startsWith('/')
  if (storedIsAbsolute) {
    return normalizeAbsolutePath(stored) === abs
  }
  if (root && isInsideRoot(root, absPath)) {
    return storedSlashed === toRelativeSource(root, absPath)
  }
  return false
}
