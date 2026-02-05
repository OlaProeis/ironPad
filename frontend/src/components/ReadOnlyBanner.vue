<script setup lang="ts">
defineProps<{
  message?: string
}>()
</script>

<template>
  <div class="read-only-banner">
    <span class="icon">🔒</span>
    <span class="message">{{ message || 'This file is being edited elsewhere. Read-only mode.' }}</span>
  </div>
</template>

<style scoped>
.read-only-banner {
  padding: 8px 16px;
  background: var(--color-primary);
  color: white;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.icon {
  font-size: 14px;
}

.message {
  flex: 1;
}
</style>
