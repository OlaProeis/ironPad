# GET library browse (backend)

`GET /api/library/browse?path=<absolute directory>` returns a **non-recursive** one-level listing for the Add-link dialog. The client supplies an absolute path (project folder root from `localStorage`, or a pasted directory in the global library).

Complements the read gate ([`library-linked-file-read.md`](./library-linked-file-read.md)) and path rules ([`library-source-resolution.md`](./library-source-resolution.md)).

## Where

| Piece | Role |
|-------|------|
| `backend/src/routes/library.rs` | Router mounted at `/api/library`; handler `GET /browse`. |
| `browse_directory` in `backend/src/services/library.rs` | Validates path, reads `fs::read_dir`, builds sorted entries and `linkable` flags. |
| `BrowseResponse` / `BrowseEntry` | JSON: `path`, `parent`, `entries[]` with `name`, `path`, `is_dir`, `size`, `linkable`. |

## Request

| Query | Required | Rule |
|-------|----------|------|
| `path` | yes | Must be an absolute path to an **existing directory**. |

## Errors

| Condition | Status | Body |
|-----------|--------|------|
| Relative or non-absolute | 400 | Plain text: browse path must be absolute. |
| Path exists but is not a directory | 400 | Plain text. |
| Directory not found | 404 | Plain text. |
| Unexpected I/O | 500 | Plain text. |

## Response

- `path`: canonical absolute directory.
- `parent`: canonical parent directory, if any (for “up” navigation).
- `entries`: sorted **directories first**, then files, case-insensitive by `name`. The `.git` entry is skipped. Other dotfiles are included.

`linkable` is `true` only when the entry is a regular file (not a directory), `suffix_allowed(path)` passes, and `size <= MAX_LINK_BYTES` (1 MiB). Oversize or disallowed suffix files still appear in the list with `linkable: false`.

Entries whose `metadata()` fails are omitted (not listed).

## Tests

`routes::library::tests::*` (fixture: `docs/`, `ai-context.md`, oversize `big.md`, `app.exe`; relative/file/missing paths) and `browse_route_returns_json` via `tower::ServiceExt::oneshot`. Filter: `cargo test library`.