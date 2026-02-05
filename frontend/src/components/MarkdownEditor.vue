<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, shallowRef } from 'vue'
import { EditorState } from '@codemirror/state'
import { EditorView, keymap, lineNumbers, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view'
import { defaultKeymap, history, historyKeymap } from '@codemirror/commands'
import { markdown, markdownLanguage } from '@codemirror/lang-markdown'
import { syntaxHighlighting, defaultHighlightStyle, bracketMatching } from '@codemirror/language'
import { oneDark } from '@codemirror/theme-one-dark'
import { useThemeStore } from '../stores'
import { assetsApi } from '../api/client'
import EditorToolbar from './EditorToolbar.vue'

const props = defineProps<{
  modelValue: string
  readonly?: boolean
  placeholder?: string
  projectId?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const themeStore = useThemeStore()
const editorContainer = ref<HTMLDivElement | null>(null)
const editorView = shallowRef<EditorView | null>(null)
const uploading = ref(false)

// Check if dark mode is active
function isDarkMode(): boolean {
  return themeStore.getEffectiveTheme() === 'dark'
}

// Create custom theme for light mode
const lightTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--color-bg)',
    color: 'var(--color-text)'
  },
  '.cm-content': {
    fontFamily: "'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace",
    fontSize: '14px',
    lineHeight: '1.6',
    padding: '16px 0'
  },
  '.cm-gutters': {
    backgroundColor: 'var(--color-bg-secondary)',
    color: 'var(--color-text-secondary)',
    border: 'none',
    borderRight: '1px solid var(--color-border)'
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--color-border)'
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(0, 0, 0, 0.03)'
  },
  '&.cm-focused .cm-cursor': {
    borderLeftColor: 'var(--color-primary)'
  },
  '&.cm-focused .cm-selectionBackground, ::selection': {
    backgroundColor: 'rgba(3, 102, 214, 0.2)'
  },
  '.cm-scroller': {
    overflow: 'auto'
  }
})

// Create dark theme extension
const darkTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--color-bg)',
    color: 'var(--color-text)'
  },
  '.cm-content': {
    fontFamily: "'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace",
    fontSize: '14px',
    lineHeight: '1.6',
    padding: '16px 0'
  },
  '.cm-gutters': {
    backgroundColor: 'var(--color-bg-secondary)',
    color: 'var(--color-text-secondary)',
    border: 'none',
    borderRight: '1px solid var(--color-border)'
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'var(--color-border)'
  },
  '.cm-activeLine': {
    backgroundColor: 'rgba(255, 255, 255, 0.03)'
  },
  '&.cm-focused .cm-cursor': {
    borderLeftColor: 'var(--color-primary)'
  },
  '&.cm-focused .cm-selectionBackground, ::selection': {
    backgroundColor: 'rgba(79, 195, 247, 0.2)'
  },
  '.cm-scroller': {
    overflow: 'auto'
  }
}, { dark: true })

// Markdown formatting keybindings
function toggleBold(view: EditorView): boolean {
  const { from, to } = view.state.selection.main
  const selectedText = view.state.sliceDoc(from, to)
  
  if (selectedText) {
    // Check if already bold
    const isBold = selectedText.startsWith('**') && selectedText.endsWith('**')
    let newText: string
    
    if (isBold) {
      newText = selectedText.slice(2, -2)
    } else {
      newText = `**${selectedText}**`
    }
    
    view.dispatch({
      changes: { from, to, insert: newText }
    })
  }
  return true
}

function toggleItalic(view: EditorView): boolean {
  const { from, to } = view.state.selection.main
  const selectedText = view.state.sliceDoc(from, to)
  
  if (selectedText) {
    // Check if already italic (single asterisk, not bold)
    const isItalic = selectedText.startsWith('*') && selectedText.endsWith('*') && 
                     !selectedText.startsWith('**')
    let newText: string
    
    if (isItalic) {
      newText = selectedText.slice(1, -1)
    } else {
      newText = `*${selectedText}*`
    }
    
    view.dispatch({
      changes: { from, to, insert: newText }
    })
  }
  return true
}

