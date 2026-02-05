# The Development Method

This document describes the AI-assisted development workflow used to build Ironpad from scratch.

## Overview

The method has six phases, applied iteratively for each feature:

1. **Multi-AI Consultation** -- Get perspectives from multiple AI models before coding
2. **PRD Creation** -- Write a detailed product requirements document
3. **Task Decomposition** -- Break the PRD into ordered, dependency-aware tasks
4. **Context Loading** -- Feed the AI current documentation and project context
5. **Implementation** -- Build features in focused sessions with handovers
6. **Verification** -- Test everything yourself; don't trust "this should work"

## Phase 1: Multi-AI Consultation

Before writing any code, discuss the idea with different AI assistants:

- **Claude** for architecture and code design
- **Perplexity** for research on libraries, crate versions, and known issues
- **Gemini** for alternative perspectives and catching blind spots

Each AI has different strengths and blind spots. Five minutes getting multiple opinions saves hours of rework later.

**Example from Ironpad:** When designing the task system, one model suggested storing tasks as checkboxes in a single `tasks.md` file. Another pointed out that individual task files with frontmatter would be more flexible and avoid concurrent edit conflicts. We went with individual files, which turned out to be the right call.

## Phase 2: PRD Creation

Task Master (and AI in general) produces dramatically better results when it knows exactly what success looks like. The PRD captures:

- Problem statement and goals
- Detailed feature specifications
- Technical architecture decisions
- API design
- Data model
- Edge cases and error handling
- Non-goals (equally important)

After drafting, run the PRD through other AIs for review. Iterate until it's tight.

**Ironpad's PRD** went through 3 versions, incorporating feedback about concurrency control, file watching, git conflict handling, and frontmatter automation -- all before a single line of code was written.

## Phase 3: Task Decomposition

Use Task Master to parse the PRD into structured tasks with dependencies. Each task should have:

- Clear inputs (what files/context are needed)
- Clear outputs (what gets created/changed)
- Explicit dependencies (what must be done first)
- Acceptance criteria (how to verify it works)

You don't need Task Master specifically. What matters is having explicit, ordered tasks rather than vague goals like "add search."

## Phase 4: Context Loading

AI models have training cutoffs. The library docs they know might be outdated.

- **Context7** (MCP tool) pulls current documentation into context for any library
- **ai-context.md** is a lean architectural reference (~100 lines) telling the AI how to write code that fits the codebase
- **Handover documents** carry context between sessions

### The ai-context.md Pattern

This file tells the AI *how* to write code that belongs in this project:

- Module structure and naming conventions
- Key types and their relationships
- Framework idioms (Axum patterns, Vue composition API patterns)
- Critical gotchas that cause bugs (e.g., Milkdown editor lifecycle)
- Current implementation status

It's not a full architecture doc. It's a cheat sheet for the AI.

## Phase 5: Implementation

### The Handover System (200K Context)

With 200K token models, each task gets a fresh chat:

1. Open new chat
2. Paste handover document with: rules, relevant files, current task
3. Work on task until done
4. AI updates the handover document for the next task
5. Close chat, repeat

**Why fresh chats?** Context accumulates noise. Three tasks in, the AI references irrelevant stuff from earlier. Starting clean with a focused handover produces better results.

### The Full-Context Approach (1M Context)

With 1M token models (Claude Opus 4.6), the workflow simplifies:

1. Load the entire codebase into context
2. Work on features directly -- the AI sees everything
3. Use handovers only for session boundaries (end of day, sleep, etc.)

The handover system doesn't disappear -- it shifts from "required between every task" to "useful between sessions."

## Phase 6: Verification

The AI writes the code. You verify the product.

- Run the feature and see if it works
- Test edge cases manually
- Check that nothing else broke
- Use compiler output (`cargo check`) and linters as mechanical verification

Don't read code line by line. Run the thing and see if it works. When something's wrong, describe the problem clearly. The AI debugs from there.

## How This Played Out for Ironpad

| Phase | What Happened |
|-------|---------------|
| 1-3 | PRD v3.0 with architecture decisions, reviewed by multiple AIs |
| 4 | ai-context.md maintained throughout, Context7 for Axum/Vue/Milkdown docs |
| 5 | Phases 1-3 built with Opus 4.5 (200K), phases 4-5 with Opus 4.6 (1M) |
| 6 | Every feature manually tested in the browser |

Total development time: approximately 2 weeks from PRD to working application with dashboard, calendar, git panel, WYSIWYG editor, subtasks, recurring tasks, and real-time sync.
