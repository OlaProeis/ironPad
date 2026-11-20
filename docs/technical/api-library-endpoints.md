# Library endpoints (API reference)

Canonical HTTP shapes for the 0.4 **Library** in **[`API.md`](../API.md)**:

| Surface | Role |
|---------|------|
| **Prompts (Library)** | `kind`, `source`, `template` on `POST`/`PUT` `/api/prompts`; link fields on `GET /api/prompts/:id?root=` |
| `GET /api/library/browse?path=` | Non-recursive directory listing for Add link |
| `POST /api/library/open` | Open a link pointer by `id` (optional `root`) via the OS default app |

**`API.md` includes:** pointer vs prompt behavior, `root` query semantics, GET response fields (`read_only`, `resolved_path`, `source_missing`, `root_required`, `source_error`, `supports_rendered`), browse `entries[].linkable`, and error status tables.

**Release artifacts:** Library notes under undated `## [0.4.0]` in [`CHANGELOG.md`](../../CHANGELOG.md); [`ROADMAP.md`](../../ROADMAP.md) lists the library PRD as in-tree; acceptance rows in [`docs/ai-workflow/CHECKLIST.md`](../ai-workflow/CHECKLIST.md) mirror [`PRD-0.4.0-library.md`](../ai-workflow/PRD-0.4.0-library.md) §11.

**Implementation detail (not duplicated in API.md):**

| Topic | Doc |
|-------|-----|
| Link GET resolution | [`library-link-get.md`](./library-link-get.md) |
| Browse listing | [`library-browse.md`](./library-browse.md) |
| Open externally | [`library-open.md`](./library-open.md) |
| Create/update payloads | [`prompt-link-create-update.md`](./prompt-link-create-update.md) |
| Frontend client | [`frontend-library-api.md`](./frontend-library-api.md) |

Route modules: `backend/src/routes/prompts.rs` (`PromptGetQuery.root`), `backend/src/routes/library.rs` (`/browse`, `/open`).