# Project `github_repo` on GET responses

Release **0.4.0** GitHub work stores a linked repository on each project’s `data/projects/{id}/index.md` frontmatter as `github_repo: owner/name`. This task exposes that key on project GET JSON so the frontend can tell whether a repo is configured before dedicated GitHub routes exist.

## Frontmatter

- Key: `github_repo`
- Value: string `owner/repo` (github.com), or omit the key when unlinked
- `PUT /api/projects/{id}/content` rewrites the markdown body and updates `updated` only; existing frontmatter keys (including `github_repo`) are preserved without handler changes

## API shapes

Handlers in `backend/src/routes/projects.rs`:

| Endpoint | Struct | `github_repo` |
|----------|--------|----------------|
| `GET /api/projects` | `Project` | `null` or string |
| `GET /api/projects/{id}` | `Project` | same |
| `GET /api/projects/{id}/content` | `ProjectWithContent` | same |
| `POST /api/projects` | `Project` | always `null` (new projects have no link) |
| `PUT /api/projects/{id}/content` | `ProjectWithContent` | read back from frontmatter after save |

Parsing uses `github_repo_from_frontmatter` on the parsed YAML mapping (`serde_yaml::Value::from("github_repo")`, `.as_str().map(String::from)`).

## Cargo dependencies (foundation)

Added for later GitHub API and cache paths; not used in application code yet:

- `reqwest = { version = "0.12", features = ["json", "gzip"] }`
- `dirs = "5"`

## Tests

Module `list_projects_tests` in `routes/projects.rs`:

- Temp data dir via `IRONPAD_DATA_DIR` + `config::init_data_dir()` (same pattern as task meta tests)
- `list_projects_impl_reads_github_repo_from_frontmatter` — index with `github_repo: owner/repo`
- `list_projects_impl_github_repo_none_when_absent` — index without the key

Filter: `cargo test -- routes::projects::list_projects_tests`

## Dedicated GitHub routes

Save/clear link and read snapshot: [`project-github-routes.md`](./project-github-routes.md) (`GET`/`PUT /api/projects/{id}/github`). GitHub REST refresh remains a later task.