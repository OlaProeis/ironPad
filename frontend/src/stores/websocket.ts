import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { FileLock } from '../types'

export const useWebSocketStore = defineStore('websocket', () => {
  // State
  const connected = ref(false)
  const clientId = ref<string | null>(null)
  const fileLocks = ref<Map<string, FileLock>>(new Map())
  const gitConflicts = ref<string[]>([])

  // Actions
  function setConnected(value: boolean) {
    connected.value = value
  }

  function setClientId(id: string | null) {
    clientId.value = id
  }

  function addFileLock(lock: FileLock) {
    fileLocks.value.set(lock.path, lock)
  }

  function removeFileLock(path: string) {
    fileLocks.value.delete(path)
  }

  function isFileLocked(path: string): FileLock | undefined {
    return fileLocks.value.get(path)
  }

  function isFileLockedByOther(path: string): boolean {
    const lock = fileLocks.value.get(path)
    return lock !== undefined && lock.client_id !== clientId.value
  }

  function setGitConflicts(files: string[]) {
    gitConflicts.value = files
  }

  function clearGitConflicts() {
    gitConflicts.value = []
  }

  function clearAllLocks() {
    fileLocks.value.clear()
  }

  return {
    // State
    connected,
    clientId,
    fileLocks,
    gitConflicts,
    // Actions
    setConnected,
    setClientId,
    addFileLock,
    removeFileLock,
    isFileLocked,
    isFileLockedByOther,
    setGitConflicts,
    clearGitConflicts,
    clearAllLocks
  }
})
