# Prompt semantic search (backend)

`GET /api/prompts/search/semantic` ranks prompts by a lightweight token scorer in `backend/src/services/prompts.rs`. Library **links** are matched from stored metadata only; linked files are never opened for search.

Related: list filter `q` on `GET /api/prompts` ([`prompt-pointer-frontmatter.md`](./prompt-pointer-frontmatter.md)); request-time link body on [`library-link-get.md`](./library-link-get.md) (`get_prompt_for_read`).

## Where

| Piece | Role |
|-------|------|
| `semantic_search` | Lists prompts, scores each, sorts by score, applies `limit`. |
| `prompt_for_semantic_search` | Builds a `Prompt` from `PromptSummary` for links (empty `content`). |
| `semantic_score` | Token match over title, tags, description; body for plain prompts; folder + `source` for links. |
| `prompt_matches_filters` | Simple substring `q` on list API; links also include `source` in the haystack. |

For `kind == link`, `semantic_search` does **not** call `get_prompt` or `library::read_linked_file`. Plain prompts still use `get_prompt` so body text remains searchable.

## Link vs prompt scoring

| Field | Prompt | Link |
|-------|--------|------|
| title, tags, description | yes | yes |
| stored body / pointer body | yes (body weight) | no |
| `folder` | no | yes |
| `source` path | no | yes (tokens + substring bonus on path) |

Semantic expansions (e.g. `roadmap` → `strategy`, `steps`) apply to all kinds.

## Tests

`prompts::link_pointer_tests::semantic_search_matches_link_source_not_linked_file_body`: project link with `source: docs/ROADMAP.md` and title `Plan` matches query `roadmap`; a term that exists only in the on-disk `ROADMAP.md` body does not match.

Filter: `cargo test semantic_search_matches_link_source`.