function toggleCode(view: EditorView): boolean {
  const { from, to } = view.state.selection.main
  const selectedText = view.state.sliceDoc(from, to)
  
  if (selectedText) {
    const isCode = selectedText.startsWith('`') && selectedText.endsWith('`')
    let newText: string
    
    if (isCode) {
      newText = selectedText.slice(1, -1)
    } else {
      newText = `\`${selectedText}\``
    }
    
    view.dispatch({
      changes: { from, to, insert: newText }
    })
  }
  return true
}

const markdownKeymap = keymap.of([
  { key: 'Mod-b', run: toggleBold },
  { key: 'Mod-i', run: toggleItalic },
  { key: 'Mod-`', run: toggleCode }
])

// Toolbar format handler
function handleFormat(type: string, extra?: string) {
  const view = editorView.value
  if (!view) return
  
  const { from, to } = view.state.selection.main
  const selectedText = view.state.sliceDoc(from, to)
  const line = view.state.doc.lineAt(from)
  const lineStart = line.from
  const lineText = line.text
  
  let insert = ''
  let newFrom = from
  let newTo = to
  
  switch (type) {
    case 'bold':
      if (selectedText) {
        insert = `**${selectedText}**`
      } else {
        insert = '**bold**'
        newFrom = from + 2
        newTo = from + 6
      }
      break
      
    case 'italic':
      if (selectedText) {
        insert = `*${selectedText}*`
      } else {
        insert = '*italic*'
        newFrom = from + 1
        newTo = from + 7
      }
      break
      
    case 'strikethrough':
      if (selectedText) {
        insert = `~~${selectedText}~~`
      } else {
        insert = '~~strikethrough~~'
        newFrom = from + 2
        newTo = from + 15
      }
      break
      
    case 'code':
      if (selectedText) {
        insert = `\`${selectedText}\``
      } else {
        insert = '`code`'
        newFrom = from + 1
        newTo = from + 5
      }
      break
      
    case 'codeblock':
      if (selectedText) {
        insert = `\n\`\`\`\n${selectedText}\n\`\`\`\n`
      } else {
        insert = '\n```\ncode\n```\n'
        newFrom = from + 5
        newTo = from + 9
      }
      break
      
    case 'heading':
      const level = parseInt(extra || '2')
      const prefix = '#'.repeat(level) + ' '
      // Check if line already has heading
      const headingMatch = lineText.match(/^(#{1,6})\s/)
      if (headingMatch) {
        // Replace existing heading
        view.dispatch({
          changes: { from: lineStart, to: lineStart + headingMatch[0].length, insert: prefix }
        })
        return
      } else {
        // Insert at line start
        view.dispatch({
          changes: { from: lineStart, to: lineStart, insert: prefix }
        })
        return
      }
      
    case 'quote':
      // Add > at start of each selected line
      if (selectedText.includes('\n')) {
        insert = selectedText.split('\n').map(l => `> ${l}`).join('\n')
      } else if (selectedText) {
        insert = `> ${selectedText}`
      } else {
        view.dispatch({
          changes: { from: lineStart, to: lineStart, insert: '> ' }
        })
        return
      }
      break
      
    case 'bullet':
      view.dispatch({
        changes: { from: lineStart, to: lineStart, insert: '- ' }
      })
      return
      
    case 'numbered':
      view.dispatch({
        changes: { from: lineStart, to: lineStart, insert: '1. ' }
      })
      return
      
    case 'task':
      view.dispatch({
        changes: { from: lineStart, to: lineStart, insert: '- [ ] ' }
      })
      return
      
    case 'hr':
      insert = '\n---\n'
      break
      
    default:
      return
  }
  
  view.dispatch({
    changes: { from, to, insert },
    selection: { anchor: newFrom, head: newTo }
  })
  view.focus()
}

// Link insertion with prompt
function handleInsertLink() {
  const view = editorView.value
  if (!view) return
  
  const { from, to } = view.state.selection.main
  const selectedText = view.state.sliceDoc(from, to)
  
  const url = prompt('Enter URL:', 'https://')
  if (!url) return
  
  const linkText = selectedText || 'link text'
  const insert = `[${linkText}](${url})`
  
  view.dispatch({
    changes: { from, to, insert }
  })
  view.focus()
}

// Image upload and insertion
async function handleInsertImage(file: File) {
  const view = editorView.value
  if (!view) return
  
  uploading.value = true
  
  try {
    // Upload via assets API
    const result = await assetsApi.upload(file, props.projectId)
    
    // Insert markdown image
    const { from, to } = view.state.selection.main
    const altText = file.name.replace(/\.[^/.]+$/, '') // filename without extension
    const insert = `![${altText}](${result.url})`
    
    view.dispatch({
      changes: { from, to, insert }
    })
    view.focus()
  } catch (err) {
    console.error('Failed to upload image:', err)
    alert('Failed to upload image: ' + (err instanceof Error ? err.message : 'Unknown error'))
  } finally {
    uploading.value = false
  }
}

function createEditorState(content: string): EditorState {
  const dark = isDarkMode()
  
  const extensions = [
    lineNumbers(),
    highlightActiveLine(),
    highlightActiveLineGutter(),
    history(),
    bracketMatching(),
    markdown({ base: markdownLanguage }),
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    dark ? darkTheme : lightTheme,
    dark ? oneDark : [],
    keymap.of([
      ...defaultKeymap,
      ...historyKeymap
    ]),
    markdownKeymap,
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        emit('update:modelValue', update.state.doc.toString())
      }
    }),
    EditorState.readOnly.of(props.readonly ?? false),
    EditorView.editable.of(!(props.readonly ?? false))
  ].flat()

  return EditorState.create({
    doc: content,
    extensions
  })
}

