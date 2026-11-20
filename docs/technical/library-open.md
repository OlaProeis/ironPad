# POST library open (backend)

`POST /api/library/open` resolves a link prompt **by id** (same rules as GET read) and opens the target file with the OS default application. The client never sends a file path—only `id` and optional `root` for project-scoped links.

Complements link GET ([`library-link-get.md`](./library-link-get.md)), path resolution ([`library-source-resolution.md`](./library-source-resolution.md)), and the read gate ([`library-linked-file-read.md`](./library-linked-file-read.md)). UI wiring is a later task.

## Where

| Piece | Role |
|-------|------|
| `backend/src/routes/library.rs` | `POST /open` on the router nested at `/api/library`. |
| `resolve_for_open` | `resolve_link_by_id` then `preflight_linked_file` (metadata + suffix only; no file body read). |
| `launch` | Spawns the OS opener: Windows `cmd /C start "" <path>`, macOS `open`, Linux `xdg-open`. Path is one argument, never interpolated into a shell string. |
| `map_open_error` | Unknown prompt id (`Prompt not found:`) maps to **404**; other errors use `LibraryError::status_code`. |
| `preflight_linked_file` in `backend/src/services/library.rs` | Shared with `read_linked_file` for regular-file + allowed-suffix checks before read size/UTF-8. |

## Request

JSON body:

| Field | Required | Rule |
|-------|----------|------|
| `id` | yes | Link prompt id to resolve. |
| `root` | no | Project folder root for project-scoped links (same semantics as `GET /api/prompts/{id}?root=`). |

A client-supplied `path` field in the body is ignored; opening never uses client paths.

## Success

**200** JSON:

```json
{ "ok": true, "resolved_path": "<absolute path>" }
```

## Errors

Plain-text body (`LibraryError::user_message()` unless noted).

| Condition | Status |
|-----------|--------|
| Linked file not found (`SourceMissing`) | 404 |
| Unknown prompt id | 404 |
| Not a link prompt, bad source, outside root, disallowed suffix, not a regular file, root required, etc. | 400 (per `LibraryError`) |
| Failed to spawn OS open | 500 (`Could not open the linked file.`) |

## Tests

`routes::library::tests::resolve_for_open_*` cover resolution only (`launch` is not called in tests). Filter: `cargo test library`.