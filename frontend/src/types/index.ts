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
  github_repo?: string
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

export interface Comment {
  date: string
  text: string
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
  estimated_minutes?: number
  project_id: string
  path: string
  created: string
  updated: string
  last_comment?: string
}

export interface TaskWithContent extends Task {
  content: string
  comments: Comment[]
}

// Helper type for time estimates
export type TimeEstimate = number | undefined // minutes

export interface SearchResult {
  path: string
  title: string
  matches: { line_number: number; line_content: string }[]
}

export type PromptKind = 'prompt' | 'link'

export interface PromptSummary {
  id: string
  title: string
  path: string
  scope: 'global' | 'project' | string
  project_id?: string
  folder: string
  tags: string[]
  description?: string
  updated?: string
  kind: PromptKind
  source?: string
}

export interface Prompt extends PromptSummary {
  template: boolean
  frontmatter: Record<string, unknown>
  content: string
  read_only: boolean
  source_missing: boolean
  resolved_path?: string
  root_required?: boolean
  source_error?: string
  supports_rendered?: boolean
}

export interface BrowseEntry {
  name: string
  path: string
  is_dir: boolean
  size: number
  linkable: boolean
}

export interface BrowseResponse {
  path: string
  parent: string | null
  entries: BrowseEntry[]
}

export interface OpenLibraryResponse {
  ok: boolean
  resolved_path: string
}

export interface LibraryPickResponse {
  paths: string[]
}

export interface PromptFolder {
  path: string
  count: number
}

export interface PromptSearchResult {
  prompt: PromptSummary
  score: number
  matched_terms: string[]
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

export interface SetRemoteResponse {
  success: boolean
  message: string
  url: string
  replaced: boolean
  fetch_error: string | null
}

export interface DailyNote {
  id: string
  date: string
  path: string
  content: string
  frontmatter: Record<string, unknown>
}

export interface Backlink {
  source_id: string
  source_title: string
  source_path: string
  context: string
  line_number: number
}

export interface ForwardLink {
  target_id: string
  target_title?: string
  context: string
  line_number: number
}

export interface NoteLinksResponse {
  note_id: string
  backlinks: Backlink[]
  forward_links: ForwardLink[]
}

export interface NoteTitleEntry {
  id: string
  title: string
  path?: string
}

export interface ProjectNotesTitlesResponse {
  notes: NoteTitleEntry[]
}

export interface ProjectNotesSearchResponse {
  query: string
  results: NoteTitleEntry[]
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

export interface GithubLabel {
  name: string
  color?: string
}

export interface GithubIssue {
  number: number
  title: string
  html_url: string
  updated_at: string
  author_login: string
  labels: GithubLabel[]
  comments_count: number
  milestone_due?: string
  assignee_logins: string[]
}

export interface GithubPull {
  number: number
  title: string
  html_url: string
  updated_at: string
  author_login: string
  labels: GithubLabel[]
  comments_count: number
  milestone_due?: string
  draft: boolean
  requested_reviewer_logins: string[]
  head_sha: string
}

export interface GithubSnapshotResponse {
  repo?: string
  login?: string
  refreshed_at?: string
  stale: boolean
  issues: GithubIssue[]
  pulls: GithubPull[]
  issues_truncated: boolean
  pulls_truncated: boolean
  error?: string
}

export interface GithubComment {
  author_login: string
  created_at: string
  body: string
}

export interface GithubItemDetail {
  number: number
  title: string
  html_url: string
  body?: string
  body_too_large: boolean
  comments: GithubComment[]
  more_comments: boolean
  commit_status?: string
  error?: string
}

export interface TrackResult {
  task_id: string
  created: boolean
}

export interface GithubSummaryEntry {
  project_id: string
  repo: string
  assigned_issues: number
  review_pulls: number
  refreshed_at?: string
}
