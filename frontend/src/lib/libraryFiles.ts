export const LINK_SUFFIXES = new Set([
  'md',
  'markdown',
  'txt',
  'rs',
  'vue',
  'ts',
  'js',
  'toml',
  'json',
  'yml',
  'yaml',
  'py',
  'css',
  'html',
  'mdc',
])

export function fileTitle(path: string): string {
  const norm = path.replace(/\\/g, '/').replace(/\/+$/, '')
  const slash = norm.lastIndexOf('/')
  return slash >= 0 ? norm.slice(slash + 1) : norm
}

export function suffixOfPath(path: string): string | null {
  const name = fileTitle(path).split('?')[0].split('#')[0]
  const dot = name.lastIndexOf('.')
  if (dot <= 0 || dot === name.length - 1) return null
  return name.slice(dot + 1).toLowerCase()
}

export function isLinkablePath(path: string): boolean {
  const suffix = suffixOfPath(path)
  return suffix != null && LINK_SUFFIXES.has(suffix)
}