function initEditor() {
  if (!editorContainer.value) return

  const state = createEditorState(props.modelValue)
  
  editorView.value = new EditorView({
    state,
    parent: editorContainer.value
  })
}

function destroyEditor() {
  if (editorView.value) {
    editorView.value.destroy()
    editorView.value = null
  }
}

// Watch for external content changes
watch(() => props.modelValue, (newValue) => {
  if (!editorView.value) return
  
  const currentValue = editorView.value.state.doc.toString()
  if (newValue !== currentValue) {
    editorView.value.dispatch({
      changes: {
        from: 0,
        to: editorView.value.state.doc.length,
        insert: newValue
      }
    })
  }
})

// Watch for readonly changes
watch(() => props.readonly, () => {
  if (!editorView.value) return
  
  // Recreate editor with new readonly state
  const content = editorView.value.state.doc.toString()
  destroyEditor()
  initEditor()
  
  // Restore content
  if (editorView.value && content) {
    editorView.value.dispatch({
      changes: {
        from: 0,
        to: editorView.value.state.doc.length,
        insert: content
      }
    })
  }
})

// Listen for theme changes
function handleThemeChange() {
  if (!editorView.value) return
  destroyEditor()
  initEditor()
  // Content will be restored via props.modelValue
}

// Watch theme store changes
watch(() => themeStore.getEffectiveTheme(), handleThemeChange)

onMounted(() => {
  initEditor()
})

onUnmounted(() => {
  destroyEditor()
})
</script>

<template>
  <div class="markdown-editor-wrapper" :class="{ readonly, uploading }">
    <!-- Toolbar (shows when not readonly) -->
    <EditorToolbar 
      v-if="!readonly"
      @format="handleFormat"
      @insert-link="handleInsertLink"
      @insert-image="handleInsertImage"
    />
    
    <!-- Upload indicator -->
    <div v-if="uploading" class="upload-indicator">
      Uploading image...
    </div>
    
    <!-- Editor -->
    <div ref="editorContainer" class="markdown-editor"></div>
  </div>
</template>

<style scoped>
.markdown-editor-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.markdown-editor {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.markdown-editor :deep(.cm-editor) {
  flex: 1;
  overflow: hidden;
}

.markdown-editor :deep(.cm-scroller) {
  padding: 0 24px;
}

.markdown-editor-wrapper.readonly .markdown-editor :deep(.cm-editor) {
  opacity: 0.7;
}

.upload-indicator {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border);
  padding: 12px 24px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 10;
  font-size: 13px;
  color: var(--color-text);
}

.markdown-editor-wrapper.uploading .markdown-editor {
  opacity: 0.5;
  pointer-events: none;
}
</style>
