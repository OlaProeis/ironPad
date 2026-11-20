# Library link source resolution (backend)

Turns a stored link `source` plus an optional project folder root into a resolved absolute path. PRD-0.4.0-library §4.3 and §4.6 — every later library endpoint should use this module so paths cannot bypass the rules.

## Module

`backend/src/services/library.rs` (registered in `services/mod.rs`).

| Export | Role |
|--------|------|
| `Scope` | `Project` (relative `source` under client-supplied root) or `Global` (absolute `source`). |
| `LibraryError` | Typed failures with `status_code()` and `user_message()` for JSON responses (same idea as `git_errors.rs` user-facing strings). |
| `validate_project_source(source)` | Non-empty; forward slashes only; no leading `/`; no `..` segment; not absolute (includes Windows drive paths). |
| `validate_global_source(source)` | Non-empty; must be absolute on the host OS. |
| `resolve_source(scope, source, root)` | Project: requires `Some(root)`, joins, canonicalizes root and file (follows symlinks), `OutsideRoot` if canonical file escapes root, `SourceMissing` on `NotFound`. Global: validates absolute path, existence check; returns path as stored (not re-canonicalized). |
| `normalize_relative(path)` | OS path → stored string with `/` (Windows project sources). |

## Error mapping (resolution-related)

| Variant | HTTP | Typical user message |
|---------|------|-------------------------|
| `SourceMissing` | 404 | The linked file was not found. |
| `OutsideRoot` | 400 | Outside project folder root (change root or use global library). |
| `RootRequired` | 400 | Set a folder root for this project. |
| `InvalidSource(_)` | 400 | Validation detail string. |
| `Io(_)` | 500 | Generic read failure (logged with `tracing::warn`). |

Other `LibraryError` variants (`NotRegularFile`, `TooLarge`, `DisallowedSuffix`, `NotUtf8`) are returned by `read_linked_file` — see [`library-linked-file-read.md`](./library-linked-file-read.md).

## Symlinks

Project resolution uses `canonicalize()` on both root and joined path. A symlink inside the root that resolves outside it yields `OutsideRoot` after canonicalization (covered by `#[cfg(unix)]` tests).

## Tests

Unit tests in `library.rs` with `tempfile` (dev-dependency). Targeted run:

```bash
cargo test library::
```

From `backend/`. Filter name is `library::`, not `git::library`.

## Related

- Linked-file read gate: [`library-linked-file-read.md`](./library-linked-file-read.md)
- Pointer frontmatter: [`prompt-pointer-frontmatter.md`](./prompt-pointer-frontmatter.md)
- Product rules: [`PRD-0.4.0-library.md`](../ai-workflow/PRD-0.4.0-library.md) §4.3, §4.6