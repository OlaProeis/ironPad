# Prompt create/update: link pointers

Release **0.4.0** library work: `POST` and `PUT` on `/api/prompts` accept optional `kind`, `source`, and `template` so clients can create and maintain **link** prompt files without writing prompt body text. GET resolution of linked files remains a separate task.

## Request fields

| Field | Create | Update |
|-------|--------|--------|
| `kind` | `prompt` (default) or `link` | May be sent only to confirm existing kind; **changing** kind returns **400** |
| `source` | Required when `kind: link` | Optional for links; re-validated when provided |
| `template` | Optional; default `false` | Optional for links |
| `content` | Ignored when `kind: link` | Ignored when the stored prompt is a link |

Plain prompts behave as before: `content` is written to the file body and pointer keys stay omitted on disk (see [`prompt-pointer-frontmatter.md`](./prompt-pointer-frontmatter.md)).

## Create behavior (`backend/src/services/prompts.rs`)

1. **Link** (`kind: link`):
   - Requires non-empty `source`.
   - **Project scope:** `library::validate_project_source` (relative path, forward slashes, no `..`, no leading `/`).
   - **Global scope:** `library::validate_global_source` (absolute path on this machine).
   - Any `content` in the JSON is ignored; the markdown body is **empty**.
   - Frontmatter gets `kind: link`, `source`, and `template` only when `true` via `PromptPointerFields::apply_to_mapping`.

2. **Prompt** (default or explicit `prompt`):
   - Writes `content` (default empty string).
   - `prepare_mapping_for_write` strips pointer keys before save.

## Update behavior

1. Load existing pointer with `PromptPointerFields::from_mapping`.
2. If `kind` in the request differs from stored kind → **400** (`Prompt kind cannot be changed.`).
3. **Link:** may update `title`, `folder`, `tags`, `description`, `source` (re-validated per scope), `template`; **does not** change body from `content`; linked **source** files on disk are never read or written.
4. **Prompt:** `content` updates the body; pointer keys remain normalized with `prepare_mapping_for_write`.

## HTTP errors (`backend/src/routes/prompts.rs`)

Validation and library shape errors return **400** when `prompts::is_client_error(&err)` is true (missing `source`, kind change, `LibraryError` user messages from validation helpers, missing `project_id`, project not found, etc.). Other failures stay **500**; missing prompt id on update stays **404**.

There is **no** backend duplicate-prompt endpoint; duplication is frontend-driven if present.

## Tests

Service tests in `backend/src/services/prompts.rs` module `link_pointer_tests` (temp `IRONPAD_DATA_DIR`):

```bash
cargo test link_pointer_tests -- --test-threads=2
```

Covers project link with ignored content and empty body, rejected `../` project source, rejected relative global source, link `template` update without body change, plain prompt without `kind` key, and rejected kind change on update.

## Related

- Frontmatter model: [`prompt-pointer-frontmatter.md`](./prompt-pointer-frontmatter.md)
- Source path rules: [`library-source-resolution.md`](./library-source-resolution.md)
- PRD: [`../ai-workflow/PRD-0.4.0-library.md`](../ai-workflow/PRD-0.4.0-library.md)