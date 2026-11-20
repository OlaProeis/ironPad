# GET link content (backend)

`GET /api/prompts/{id}` reads a library link's source file at request time and returns that text as `content`. The project folder root is only in browser `localStorage`, so the client sends it as an optional query parameter: `GET /api/prompts/{id}?root=<absolute directory>`.

Complements pointer frontmatter ([`prompt-pointer-frontmatter.md`](./prompt-pointer-frontmatter.md)), path rules ([`library-source-resolution.md`](./library-source-resolution.md)), and the read gate ([`library-linked-file-read.md`](./library-linked-file-read.md)).

## Where

| Piece | Role |
|-------|------|
| `PromptGetQuery.root` in `backend/src/routes/prompts.rs` | Optional client-supplied root. Empty or whitespace is treated as absent. |
| `get_prompt_for_read` in `backend/src/services/prompts.rs` | Builds `PromptGetResponse`. Prompts keep the stored body. Links call resolve then read. |
| `PromptGetResponse` in `backend/src/models/prompt.rs` | Flattens `Prompt` and adds `read_only`, `source_missing`, `resolved_path`, `supports_rendered`, `root_required`, `source_error`. |
| `PromptFileStore` and `resolve_link_by_id` in `backend/src/services/library.rs` | Id → stored `scope`/`source` → `resolve_source`. Reuse this for later open/browse work. |
| `unresolved_link_path` | Path string to report when resolve does not return a path: `root.join(source)`, or the stored `source` when a project link has no root. |

`get_prompt` still returns the stored pointer body only. Request-time link file text is only on this GET; semantic search uses metadata for links — see [`prompt-semantic-search.md`](./prompt-semantic-search.md).

## Response

Always **200** for a prompt that exists, including soft link failures, so the pointer stays editable. Missing prompt id is still **404**. Unexpected I/O is **500**.

| Case | Body |
|------|------|
| Plain prompt | Stored `content`, `kind: "prompt"`, `read_only: false`, `source_missing: false`. `root_required` omitted when false. |
| Link read ok | `content` is the file text (not cached). `resolved_path` is the absolute path. `read_only: true`, `source_missing: false`, `supports_rendered` from `supports_rendered(path)`. |
| `SourceMissing` | `content: ""`, `source_missing: true`, `resolved_path` from `unresolved_link_path`. |
| `RootRequired` | `content: ""`, `root_required: true`, `resolved_path` is the stored source when root is absent. |
| `TooLarge`, `DisallowedSuffix`, `NotUtf8`, `NotRegularFile`, `OutsideRoot`, `InvalidSource` | `content: ""`, `source_error` is `LibraryError::user_message()`. |

## Tests

`library::tests::resolve_link_by_id_reads_pointer` and `prompts::link_pointer_tests::get_link_and_prompt_content_follow_request_time_rules` (temp `data/` plus temp root; overwrite then GET again; deleted source; project link without `root`; plain prompt body).
