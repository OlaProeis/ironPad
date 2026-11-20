# Git credential helper (HTTPS) and SSH agent

Fetch and push authenticate by remote URL scheme. The app does not store passwords or tokens.

## Service layer

`backend/src/services/git.rs`

| Symbol | Role |
|--------|------|
| `remote_auth_kind(url)` | `http://` and `https://` → credential helper; everything else (`ssh://`, `git@host:path`) → SSH agent. |
| `remote_credentials` | Callback used by `fetch_from_remote` and `push_to_remote`. |

HTTPS/HTTP calls `git2::Cred::credential_helper` with `Config::open_default()` so Git's `credential.helper` (for example Git Credential Manager) supplies the username and password. SSH calls `git2::Cred::ssh_key_from_agent`.

Embedded credentials in remote URLs stay rejected by `validate_remote_url` (see [`git-remote-origin.md`](./git-remote-origin.md)).

## Tests

Unit tests `auth_kind_https_uses_credential_helper` and `auth_kind_ssh_uses_agent` in `backend/src/services/git.rs`. They check scheme selection only; they do not contact a network or a credential helper.
