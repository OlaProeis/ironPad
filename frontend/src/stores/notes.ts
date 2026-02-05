import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Note, NoteSummary } from '../types'
import { notesApi } from '../api/client'

export const useNotesStore = defineStore('notes', () => {
  // State
  const notes = ref<NoteSummary[]>([])
  const currentNote = ref<Note | null>(null)
  const loading = ref(false)
  const loadingNote = ref(false)
  const error = ref<string | null>(null)
  const saveStatus = ref<'idle' | 'saving' | 'saved' | 'error'>('idle')

  // Getters
  const sortedNotes = computed(() => 
    [...notes.value].sort((a, b) => {
      const dateA = a.updated ? new Date(a.updated).getTime() : 0
      const dateB = b.updated ? new Date(b.updated).getTime() : 0
      return dateB - dateA
    })
  )

  const getNoteById = computed(() => (id: string) => 
    notes.value.find(n => n.id === id)
  )

  // Actions
  async function loadNotes() {
    try {
      loading.value = true
      error.value = null
      notes.value = await notesApi.list()
    } catch (err) {
      error.value = `Failed to load notes: ${err}`
    } finally {
      loading.value = false
    }
  }

  async function loadNote(id: string) {
    try {
      loadingNote.value = true
      error.value = null
      currentNote.value = await notesApi.get(id)
      saveStatus.value = 'idle'
    } catch (err) {
      error.value = `Failed to load note: ${err}`
      currentNote.value = null
    } finally {
      loadingNote.value = false
    }
  }

  async function createNote() {
    try {
      error.value = null
      const newNote = await notesApi.create()
      await loadNotes()
      return newNote
    } catch (err) {
      error.value = `Failed to create note: ${err}`
      throw err
    }
  }

  async function saveNote(content: string) {
    if (!currentNote.value) return

    try {
      saveStatus.value = 'saving'
      currentNote.value = await notesApi.update(currentNote.value.id, content)
      saveStatus.value = 'saved'
      await loadNotes() // Refresh list to update timestamps
      setTimeout(() => {
        if (saveStatus.value === 'saved') saveStatus.value = 'idle'
      }, 2000)
    } catch (err) {
      saveStatus.value = 'error'
      error.value = `Failed to save note: ${err}`
      throw err
    }
  }

  async function deleteNote(id?: string) {
    const noteId = id || currentNote.value?.id
    if (!noteId) return

    try {
      error.value = null
      await notesApi.delete(noteId)
      if (currentNote.value?.id === noteId) {
        currentNote.value = null
      }
      await loadNotes()
    } catch (err) {
      error.value = `Failed to archive note: ${err}`
      throw err
    }
  }

  function clearCurrentNote() {
    currentNote.value = null
    saveStatus.value = 'idle'
  }

  function clearError() {
    error.value = null
  }

  return {
    // State
    notes,
    currentNote,
    loading,
    loadingNote,
    error,
    saveStatus,
    // Getters
    sortedNotes,
    getNoteById,
    // Actions
    loadNotes,
    loadNote,
    createNote,
    saveNote,
    deleteNote,
    clearCurrentNote,
    clearError
  }
})
