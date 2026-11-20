export interface BundleItem {
  title: string
  path: string
  text: string
}

/** One markdown document: heading, path line, and raw text per item, in order. */
export function buildBundle(items: BundleItem[]): string {
  if (items.length === 0) return ''
  return items
    .map((item) => `## ${item.title}\n\n${item.path}\n\n${item.text}\n\n`)
    .join('')
    .replace(/\n$/, '')
}
