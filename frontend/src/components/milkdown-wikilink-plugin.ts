/**
 * WikiLink Plugin for Milkdown
 *
 * This plugin provides support for wiki-style links: [[note-id]] or [[note-id|display text]]
 * It works by intercepting the markdown serialization/deserialization process.
 */

/**
 * Regex pattern for wiki-style links
 * Matches: [[note-id]] or [[note-id|display text]]
 * Also handles escaped versions like \[\[note-id]] that Milkdown may produce
 */
export const WIKILINK_REGEX = /\\?\[\\?\[([^\[\]]+?)(?:\|([^\[\]]*?))?\\?\]\\?\]/
export const WIKILINK_REGEX_GLOBAL = /\\?\[\\?\[([^\[\]]+?)(?:\|([^\[\]]*?))?\\?\]\\?\]/g

/**
 * Parse a wikilink string and return target and display text
 */
export function parseWikilink(text: string): { target: string; display: string } | null {
  const match = text.match(WIKILINK_REGEX)
  if (!match) return null

  const target = match[1]?.trim()
  if (!target) return null

  const display = match[2]?.trim() || target

  return { target, display }
}

/**
 * Create a wikilink string from target and optional display text
 */
export function createWikilink(target: string, display?: string): string {
  if (display && display !== target) {
    return `[[${target}|${display}]]`
  }
  return `[[${target}]]`
}

/**
 * Convert wikilinks in markdown text to regular markdown links
 * This is called before the markdown is parsed by Milkdown
 */
export function preprocessWikilinks(markdown: string): string {
  if (!markdown) return ''
  
  const result = markdown.replace(WIKILINK_REGEX_GLOBAL, (_match, target, display) => {
    const targetStr = String(target || '').trim()
    const displayStr = String(display || '').trim()
    const alias = displayStr || targetStr
    // Convert to a markdown link that Milkdown can understand
    // Use a hash link for client-side navigation
    return `[${alias}](#/note/${encodeURIComponent(targetStr)})`
  })
  
  console.log('[WikilinkPlugin] Preprocessed:', {
    original: markdown?.substring(0, 100),
    result: result?.substring(0, 100),
    changed: markdown !== result
  })
  
  return result
}

/**
 * Handle link clicks for wikilinks
 * Intercepts clicks on links that point to #/note/... and navigates to them
 */
export function setupWikilinkClickHandler(editorView: HTMLElement, router: { push: (path: object) => void }) {
  console.log('[WikilinkPlugin] Setting up click handler on:', editorView)
  
  const handler = (e: MouseEvent) => {
    const target = e.target as HTMLElement
    const link = target.closest('a[href^="#/note/"]') as HTMLAnchorElement | null
    
    console.log('[WikilinkPlugin] Click detected:', { 
      target: target?.tagName, 
      link: link?.getAttribute('href'),
      isWikilink: !!link 
    })

    if (link) {
      e.preventDefault()
      e.stopPropagation()

      const href = link.getAttribute('href')
      if (href) {
        const noteId = decodeURIComponent(href.replace('#/note/', ''))
        console.log('[WikilinkPlugin] Navigating to note:', noteId)
        router.push({ name: 'note', params: { id: noteId } })
      }
    }
  }

  editorView.addEventListener('click', handler)

  // Return cleanup function
  return () => {
    editorView.removeEventListener('click', handler)
    console.log('[WikilinkPlugin] Click handler removed')
  }
}

/**
 * Post-process markdown to convert regular links back to wikilinks
 * This is called when serializing markdown for saving
 */
export function postprocessWikilinks(markdown: string): string {
  if (!markdown) return ''
  
  // Find markdown links that were originally wikilinks (point to #/note/...)
  const result = markdown.replace(/\[([^\]]+)\]\(#\/note\/([^)]+)\)/g, (match, display, encodedTarget) => {
    try {
      const target = decodeURIComponent(encodedTarget)
      // Only convert back if the display matches the target or is different
      return createWikilink(target, display !== target ? display : undefined)
    } catch {
      return match
    }
  })
  
  console.log('[WikilinkPlugin] Postprocessed:', {
    original: markdown?.substring(0, 100),
    result: result?.substring(0, 100),
    changed: markdown !== result
  })
  
  return result
}
