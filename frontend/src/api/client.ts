// API client for Ironpad backend

import type { 
  Note, 
  NoteSummary, 
  Project,
  ProjectWithContent,
  ProjectNote,
  ProjectNoteWithContent,
  Task,
  TaskWithContent,
  SearchResult, 
  GitStatus, 
  CommitInfo,
  CommitDetail,
  DiffInfo,
  RemoteInfo,
  DailyNote 
} from '../types'

const API_BASE = '/api'

async function request<T>(url: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${url}`, options)
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || `HTTP ${res.status}`)
  }
  // Handle empty responses
  const contentType = res.headers.get('content-type')
  if (contentType?.includes('application/json')) {
    return res.json()
  }
  return undefined as T
}

// Notes API
export const notesApi = {
  list: () => request<NoteSummary[]>('/notes'),
  
  get: (id: string) => request<Note>(`/notes/${encodeURIComponent(id)}`),
  
  create: () => request<Note>('/notes', { method: 'POST' }),
  
  update: (id: string, content: string) => 
    request<Note>(`/notes/${encodeURIComponent(id)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'text/plain' },
      body: content
    }),
  
  delete: (id: string) => 
    request<void>(`/notes/${encodeURIComponent(id)}`, { method: 'DELETE' })
}

// Projects API
export const projectsApi = {
  list: () => request<Project[]>('/projects'),
  
  get: (id: string) => request<Project>(`/projects/${encodeURIComponent(id)}`),
  
  getContent: (id: string) => 
    request<ProjectWithContent>(`/projects/${encodeURIComponent(id)}/content`),
  
  updateContent: (id: string, content: string) =>
    request<ProjectWithContent>(`/projects/${encodeURIComponent(id)}/content`, {
      method: 'PUT',
      headers: { 'Content-Type': 'text/plain' },
      body: content
    }),
  
  create: (name: string) => 
    request<Project>('/projects', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ name })
    }),
  
  // Project Notes
  listNotes: (projectId: string) =>
    request<ProjectNote[]>(`/projects/${encodeURIComponent(projectId)}/notes`),
  
  getNote: (projectId: string, noteId: string) =>
    request<ProjectNoteWithContent>(`/projects/${encodeURIComponent(projectId)}/notes/${encodeURIComponent(noteId)}`),
  
  createNote: (projectId: string, title?: string) =>
    request<ProjectNoteWithContent>(`/projects/${encodeURIComponent(projectId)}/notes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title })
    }),
  
  updateNote: (projectId: string, noteId: string, content: string) =>
    request<ProjectNoteWithContent>(`/projects/${encodeURIComponent(projectId)}/notes/${encodeURIComponent(noteId)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'text/plain' },
      body: content
    }),
  
  deleteNote: (projectId: string, noteId: string) =>
    request<void>(`/projects/${encodeURIComponent(projectId)}/notes/${encodeURIComponent(noteId)}`, {
      method: 'DELETE'
    })
}

// Tasks API (file-based tasks)
export const tasksApi = {
  // List all tasks across all projects
  listAll: () => request<Task[]>('/tasks'),
  
  // List tasks for a specific project
  list: (projectId: string) => 
    request<Task[]>(`/projects/${encodeURIComponent(projectId)}/tasks`),
  
  // Get a single task with content
  get: (projectId: string, taskId: string) => 
    request<TaskWithContent>(`/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}`),
  
  // Create a new task
  create: (projectId: string, title: string, section?: string, parentId?: string) =>
    request<TaskWithContent>(`/projects/${encodeURIComponent(projectId)}/tasks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ title, section, parent_id: parentId || undefined })
    }),
  
  // Update task content (markdown body)
  updateContent: (projectId: string, taskId: string, content: string) =>
    request<TaskWithContent>(`/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'text/plain' },
      body: content
    }),
  
  // Toggle task completion
  toggle: (projectId: string, taskId: string) =>
    request<Task>(`/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}/toggle`, {
      method: 'PUT'
    }),
  
  // Update task metadata
  updateMeta: (projectId: string, taskId: string, meta: { title?: string; section?: string; priority?: string; due_date?: string; is_active?: boolean; tags?: string[]; recurrence?: string; recurrence_interval?: number }) =>
    request<Task>(`/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}/meta`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(meta)
    }),
  
  // Delete (archive) a task
  delete: (projectId: string, taskId: string) =>
    request<void>(`/projects/${encodeURIComponent(projectId)}/tasks/${encodeURIComponent(taskId)}`, { 
      method: 'DELETE' 
    })
}

// Search API
export const searchApi = {
  search: (query: string) => 
    request<SearchResult[]>(`/search?q=${encodeURIComponent(query)}`)
}

// Git API
export const gitApi = {
  status: () => request<GitStatus>('/git/status'),
  
  commit: (message?: string) => 
    request<CommitInfo>('/git/commit', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ message })
    }),
  
  push: () => request<{ success: boolean; message: string }>('/git/push', { method: 'POST' }),
  
  conflicts: () => request<string[]>('/git/conflicts'),
  
  // Commit history
  log: (limit?: number) => 
    request<CommitDetail[]>(`/git/log${limit ? `?limit=${limit}` : ''}`),
  
  // Working directory diff (uncommitted changes)
  diff: () => request<DiffInfo>('/git/diff'),
  
  // Diff for a specific commit
  commitDiff: (commitId: string) => 
    request<DiffInfo>(`/git/diff/${encodeURIComponent(commitId)}`),
  
  // Remote repository info
  remote: () => request<RemoteInfo | null>('/git/remote'),
  
  // Fetch from remote
  fetch: () => request<{ success: boolean; message: string }>('/git/fetch', { method: 'POST' })
}

// Daily Notes API
export const dailyApi = {
  list: () => request<DailyNote[]>('/daily'),
  
  today: () => request<DailyNote>('/daily/today'),
  
  get: (date: string) => request<DailyNote>(`/daily/${date}`),
  
  create: (date: string, content?: string) => 
    request<DailyNote>(`/daily/${date}`, { 
      method: 'POST',
      headers: content ? { 'Content-Type': 'application/json' } : undefined,
      body: content ? JSON.stringify({ content }) : undefined
    }),
  
  update: (date: string, content: string) =>
    request<DailyNote>(`/daily/${date}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'text/plain' },
      body: content
    })
}

// Assets API
export const assetsApi = {
  upload: async (file: File, projectId?: string): Promise<{ url: string; filename: string }> => {
    const formData = new FormData()
    formData.append('file', file)
    
    const params = projectId ? `?project=${encodeURIComponent(projectId)}` : ''
    const res = await fetch(`${API_BASE}/assets/upload${params}`, {
      method: 'POST',
      body: formData
    })
    
    if (!res.ok) {
      const text = await res.text()
      throw new Error(text || `HTTP ${res.status}`)
    }
    
    return res.json()
  },
  
  getUrl: (project: string, filename: string) => 
    `${API_BASE}/assets/${encodeURIComponent(project)}/${encodeURIComponent(filename)}`
}
