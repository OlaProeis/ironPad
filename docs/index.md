# Documentation Index

This index provides an overview of all documentation for Ironpad. Attach this file to AI chats when asking questions about documentation structure.

---

## Core Documentation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`/README.md`](../README.md) | Project overview, quick start, installation | First time setup, sharing the project |
| [`/ai-context.md`](../ai-context.md) | Architecture, critical patterns, active work | Every AI chat — attach this first |
| [`/HANDOVER.md`](../HANDOVER.md) | Session handover, recent changes, next tasks | Starting a new session, continuing work |
| [`/ROADMAP.md`](../ROADMAP.md) | Feature roadmap, release planning | Planning features, checking what's next |
| [`/CHANGELOG.md`](../CHANGELOG.md) | Version history, completed features | Understanding what changed when |

---

## Technical Documentation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`API.md`](./API.md) | Complete REST API reference with examples | Implementing new endpoints, frontend API calls |
| [`ARCHITECTURE.md`](./ARCHITECTURE.md) | System design, service layer, data models | Understanding how the system works |
| [`backlinks.md`](./backlinks.md) | Note linking system — `/link` command, BacklinksPanel, link index | Working with backlinks, link API, or the editor |
| [`system-tray-implementation.md`](./system-tray-implementation.md) | System tray mode implementation details | Working with tray functionality |

---

## AI Workflow Documentation

Documentation about the AI-assisted development process used to build Ironpad:

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`ai-workflow/README.md`](./ai-workflow/README.md) | Overview of AI workflow methodology | Understanding the development process |
| [`ai-workflow/PRD.md`](./ai-workflow/PRD.md) | Product requirements document | Understanding original feature specifications |
| [`ai-workflow/method.md`](./ai-workflow/method.md) | Detailed workflow method | Implementing similar AI workflows |
| [`ai-workflow/CHECKLIST.md`](./ai-workflow/CHECKLIST.md) | Implementation checklist | Tracking feature completion |
| [`ai-workflow/tools.md`](./ai-workflow/tools.md) | Tools and setup | Setting up development environment |
| [`ai-workflow/lessons-learned.md`](./ai-workflow/lessons-learned.md) | Development insights | Avoiding past mistakes, understanding decisions |
| [`ai-workflow/HANDOVER.md`](./ai-workflow/HANDOVER.md) | Archived handover notes | Historical session context |

---

## Frontend Documentation

| Document | Purpose | When to Read |
|----------|---------|--------------|
| [`/frontend/README.md`](../frontend/README.md) | Frontend architecture, component structure | Working on Vue components, Milkdown editor |

---

## Documentation Guidelines

### When Adding New Documentation

1. **Technical docs** → Add to `docs/` folder, update this index
2. **Architecture changes** → Update `ai-context.md` AND this index
3. **API changes** → Update `docs/API.md`
4. **Feature specs** → Add to `docs/ai-workflow/` with PRD format
5. **Workflow/process docs** → Add to `docs/ai-workflow/`

### File Naming Conventions

- Use lowercase with hyphens: `api-reference.md`
- README files for folder overviews
- Match document purpose to filename

---

## Quick Reference

**Starting a new feature:**
1. Read `ROADMAP.md` for planned features
2. Read `HANDOVER.md` for current context
3. Read `ai-context.md` for architecture
4. Check `docs/API.md` if adding endpoints

**Debugging an issue:**
1. Read `ai-context.md` for known issues
2. Read `ARCHITECTURE.md` for system flow
3. Read relevant component docs

**Adding documentation:**
1. Choose the right folder (docs/ or docs/ai-workflow/)
2. Update this index
3. Link from relevant files
