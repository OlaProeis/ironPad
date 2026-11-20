# Library linked-file read gate (backend)

Single gate for reading a **already resolved** absolute path: used by future read, browse (`linkable` flag), and open endpoints. PRD-0.4.0-library — complements path resolution in [`library-source-resolution.md`](./library-source-resolution.md).

## Module

`backend/src/services/library.rs`.

| Export | Role |
|--------|------|
| `MAX_LINK_BYTES` | `1_048_576` (1 MiB); size checked from metadata **before** reading body. |
| `ALLOWED_SUFFIXES` | `md`, `markdown`, `txt`, `rs`, `vue`, `ts`, `js`, `toml`, `json`, `yml`, `yaml`, `py`, `css`, `html`, `mdc`. |
| `suffix_allowed(path)` | Case-insensitive extension check (no extension → false). |
| `supports_rendered(path)` | True only for `md` / `markdown` (drives rendered vs raw open). |
| `read_linked_file(path)` | `metadata` → must be regular file → allowed suffix → size ≤ max → `fs::read` → `String::from_utf8` (never lossy). |

## `read_linked_file` flow

1. `fs::metadata` → `LibraryError::Io` on failure.
2. Not `is_file()` → `NotRegularFile`.
3. Fails `suffix_allowed` → `DisallowedSuffix` (lowercase extension string, or empty if none).
4. `meta.len() > MAX_LINK_BYTES` → `TooLarge { size, max }` (user message includes byte size).
5. Read bytes → `Io` on failure.
6. Invalid UTF-8 → `NotUtf8`.

## Error mapping (read-related)

| Variant | HTTP | User message |
|---------|------|----------------|
| `NotRegularFile` | 400 | Only regular files can be linked. |
| `DisallowedSuffix(ext)` | 400 | Files with the “{ext}” suffix cannot be linked. |
| `TooLarge { size, max }` | 400 | Linked file is too large ({size} bytes; maximum is {max} bytes). |
| `NotUtf8` | 400 | The file is not UTF-8 text. |
| `Io(_)` | 500 | Could not read the linked file. |

Resolution errors (`SourceMissing`, `OutsideRoot`, etc.) are documented in [`library-source-resolution.md`](./library-source-resolution.md).

## Tests

Unit tests in `library.rs` (`tempfile`): uppercase `.MD` UTF-8 round-trip, directory → `NotRegularFile`, `.exe` → `DisallowedSuffix`, `set_len(1_048_577)` → `TooLarge` with `Display` containing `1048577`, exactly 1 MiB passes, invalid UTF-8 bytes → `NotUtf8`, `supports_rendered` for md/markdown vs txt.

Targeted run from `backend/`:

```bash
cargo test library::
```

## Related

- Source resolution: [`library-source-resolution.md`](./library-source-resolution.md)
- Product rules: [`PRD-0.4.0-library.md`](../ai-workflow/PRD-0.4.0-library.md)