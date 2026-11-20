# Prompt library pointer frontmatter

Library release **0.4.0** treats some markdown files under prompt folders as **links** to external files, not editable prompt bodies. Task foundation: typed frontmatter fields and list/detail exposure without resolving `source` paths.

## Fields

| Key | When present | Meaning |
|-----|----------------|---------|
| `kind` | Links only on disk | `link` for a pointer; omitted for normal prompts |
| `source` | Links | Path string (relative or absolute; validation in later tasks) |
| `template` | Links, when `true` | Enables `{{variables}}` for that link (later tasks) |

**Backward compatibility:** Files with no `kind`, or with `kind: prompt`, parse as **`PromptKind::Prompt`**. `source` is `None` and `template` is `false`.

## Backend types (`backend/src/models/prompt.rs`)

- **`PromptKind`**: `prompt` | `link` (serde `rename_all = "lowercase"`, default `prompt`).
- **`PromptPointerFields`**: `kind`, `source`, `template` — parse/serialize rules for these keys only.
- **`PromptSummary`** / **`Prompt`**: include `kind` and `source`; full **`Prompt`** also includes `template`.

## Parsing and writing

- **`PromptPointerFields::from_mapping(&Mapping)`** reads pointer fields from an already-parsed YAML frontmatter mapping (list/detail never open linked **source** files — only the pointer `.md` file).
- **`apply_to_mapping` / `prepare_mapping_for_write`** normalize keys before save:
  - **Prompts:** remove `kind`, `source`, and `template` so existing prompt files are not rewritten with link keys.
  - **Links:** write `kind: link`, `source`, and `template` only when `true`.

Custom **`Serialize`** on `PromptPointerFields` omits all three keys for prompts; links emit `kind` + `source`, and `template` only when set.

**Call sites:** `build_summary` / `build_prompt` set API fields from `from_mapping`. Create/update write rules for links vs prompts (validation, empty link body, immutable `kind`) are in [`prompt-link-create-update.md`](./prompt-link-create-update.md). Plain prompts still call `prepare_mapping_for_write` before `serialize_frontmatter`; links use `apply_to_mapping` with validated `source`.

## Tests

Unit tests live in `models::prompt::tests` in `backend/src/models/prompt.rs`:

```bash
cargo test models::prompt::
```

Coverage includes default prompt parsing, explicit `kind: prompt`, link round-trip, prompt serialization without link keys, and link serialization with `kind` + `source`.

## Related scope (not in this doc’s implementation)

Path resolution, reading external file bytes, browse/open endpoints, and frontend library UI are covered in [`../ai-workflow/PRD-0.4.0-library.md`](../ai-workflow/PRD-0.4.0-library.md) and follow-on tasks.