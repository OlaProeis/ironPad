/**
 * Link Handler for Milkdown
 * 
 * Handles click events on standard markdown links and navigates to internal notes.
 * Detects links in the format [text](note-id) where note-id is an internal note reference.
 */

import type { Router } from 'vue-router'

/**
 * Extract note ID from a link target
 * Handles various formats:
 * - Simple note ID: `note-id`
 * - Project-relative path: `../project/notes/file.md`
 * - Hash-based navigation: `#/note/note-id`
 */
function extractNoteId(href: string): string | null {
  // Handle hash-based navigation from old wikilinks
  if (href.includes('#/note/')) {
    const match = href.match(/#\/note\/([^?&]+)/)
    if (match && match[1]) {
      return decodeURIComponent(match[1])
    }
  }
  
  // Remove hash fragment
  const hashIdx = href.indexOf('#')
  const url = hashIdx >= 0 ? href.substring(0, hashIdx) : href
  
  // Handle relative paths like ../project/notes/20260226.md
  if (url.includes('/notes/') || url.includes('\\notes\\')) {
    const match = url.match(/notes[\\/]([^\\/]+)\.md$/)
    if (match && match[1]) {
      return match[1]
    }
  }
  
  // Handle plain note IDs (alphanumeric with dashes/underscores)
  // Exclude URLs and absolute paths
  if (!url.startsWith('http') && !url.startsWith('/') && !url.startsWith('//')) {
    // Extract the last path component if there's a /
    const parts = url.split(/[\\/]/)
    const lastPart = parts[parts.length - 1]
    
    if (!lastPart) return null
    
    // Remove .md extension if present
    const noteId = lastPart.replace(/\.md$/, '')
    
    // Validate it looks like a note ID
    if (noteId.length >= 3 && /^[a-zA-Z0-9_-]+$/.test(noteId)) {
      return noteId
    }
  }
  
  return null
}

/**
 * Check if a link is an internal note link
 */
function isInternalNoteLink(href: string): boolean {
  // Exclude external URLs
  if (href.startsWith('http://') || href.startsWith('https://') || href.startsWith('//')) {
    return false
  }
  
  // Exclude mailto and other protocols
  if (href.includes(':') && !href.includes('://')) {
    return false
  }
  
  // Check if we can extract a note ID
  return extractNoteId(href) !== null
}

/**
 * Setup click handler for links in the editor
 */
export function setupLinkClickHandler(
  editorView: HTMLElement, 
  router: Router
): () => void {
  console.log('[LinkHandler] Setting up click handler on:', editorView)
  
  const handler = (e: MouseEvent) => {
    const target = e.target as HTMLElement
    const link = target.closest('a') as HTMLAnchorElement | null
    
    if (!link) return
    
    const href = link.getAttribute('href')
    if (!href) return
    
    // Check if this is an internal note link
    if (!isInternalNoteLink(href)) {
      // Let external links open normally
      return
    }
    
    console.log('[LinkHandler] Internal link clicked:', href)
    
    e.preventDefault()
    e.stopPropagation()
    
    const noteId = extractNoteId(href)
    if (!noteId) {
      console.warn('[LinkHandler] Could not extract note ID from:', href)
      return
    }
    
    // Determine if this is a project note from the current route
    const currentRoute = router.currentRoute.value
    const isProjectNote = currentRoute.name === 'project-notes' || 
                          currentRoute.path.includes('/projects/')
    const currentProjectId = currentRoute.params.id as string | undefined
    
    console.log('[LinkHandler] Navigating to note:', { 
      noteId, 
      isProjectNote, 
      currentProjectId,
      currentPath: currentRoute.path 
    })
    
    // Navigate to the note
    // For now, assume links within the same project context
    // Cross-project links would need additional resolution logic
    if (isProjectNote && currentProjectId) {
      router.push({
        name: 'project-notes',
        params: { 
          id: currentProjectId, 
          noteId: noteId 
        }
      })
    } else {
      // Regular note navigation
      router.push({ 
        name: 'note', 
        params: { id: noteId } 
      })
    }
  }
  
  editorView.addEventListener('click', handler)
  
  // Return cleanup function
  return () => {
    editorView.removeEventListener('click', handler)
    console.log('[LinkHandler] Click handler removed')
  }
}
