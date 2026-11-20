/**
 * Removes a leading YAML frontmatter block (`---` … `---`) only when it starts at offset 0.
 */
export function stripFrontmatter(text: string): string {
  const match = text.match(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/)
  if (!match) return text
  return text.slice(match[0].length)
}

/**
 * Strip raw HTML patterns that should never be rendered, before markdown-it (html: false) runs.
 */
export function sanitizeMarkdownSource(text: string): string {
  return text
    .replace(/<script\b[\s\S]*?<\/script>/gi, '')
    .replace(/\bonerror\s*=\s*[^\s>'"]+/gi, '')
}

export function prepareMarkdownBody(text: string): string {
  return sanitizeMarkdownSource(stripFrontmatter(text))
}