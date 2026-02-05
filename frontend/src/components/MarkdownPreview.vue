<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import MarkdownIt from 'markdown-it'

const props = defineProps<{
  content: string
}>()

// Initialize markdown-it with CommonMark preset
const md = ref<MarkdownIt | null>(null)

onMounted(() => {
  md.value = new MarkdownIt({
    html: false,        // Disable HTML tags in source
    xhtmlOut: true,     // Use '/' to close single tags (<br />)
    breaks: true,       // Convert '\n' in paragraphs into <br>
    linkify: true,      // Autoconvert URL-like text to links
    typographer: true,  // Enable smartquotes and other typographic replacements
  })
})

const renderedHtml = computed(() => {
  if (!md.value) return ''
  
  try {
    return md.value.render(props.content)
  } catch (e) {
    console.error('Markdown rendering error:', e)
    return `<pre>${props.content}</pre>`
  }
})
</script>

<template>
  <div class="markdown-preview">
    <div class="preview-content" v-html="renderedHtml"></div>
  </div>
</template>

<style scoped>
.markdown-preview {
  flex: 1;
  overflow-y: auto;
  padding: 16px 24px;
  background: var(--color-bg);
}

.preview-content {
  max-width: 800px;
  line-height: 1.7;
}

/* Markdown styling */
.preview-content :deep(h1) {
  font-size: 2em;
  font-weight: 600;
  margin: 0.67em 0;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--color-border);
}

.preview-content :deep(h2) {
  font-size: 1.5em;
  font-weight: 600;
  margin: 1em 0 0.5em;
  padding-bottom: 0.3em;
  border-bottom: 1px solid var(--color-border);
}

.preview-content :deep(h3) {
  font-size: 1.25em;
  font-weight: 600;
  margin: 1em 0 0.5em;
}

.preview-content :deep(h4),
.preview-content :deep(h5),
.preview-content :deep(h6) {
  font-size: 1em;
  font-weight: 600;
  margin: 1em 0 0.5em;
}

.preview-content :deep(p) {
  margin: 0.5em 0 1em;
}

.preview-content :deep(ul),
.preview-content :deep(ol) {
  margin: 0.5em 0 1em;
  padding-left: 2em;
}

.preview-content :deep(li) {
  margin: 0.25em 0;
}

.preview-content :deep(li > ul),
.preview-content :deep(li > ol) {
  margin: 0.25em 0;
}

.preview-content :deep(blockquote) {
  margin: 1em 0;
  padding: 0.5em 1em;
  border-left: 4px solid var(--color-primary);
  background: var(--color-bg-secondary);
  color: var(--color-text-secondary);
}

.preview-content :deep(blockquote p) {
  margin: 0;
}

.preview-content :deep(code) {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
  font-size: 0.9em;
  padding: 0.2em 0.4em;
  background: var(--color-bg-secondary);
  border-radius: 4px;
}

.preview-content :deep(pre) {
  margin: 1em 0;
  padding: 1em;
  background: var(--color-bg-secondary);
  border-radius: 6px;
  overflow-x: auto;
}

.preview-content :deep(pre code) {
  padding: 0;
  background: transparent;
  font-size: 0.85em;
  line-height: 1.5;
}

.preview-content :deep(hr) {
  margin: 2em 0;
  border: none;
  border-top: 1px solid var(--color-border);
}

.preview-content :deep(a) {
  color: var(--color-primary);
  text-decoration: none;
}

.preview-content :deep(a:hover) {
  text-decoration: underline;
}

.preview-content :deep(img) {
  max-width: 100%;
  height: auto;
  border-radius: 6px;
}

.preview-content :deep(table) {
  width: 100%;
  margin: 1em 0;
  border-collapse: collapse;
}

.preview-content :deep(th),
.preview-content :deep(td) {
  padding: 0.5em 1em;
  border: 1px solid var(--color-border);
  text-align: left;
}

.preview-content :deep(th) {
  background: var(--color-bg-secondary);
  font-weight: 600;
}

.preview-content :deep(tr:nth-child(even)) {
  background: var(--color-bg-secondary);
}

/* Task list styling */
.preview-content :deep(input[type="checkbox"]) {
  margin-right: 0.5em;
}

.preview-content :deep(li.task-list-item) {
  list-style: none;
  margin-left: -1.5em;
}

/* Strong and emphasis */
.preview-content :deep(strong) {
  font-weight: 600;
}

.preview-content :deep(em) {
  font-style: italic;
}

.preview-content :deep(del) {
  text-decoration: line-through;
  color: var(--color-text-secondary);
}
</style>
