# Tools Used

Every tool in the Ironpad development workflow and how it fits in.

## IDE

### Cursor IDE

The primary development environment. Cursor is a fork of VS Code with built-in AI integration.

- **Why:** Direct integration with Claude models, inline code editing, multi-file context
- **How used:** All coding, file editing, terminal commands, and AI conversations happen in Cursor
- **Alternative:** VS Code + Copilot, Windsurf, or any editor with AI integration

## AI Models

### Claude Opus 4.5 (Anthropic)

Used for the majority of Ironpad's development (Phases 1-3 of the implementation).

- **Context window:** 200K tokens
- **Strengths:** Excellent at Rust code, understands Axum patterns well, good at architecture
- **Limitation:** Can only hold ~5 files at once; required task splitting and handover documents
- **How used:** Feature implementation, debugging, code review

### Claude Opus 4.6 (Anthropic)

Used for later phases and the full codebase audit.

- **Context window:** 1M tokens (5x increase)
- **Strengths:** Can hold the entire codebase at once; finds cross-file bugs; handles complex refactors in one session
- **How used:** Codebase audit (found 16 issues), cross-cutting refactors, feature implementation without task splitting

### Perplexity AI

Used for research before coding.

- **Why:** Has internet access, provides current information with citations
- **How used:** Checking library versions, finding known issues with crates, researching approaches
- **Example:** Verified that `serde_yaml` was deprecated before we chose to use it (accepted risk for v1)

### Google Gemini

Used as a second opinion on architecture.

- **Why:** Different training data and perspective from Claude
- **How used:** PRD review, architecture review, catching blind spots
- **Example:** Flagged the need for file locking between Task View and Editor (race condition that Claude's initial design missed)

## MCP Tools

### Task Master

A Model Context Protocol (MCP) tool for structured task management.

- **What it does:** Parses a PRD and generates ordered task lists with dependencies
- **How used:** Fed the PRD to Task Master to generate the implementation plan; tasks tracked through completion
- **Why it matters:** Turns a document into actionable, sequenced work items

### Context7

A Model Context Protocol (MCP) tool for pulling current library documentation.

- **What it does:** Fetches up-to-date documentation for any library and loads it into the AI's context
- **How used:** Pulled current Axum 0.8 docs, Vue 3 Composition API docs, Milkdown editor API docs
- **Why it matters:** Eliminates bugs caused by the AI using outdated API knowledge

## Build Tools

### Rust / Cargo

- Backend language and build system
- `cargo check` for fast compilation checks
- `cargo build --release` for production binaries
- Strict compiler catches entire categories of bugs before runtime

### Node.js / Vite

- Frontend build tooling
- `npm run dev` for development with hot reload
- `npm run build` for production static files
- Vue SFC compilation via `@vitejs/plugin-vue`

### Git

- Version control for all data files
- `git2` crate for programmatic access from Rust
- Automatic 60-second commit batching
- Full diff viewer in the UI

## The Tool Stack in Practice

```
Idea
  |
  v
[Perplexity] -- Research libraries, check feasibility
[Gemini]     -- Second opinion on approach
  |
  v
[Claude]     -- Draft PRD
[Gemini]     -- Review PRD
  |
  v
[Task Master] -- Generate ordered task list
  |
  v
[Cursor + Claude] -- Implement each task
[Context7]        -- Current docs when needed
  |
  v
[Manual Testing]  -- Verify in browser
[cargo check]     -- Compiler verification
```

No single tool does everything. The value is in how they compose.
