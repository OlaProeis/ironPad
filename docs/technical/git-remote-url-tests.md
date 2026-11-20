# Git remote URL validation tests (backend)

Table-driven and temp-repo tests lock in `validate_remote_url` and the validate-then-`origin` flow without live Git hosts or network.

## Location

`backend/src/services/git.rs` — nested module `#[cfg(test)] mod tests { mod remote { ... } }`.

## What is covered

| Area | Test |
|------|------|
| Allowed schemes | `https://`, `http://`, `ssh://`, `git@host:path`; trimmed URLs |
| Rejections | Empty/whitespace, `file://`, Unix/Windows path-like URLs, HTTP(S) credentials in userinfo, `ssh://user:pass@...`, bad schemes, malformed SCP-style |
| Temp repo | Valid URLs add `origin` via git2; invalid URLs leave repo without `origin`; replace updates `origin` URL |
| API guard | `set_remote_origin` returns validation errors before opening `data/` (no `IRONPAD_DATA_DIR` needed for invalid URLs) |

Helper in tests: `add_origin_if_valid` mirrors validate + `repo.remote` / `remote_set_url` (same steps as `set_remote_origin`, without fetch).

## Running tests

From `backend/`:

```bash
cargo test remote:: -- --test-threads=2
```

All git service tests (pull/push/remote):

```bash
cargo test services::git::tests -- --test-threads=2
```

Note: `cargo test git::remote` matches **no** tests on the `ironpad` binary; the filter prefix is `remote::` or `services::git::tests::remote::`.

## Related

- Validation rules and API: [`git-remote-origin.md`](./git-remote-origin.md)
- Credential-in-URL policy: [`git-credential-helper.md`](./git-credential-helper.md)