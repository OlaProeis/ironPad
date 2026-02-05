<script setup lang="ts">
import { ref } from 'vue'

const emit = defineEmits<{
  'format': [type: string, extra?: string]
  'insert-image': [file: File]
  'insert-link': []
}>()

const fileInput = ref<HTMLInputElement | null>(null)

// Formatting actions
function bold() { emit('format', 'bold') }
function italic() { emit('format', 'italic') }
function strikethrough() { emit('format', 'strikethrough') }
function heading(level: number) { emit('format', 'heading', String(level)) }
function link() { emit('insert-link') }
function code() { emit('format', 'code') }
function codeBlock() { emit('format', 'codeblock') }
function quote() { emit('format', 'quote') }
function bulletList() { emit('format', 'bullet') }
function numberedList() { emit('format', 'numbered') }
function taskList() { emit('format', 'task') }
function horizontalRule() { emit('format', 'hr') }

// Image handling
function triggerImageUpload() {
  fileInput.value?.click()
}

function handleFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (file) {
    emit('insert-image', file)
    // Reset input so same file can be selected again
    input.value = ''
  }
}

// Heading dropdown
const showHeadingDropdown = ref(false)

function toggleHeadingDropdown() {
  showHeadingDropdown.value = !showHeadingDropdown.value
}

function selectHeading(level: number) {
  heading(level)
  showHeadingDropdown.value = false
}

// Close dropdown when clicking outside
function closeDropdowns() {
  showHeadingDropdown.value = false
}
</script>

<template>
  <div class="editor-toolbar" @click.stop>
    <!-- Hidden file input for image upload -->
    <input
      ref="fileInput"
      type="file"
      accept="image/*"
      style="display: none"
      @change="handleFileSelect"
    />

    <!-- Text formatting group -->
    <div class="toolbar-group">
      <button 
        class="toolbar-btn" 
        @click="bold" 
        title="Bold (Ctrl+B)"
      >
        <span class="icon">B</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="italic" 
        title="Italic (Ctrl+I)"
      >
        <span class="icon italic">I</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="strikethrough" 
        title="Strikethrough"
      >
        <span class="icon strikethrough">S</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="code" 
        title="Inline code (Ctrl+`)"
      >
        <span class="icon mono">&lt;/&gt;</span>
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Heading dropdown -->
    <div class="toolbar-group">
      <div class="dropdown-container">
        <button 
          class="toolbar-btn" 
          @click="toggleHeadingDropdown"
          title="Heading"
        >
          <span class="icon">H</span>
          <span class="dropdown-arrow">▾</span>
        </button>
        <div v-if="showHeadingDropdown" class="dropdown-menu" @click.stop>
          <button @click="selectHeading(1)">Heading 1</button>
          <button @click="selectHeading(2)">Heading 2</button>
          <button @click="selectHeading(3)">Heading 3</button>
          <button @click="selectHeading(4)">Heading 4</button>
        </div>
      </div>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Insert group -->
    <div class="toolbar-group">
      <button 
        class="toolbar-btn" 
        @click="link" 
        title="Insert link"
      >
        <span class="icon">🔗</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="triggerImageUpload" 
        title="Insert image"
      >
        <span class="icon">🖼️</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="codeBlock" 
        title="Code block"
      >
        <span class="icon mono">{}</span>
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- List group -->
    <div class="toolbar-group">
      <button 
        class="toolbar-btn" 
        @click="bulletList" 
        title="Bullet list"
      >
        <span class="icon">•</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="numberedList" 
        title="Numbered list"
      >
        <span class="icon">1.</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="taskList" 
        title="Task list"
      >
        <span class="icon">☐</span>
      </button>
    </div>

    <div class="toolbar-divider"></div>

    <!-- Block group -->
    <div class="toolbar-group">
      <button 
        class="toolbar-btn" 
        @click="quote" 
        title="Quote"
      >
        <span class="icon">"</span>
      </button>
      <button 
        class="toolbar-btn" 
        @click="horizontalRule" 
        title="Horizontal rule"
      >
        <span class="icon">—</span>
      </button>
    </div>

    <!-- Click outside to close dropdowns -->
    <div 
      v-if="showHeadingDropdown" 
      class="dropdown-overlay" 
      @click="closeDropdowns"
    ></div>
  </div>
</template>

<style scoped>
.editor-toolbar {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 12px;
  background: var(--color-bg-secondary);
  border-bottom: 1px solid var(--color-border);
  flex-wrap: wrap;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  padding: 0;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text);
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}

.toolbar-btn:hover {
  background: var(--color-bg-hover);
}

.toolbar-btn:active {
  background: var(--color-border);
}

.toolbar-btn .icon {
  font-size: 14px;
  font-weight: 600;
}

.toolbar-btn .icon.italic {
  font-style: italic;
}

.toolbar-btn .icon.strikethrough {
  text-decoration: line-through;
}

.toolbar-btn .icon.mono {
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 500;
}

.toolbar-btn .dropdown-arrow {
  font-size: 8px;
  margin-left: 2px;
  color: var(--color-text-secondary);
}

.toolbar-divider {
  width: 1px;
  height: 20px;
  background: var(--color-border);
  margin: 0 6px;
}

/* Dropdown */
.dropdown-container {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 4px;
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  min-width: 120px;
  overflow: hidden;
}

.dropdown-menu button {
  display: block;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  color: var(--color-text);
  text-align: left;
  cursor: pointer;
  font-size: 13px;
}

.dropdown-menu button:hover {
  background: var(--color-bg-hover);
}

.dropdown-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 99;
}
</style>
