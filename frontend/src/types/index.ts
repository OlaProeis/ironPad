// Types for Ironpad

export interface NoteSummary {
  id: string
  title: string
  path: string
  note_type: string
  updated?: string
}

export interface Note {
  id: string
  path: string
  note_type: string
  frontmatter: Record<string, unknown>
  content: string
}

export interface Project {
  id: string
  name: string
  path: string
  created: string
}

export interface ProjectWithContent extends Project {
  content: string
}

export interface ProjectNote {
  id: string
  title: string
  path: string
  project_id: string
  created: string
  updated: string
}

export interface ProjectNoteWithContent extends ProjectNote {
  content: string
}

export interface Task {
  id: string
  title: string
  completed: boolean
  section: string
  priority?: string
  due_date?: string
  is_active: boolean
  tags: string[]
  parent_id?: string
  recurrence?: string
  recurrence_interval?: number
  project_id: string
  path: string
  created: string
  updated: string
}

export interface TaskWithContent extends Task {
  content: string
}

export interface SearchResult {
  path: string
  title: string
  matches: { line_number: number; line_content: string }[]
}

export interface GitStatus {
  is_repo: boolean
  branch?: string
  has_changes: boolean
  files: { path: string; status: string }[]
  last_commit?: { id: string; message: string; timestamp: string }
  conflicts?: string[]
}

export interface CommitInfo {
  id: string
  message: string
  timestamp: string
}

export interface CommitDetail {
  id: string
  short_id: string
  message: string
  author: string
  timestamp: string
  files_changed: number
}

export interface DiffLine {
  origin: string
  content: string
}

export interface DiffHunk {
  header: string
  lines: DiffLine[]
}

export interface FileDiff {
  path: string
  status: string
  additions: number
  deletions: number
  hunks: DiffHunk[]
}

export interface DiffStats {
  files_changed: number
  insertions: number
  deletions: number
}

export interface DiffInfo {
  files: FileDiff[]
  stats: DiffStats
}

export interface RemoteInfo {
  name: string
  url: string
  has_upstream: boolean
  ahead: number
  behind: number
}

export interface DailyNote {
  id: string
  date: string
  path: string
  content: string
  frontmatter: Record<string, unknown>
}

export interface FileLock {
  path: string
  client_id: string
  lock_type: 'editor' | 'task_view'
}

// WebSocket message types
export type WsMessageType = 
  | 'Connected'
  | 'FileCreated'
  | 'FileModified'
  | 'FileDeleted'
  | 'FileRenamed'
  | 'FileLocked'
  | 'FileUnlocked'
  | 'GitConflict'
  | 'Ping'

export interface WsMessage {
  type: WsMessageType
  payload?: unknown
}

export interface WsConnectedPayload {
  client_id: string
}

export interface WsFilePayload {
  path: string
}

export interface WsFileLockPayload {
  path: string
  client_id: string
  lock_type: 'editor' | 'task_view'
}
