# Ironpad

![Ironpad Banner](docs/graphics/ironpad-banner.png)

**A local-first, file-based project & knowledge management system.**

![Build](https://github.com/OlaProeis/ironPad/actions/workflows/release.yml/badge.svg)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![Version](https://img.shields.io/badge/version-0.1.0-green)

Ironpad stores all your notes, projects, and tasks as plain Markdown files. No cloud services, no vendor lock-in -- your data stays on your machine in a format you can read and edit with any text editor. Every change is automatically versioned with Git.

![Ironpad Screenshot](docs/screenshot.jpg)

> **v0.1.0 -- Early Release.** This is the first public release. It's functional and we use it daily, but expect rough edges. Bug reports and feature requests are welcome via [Issues](https://github.com/OlaProeis/ironPad/issues).

---

## Features

- **File-based storage** -- All data stored as Markdown files with YAML frontmatter
- **Local-first** -- Works fully offline, no internet required
- **Git integration** -- Automatic version control with 60-second commit batching, full diff viewer, push/fetch
- **WYSIWYG editing** -- Milkdown editor with real-time markdown rendering and formatting toolbar
- **Project management** -- Organize tasks and notes by project with due dates, tags, subtasks, and recurrence
- **Calendar view** -- Month grid showing tasks by due date with color-coded urgency
- **Dashboard** -- Cross-project overview with active task summaries
- **Daily notes** -- Quick capture with templates for daily journaling
- **Real-time sync** -- WebSocket-based live updates; edit in VS Code, see changes in the browser instantly
- **External editing** -- Full support for VS Code, Obsidian, Vim, or any text editor
- **Search** -- ripgrep-powered full-text search across all files (Ctrl+K)
- **Dark theme** -- Beautiful dark UI by default with light mode toggle
- **Tiny footprint** -- 5 MB binary, ~20 MB RAM, sub-second startup

## Quick Start

### Option 1: Download Release (Recommended)

1. Download the latest release for your platform from [Releases](https://github.com/OlaProeis/ironPad/releases)
2. Extract and run the executable
3. Your browser opens automatically -- start using Ironpad

Data is stored in a `data/` folder next to the executable. To use a custom location, set the `IRONPAD_DATA_DIR` environment variable.

### Option 2: Build From Source

**Prerequisites:** [Rust](https://rustup.rs/) (1.70+), [Node.js](https://nodejs.org/) (18+), [Git](https://git-scm.com/)

```bash
# Clone the repository
git clone https://github.com/OlaProeis/ironPad.git
cd ironPad

# Start the backend
cd backend
cargo run

# In a new terminal, start the frontend
cd frontend
npm install
npm run dev
```

Open http://localhost:5173 in your browser.

## Tech Stack

![Tech Stack](docs/graphics/tech-stack.png)

| Component | Technology |
|-----------|------------|
| Backend | Rust, Axum 0.8, Tokio |
| Frontend | Vue 3, Vite, TypeScript |
| Editor | Milkdown (ProseMirror-based) |
| State | Pinia |
| Routing | Vue Router |
| Data | Markdown + YAML frontmatter |
| Version Control | Git (via git2) |
| Search | ripgrep |

## Roadmap

![Roadmap](docs/graphics/roadmap.png)

Ironpad is under active development. Here's what's planned:

- [ ] UI polish and animations
- [ ] Tag extraction and filtering across projects
- [ ] Backlinks between notes
- [ ] Graph view of note connections
- [ ] Export to PDF / HTML
- [ ] Custom themes
- [ ] Global hotkey (Ctrl+Shift+Space)
- [ ] System tray mode
- [ ] Kanban board view for tasks

See [CHECKLIST.md](docs/ai-workflow/CHECKLIST.md) for detailed implementation status.

## Built With AI

![AI Workflow](docs/graphics/ai-workflow.png)

This entire application was built using AI-assisted development -- an approach we call **Open Method**. We share not just the code, but the complete process: the PRD, task breakdowns, handover documents, and workflow artifacts.

Read about the method:
- [The AI Development Workflow I Actually Use](https://dev.to/olaproeis/the-ai-development-workflow-i-actually-use-549i) -- The original workflow article
- [docs/ai-workflow/](docs/ai-workflow/) -- Documentation of the AI-assisted development process used to build Ironpad

**Tools used:** Cursor IDE, Claude Opus 4.5/4.6, Context7 MCP

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| Data directory | `data/` next to executable | Override with `IRONPAD_DATA_DIR` env var |
| Backend port | 3000 (auto-increments to 3010) | Dynamic port selection |
| Auto-commit | Every 60 seconds | Git commits when changes exist |
| Auto-save | 1 second debounce | Frontend saves after typing stops |

## Documentation

![Architecture](docs/graphics/architecture.png)

| Document | Description |
|----------|-------------|
| [docs/API.md](docs/API.md) | Complete REST API reference |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design and technical details |
| [docs/ai-workflow/](docs/ai-workflow/) | AI development workflow and methodology |

## Contributing

This is an early release and contributions are welcome!

1. Check [Issues](https://github.com/OlaProeis/ironPad/issues) for open bugs and feature requests
2. Create a branch for your feature/fix
3. Follow the code style (`cargo fmt` for Rust)
4. Test your changes thoroughly
5. Submit a pull request

## License

[MIT License](LICENSE)

## Acknowledgments

- [Milkdown](https://milkdown.dev/) -- WYSIWYG Markdown editor
- [Axum](https://github.com/tokio-rs/axum) -- Rust web framework
- [Vue.js](https://vuejs.org/) -- Frontend framework
- [Pinia](https://pinia.vuejs.org/) -- State management
- [Anthropic Claude](https://www.anthropic.com/) -- AI-assisted development
