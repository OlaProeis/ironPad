<script setup lang="ts">
import { computed } from 'vue'
import MarkdownIt from 'markdown-it'
import { prepareMarkdownBody } from '../../lib/markdownBody'
import {
  isExternalHref,
  isImagePath,
  libraryMediaUrl,
  resolveLocalMarkdownPath,
} from '../../lib/markdownTarget'

const props = defineProps<{
  source: string
  basePath?: string | null
  linkRoot?: string | null
}>()

const emit = defineEmits<{
  'open-file': [path: string]
}>()

function renderMarkdown(source: string): string {
  const md = new MarkdownIt({
    html: false,
    xhtmlOut: true,
    breaks: true,
    linkify: true,
    typographer: true,
  })
  const basePath = props.basePath
  if (basePath) {
    const defaultImage = md.renderer.rules.image
    md.renderer.rules.image = (tokens, idx, options, env, self) => {
      const token = tokens[idx]
      const src = token.attrGet('src')
      if (src) {
        const file = resolveLocalMarkdownPath(basePath, src, props.linkRoot ?? null)
        if (file && isImagePath(file)) {
          token.attrSet('src', libraryMediaUrl(file))
        }
      }
      if (defaultImage) return defaultImage(tokens, idx, options, env, self)
      return self.renderToken(tokens, idx, options)
    }

    const defaultLinkOpen =
      md.renderer.rules.link_open ||
      ((tokens, idx, options, _env, self) => self.renderToken(tokens, idx, options))
    md.renderer.rules.link_open = (tokens, idx, options, env, self) => {
      const token = tokens[idx]
      const href = token.attrGet('href') ?? ''
      if (isExternalHref(href)) {
        token.attrSet('target', '_blank')
        token.attrSet('rel', 'noopener noreferrer')
      } else {
        const file = resolveLocalMarkdownPath(basePath, href, props.linkRoot ?? null)
        if (file) {
          token.attrSet('href', '#')
          token.attrSet('data-library-path', file)
        }
      }
      return defaultLinkOpen(tokens, idx, options, env, self)
    }
  }
  return md.render(source)
}

const renderedHtml = computed(() => {
  const body = prepareMarkdownBody(props.source)
  if (!body) return ''
  try {
    return renderMarkdown(body)
  } catch (e) {
    console.error('Markdown rendering error:', e)
    return ''
  }
})

function onClick(event: MouseEvent) {
  const target = event.target
  if (!(target instanceof Element)) return
  const anchor = target.closest('a[data-library-path]')
  if (!anchor) return
  event.preventDefault()
  const path = anchor.getAttribute('data-library-path')
  if (path) emit('open-file', path)
}
</script>

<template>
  <div class="rendered-markdown" data-testid="rendered-markdown" @click="onClick">
    <div class="rendered-markdown__body" v-html="renderedHtml" />
  </div>
</template>

<style scoped>
.rendered-markdown {
  overflow-y: auto;
}

.rendered-markdown__body {
  line-height: 1.7;
}

.rendered-markdown__body :deep(h1) {
  font-size: 1.5em;
  font-weight: 600;
  margin: 0.67em 0;
}

.rendered-markdown__body :deep(p) {
  margin: 0.5em 0 1em;
}

.rendered-markdown__body :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
}

.rendered-markdown__body :deep(a:hover) {
  text-decoration: underline;
}

.rendered-markdown__body :deep(code) {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
  font-size: 0.9em;
  padding: 0.2em 0.4em;
  background: var(--color-bg-secondary);
  border-radius: 4px;
}

.rendered-markdown__body :deep(pre) {
  margin: 1em 0;
  padding: 1em;
  background: var(--color-bg-secondary);
  border-radius: 6px;
  overflow-x: auto;
}

.rendered-markdown__body :deep(pre code) {
  padding: 0;
  background: transparent;
}
</style>