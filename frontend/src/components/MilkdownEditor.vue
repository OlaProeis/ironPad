<script setup lang="ts">
import { ref, computed } from 'vue'
import { MilkdownProvider } from '@milkdown/vue'
import { Crepe } from '@milkdown/crepe'
import { useThemeStore } from '../stores'
import MilkdownEditorCore from './MilkdownEditorCore.vue'

const props = defineProps<{
  modelValue: string
  readonly?: boolean
  placeholder?: string
  projectId?: string
  editorKey?: string | number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const themeStore = useThemeStore()
const uploading = ref(false)
const editorInstance = ref<Crepe | null>(null)
const remountCounter = ref(0)

const isDarkMode = computed(() => themeStore.getEffectiveTheme() === 'dark')
const effectiveKey = computed(() => `${props.editorKey}-${remountCounter.value}`)

function handleContentUpdate(value: string) {
  emit('update:modelValue', value)
}

function handleEditorReady(crepe: Crepe) {
  editorInstance.value = crepe
}

function handleForceRemount() {
  remountCounter.value++
}
</script>

<template>
  <div 
    class="milkdown-editor-wrapper" 
    :class="{ 
      readonly, 
      uploading,
      'dark-mode': isDarkMode
    }"
  >
    <!-- Upload indicator -->
    <div v-if="uploading" class="upload-indicator">
      Uploading image...
    </div>
    
    <div :key="effectiveKey" class="milkdown-container" :class="{ 'is-readonly': readonly }">
      <MilkdownProvider>
        <MilkdownEditorCore
          :model-value="modelValue"
          :readonly="readonly"
          :project-id="projectId"
          @update:model-value="handleContentUpdate"
          @editor-ready="handleEditorReady"
          @force-remount="handleForceRemount"
        />
      </MilkdownProvider>
    </div>
  </div>
</template>

<style scoped>
.milkdown-editor-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.milkdown-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: auto;
}

.milkdown-container.is-readonly {
  pointer-events: none;
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

.milkdown-editor-wrapper.uploading .milkdown-container {
  opacity: 0.5;
  pointer-events: none;
}
</style>
