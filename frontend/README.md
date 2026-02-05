# Ironpad Frontend

Vue 3 single-page application for Ironpad.

## Quick Start

```bash
npm install
npm run dev
```

Open http://localhost:5173 (requires backend running on port 3000).

## Tech Stack

- **Vue 3** with Composition API and `<script setup>`
- **TypeScript** for type safety
- **Vite** for fast development and builds
- **Pinia** for state management
- **Vue Router** for navigation
- **Milkdown** for WYSIWYG Markdown editing

## Project Structure

```
src/
├── api/              # API client for backend communication
│   └── client.ts
├── components/       # Reusable Vue components
│   ├── MilkdownEditor.vue      # Editor wrapper
│   ├── MilkdownEditorCore.vue  # Core editor logic
│   ├── Sidebar.vue             # Navigation sidebar
│   ├── GitPanel.vue            # Git operations panel
│   └── ...
├── composables/      # Vue composables
│   └── useWebSocket.ts
├── router/           # Vue Router configuration
│   └── index.ts
├── stores/           # Pinia stores
│   ├── notes.ts      # Notes state
│   ├── projects.ts   # Projects state
│   ├── tasks.ts      # Tasks state
│   ├── git.ts        # Git state
│   ├── theme.ts      # Theme state
│   ├── ui.ts         # UI state
│   ├── websocket.ts  # WebSocket state
│   └── workspace.ts  # Workspace state
├── types/            # TypeScript type definitions
│   └── index.ts
├── views/            # Route views
│   ├── DashboardView.vue        # Home page with project cards + task summaries
│   ├── ProjectView.vue          # Project overview with editor
│   ├── ProjectNotesView.vue     # Project notes split view
│   ├── ProjectsView.vue         # Projects management list
│   ├── TasksView.vue            # Task split view (list + detail)
│   ├── CalendarView.vue         # Month grid calendar
│   └── DailyView.vue            # Daily notes
├── App.vue           # Root component
├── main.ts           # Entry point
└── style.css         # Global styles
```

## Key Components

### Milkdown Editor

The editor consists of two components:

- **MilkdownEditor.vue** — Wrapper component that accepts a `:key` prop for recreation
- **MilkdownEditorCore.vue** — Core editor using the `@milkdown/vue` integration

**Critical Pattern**: When switching between notes/tasks, content MUST be set BEFORE updating the editor key:

```javascript
// CORRECT order:
editorContent.value = newContent    // Set content first
editorKey.value = noteId            // Then trigger editor recreation

// WRONG order (causes stale content):
editorKey.value = noteId            // Editor recreates with wrong content
editorContent.value = newContent    // Too late!
```

### Task System Features

The task view (`TasksView.vue`) includes:

- **Tag system** — tags stored in YAML frontmatter, filterable via tag bar, autocomplete from project tags
- **Subtasks** — tasks with `parent_id` grouped under parents, inline creation, count badges
- **Recurring tasks** — daily/weekly/monthly/yearly, auto-creates next on completion
- **Due date picker** — inline date input, clearable, color-coded urgency display
- **Active/Backlog toggle** — move tasks between states

### Dashboard (`DashboardView.vue`)

Cross-project home page showing all projects as cards with:
- Active task count, backlog count, overdue count
- Top 5 active tasks per project with tags and due dates
- Click-through to project or individual task

### Calendar (`CalendarView.vue`)

Month grid calendar showing:
- Tasks plotted by `due_date` (only tasks with dates appear)
- Daily notes shown as blue dots
- Color-coded urgency (overdue=red, today=red, soon=yellow)
- Navigation: prev/next month, Today button

### State Management

Each domain has its own Pinia store:

- `notesStore` — Standalone notes CRUD
- `projectsStore` — Projects list and details
- `tasksStore` — Project tasks with active/backlog sections, tag filtering, subtask helpers
- `gitStore` — Git status, commits, push/pull
- `themeStore` — Dark/light mode
- `uiStore` — Search panel, modals
- `websocketStore` — Real-time connection state
- `workspaceStore` — Active project tracking

### Auto-save Behavior

Views implement smart auto-save that:
1. Tracks the "last saved content" when a note/task loads
2. Only saves when content differs from last saved
3. Uses 1-second debounce to batch rapid edits
4. Prevents unnecessary saves when just opening items

## Commands

```bash
npm run dev       # Start dev server (hot reload)
npm run build     # Production build to dist/
npm run preview   # Preview production build
npm run lint      # Run ESLint
```

## Environment

The frontend expects the backend API at `http://localhost:3000`. This is configured in `src/api/client.ts`.

For production, build the frontend and serve from any static host, configuring the API URL as needed.
