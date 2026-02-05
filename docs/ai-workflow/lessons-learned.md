# Lessons Learned

What worked, what didn't, and what changed when the context window went from 200K to 1M tokens.

## What Worked Well

### 1. PRD-First Development

Writing a detailed PRD before any code was the single highest-leverage activity. The AI produces dramatically better code when it knows exactly what success looks like.

Ironpad's PRD went through 3 versions. Each round of multi-AI review caught issues that would have been expensive to fix later:
- Concurrent editing race conditions (file locking needed)
- File watcher integration (external editor support)
- Git conflict handling (graceful degradation, not crashes)
- Frontmatter automation (users shouldn't manually edit metadata)

**Lesson:** Time spent on the PRD pays off 10x during implementation.

### 2. Rust's Strict Compiler

Rust stands out for AI-assisted development because the compiler is extraordinarily strict. It catches:
- Memory safety issues
- Type mismatches
- Lifetime problems
- Unused variables and imports
- Missing error handling

With dynamic languages, bugs hide until runtime. With Rust, the AI gets immediate, precise feedback on what's broken. The feedback loop is tighter and more reliable.

`cargo check` became the primary verification tool. If it compiles, a large category of bugs is already eliminated.

**Lesson:** A strict compiler is a massive advantage for AI-generated code.

### 3. The ai-context.md Pattern

Maintaining a lean (~100 line) architectural reference that tells the AI how to write code for this specific project eliminated a whole class of "doesn't fit the codebase" problems.

Without it, the AI would invent new patterns, use different naming conventions, or structure code differently from the existing codebase. With it, code consistently matched existing patterns.

**Lesson:** A small context document is worth more than a large architecture doc. The AI needs a cheat sheet, not a textbook.

### 4. Fresh Chats Over Long Conversations

Context accumulates noise. By the third task in a single chat, the AI references irrelevant earlier context. Starting fresh with a focused handover produced consistently better results.

**Lesson:** Shorter, focused sessions beat long wandering ones.

## What Didn't Work

### 1. Trusting "This Should Work"

The AI confidently says "this should work" when it doesn't. Every single time. Without exception.

Early on, I'd take the AI's word and move on. Then things would break two features later when the untested code interacted with something else.

**Fix:** Test everything yourself. Run the feature. Click the buttons. Try the edge cases. The AI writes code; you verify the product.

### 2. Vague Requirements

"Add search" produces mediocre results. "Add full-text search with ripgrep, triggered by Ctrl+K, showing filename and matching line with context, limited to 5 matches per file, falling back to manual string search if ripgrep isn't available" produces excellent results.

**Fix:** Be specific. The more precise the requirement, the better the code.

### 3. Over-Engineering

The AI tends to add abstractions, patterns, and generalization that aren't needed yet. It builds for a future that may never come.

**Fix:** Explicitly state YAGNI in the context. Call out when something is over-engineered. The AI responds well to "simplify this."

### 4. Ignoring the Editor Lifecycle

The Milkdown WYSIWYG editor had a complex initialization lifecycle that the AI didn't fully understand. This caused a critical bug where switching between notes showed stale content, leading to data loss.

**Fix:** Document critical component lifecycles in ai-context.md. The "Milkdown Editor Lifecycle" section was added after this bug and prevented similar issues.

## The 200K to 1M Context Shift

This was the most significant change in the project's development workflow.

### Before: 200K Tokens (Claude Opus 4.5)

| Aspect | Reality |
|--------|---------|
| Files in context | ~3-5 at once |
| Task granularity | Must split features into 3-5 micro-tasks |
| Handovers | Required between every task |
| Cross-file bugs | Hard to find (AI can't see all files) |
| Refactors | Multi-session, risk of inconsistency |
| Overhead per task | ~15-20 min (handover + context setup) |

### After: 1M Tokens (Claude Opus 4.6)

| Aspect | Reality |
|--------|---------|
| Files in context | Entire codebase (80+ files) |
| Task granularity | Full features in one session |
| Handovers | Only needed between days/sessions |
| Cross-file bugs | Found automatically (AI sees everything) |
| Refactors | Single session, guaranteed consistency |
| Overhead per task | ~0 min |

### The Codebase Audit

The clearest demonstration of the shift: loading the entire Ironpad codebase into a single context and asking "what's wrong?"

The AI found 16 issues, including:
- **Auto-commit silently broken** -- A flag that was never set to `true` anywhere in the codebase. Finding this required reading `main.rs`, `git.rs`, and every route handler simultaneously.
- **Operator precedence bug** -- `0 > 0` evaluated before `??` due to JavaScript precedence rules. Subtle and easy to miss.
- **Missing atomic writes** -- Only one of eight write paths used the safe atomic write pattern.

14 of 16 issues were fixed in a single session. Zero compilation errors introduced.

This type of comprehensive audit was not practical at 200K tokens.

### What Didn't Change

The 1M context window doesn't change everything:
- **PRDs are still essential.** More context doesn't substitute for clear requirements.
- **Testing is still essential.** The AI still says "this should work" when it doesn't.
- **Specificity still matters.** Vague asks still produce vague results.
- **Handovers still matter** between sessions (sleep, context switches, etc.)

The context window is a force multiplier, not a replacement for process.

## Advice for Others

1. **Start with the PRD.** It's the highest-leverage activity.
2. **Use a strict language if you can.** Rust, TypeScript (strict mode), Go -- anything with a compiler that catches bugs.
3. **Maintain ai-context.md.** Keep it under 100 lines. Update it when patterns change.
4. **Test everything.** Don't read code. Run the thing.
5. **Use multiple AI models.** They have different blind spots.
6. **Be specific.** The more precise the requirement, the better the result.
7. **Keep sessions focused.** One task, one chat (at 200K). One feature, one chat (at 1M).
