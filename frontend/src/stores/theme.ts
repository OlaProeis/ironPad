import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ThemeMode = 'dark' | 'light' | 'system'

const STORAGE_KEY = 'ironpad-theme'

export const useThemeStore = defineStore('theme', () => {
  // Default to dark mode
  const mode = ref<ThemeMode>('dark')
  
  // Load saved preference
  function loadSavedTheme() {
    const saved = localStorage.getItem(STORAGE_KEY)
    if (saved && ['dark', 'light', 'system'].includes(saved)) {
      mode.value = saved as ThemeMode
    }
  }
  
  // Get the effective theme (resolves 'system' to actual theme)
  function getEffectiveTheme(): 'dark' | 'light' {
    if (mode.value === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    return mode.value
  }
  
  // Apply theme to document
  function applyTheme() {
    const effectiveTheme = getEffectiveTheme()
    document.documentElement.setAttribute('data-theme', effectiveTheme)
    
    // Also set class for easier CSS targeting
    document.documentElement.classList.remove('theme-dark', 'theme-light')
    document.documentElement.classList.add(`theme-${effectiveTheme}`)
  }
  
  // Set theme mode
  function setTheme(newMode: ThemeMode) {
    mode.value = newMode
    localStorage.setItem(STORAGE_KEY, newMode)
    applyTheme()
  }
  
  // Toggle between dark and light
  function toggleTheme() {
    const current = getEffectiveTheme()
    setTheme(current === 'dark' ? 'light' : 'dark')
  }
  
  // Initialize
  function init() {
    loadSavedTheme()
    
    // Ensure data-theme attribute is set (even if same as default)
    applyTheme()
    
    // Listen for system theme changes when in system mode
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
      if (mode.value === 'system') {
        applyTheme()
      }
    })
    
    // Theme is now initialized
  }
  
  return {
    mode,
    getEffectiveTheme,
    setTheme,
    toggleTheme,
    applyTheme,
    init
  }
})
