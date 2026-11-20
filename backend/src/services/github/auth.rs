use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const RESOLVE_ERR: &str =
    "GitHub needs a login. Run gh auth login, or set GH_TOKEN for this app. Ironpad does not store a token.";

/// Resolve a GitHub API token: `gh auth token`, then `GH_TOKEN`, then `GITHUB_TOKEN`.
pub fn resolve_token() -> Result<String, String> {
    if let Some(token) = token_from_gh() {
        return Ok(token);
    }
    if let Ok(token) = std::env::var("GH_TOKEN") {
        let token = token.trim();
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        let token = token.trim();
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }
    Err(RESOLVE_ERR.to_string())
}

fn token_from_gh() -> Option<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = Command::new("gh")
            .args(["auth", "token"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(output)) if output.status.success() => {
            let token = String::from_utf8(output.stdout).ok()?;
            let token = token.trim();
            if token.is_empty() {
                None
            } else {
                Some(token.to_string())
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    fn env_lock() -> &'static Mutex<()> {
        ENV_LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn remove(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    /// Path with no `gh` so env-based resolution is exercised deterministically.
    fn path_without_gh() -> &'static str {
        #[cfg(windows)]
        {
            "C:\\Windows\\System32"
        }
        #[cfg(not(windows))]
        {
            "/usr/bin"
        }
    }

    #[test]
    fn resolve_token_uses_gh_token_env() {
        let _lock = env_lock().lock().expect("env lock");
        let _path = EnvVarGuard::set("PATH", path_without_gh());
        let _github = EnvVarGuard::remove("GITHUB_TOKEN");
        let _set = EnvVarGuard::set("GH_TOKEN", "test-value");

        assert_eq!(resolve_token().expect("token"), "test-value");
    }

    #[test]
    fn resolve_token_err_when_no_login() {
        let _lock = env_lock().lock().expect("env lock");
        let _path = EnvVarGuard::set("PATH", path_without_gh());
        let _gh = EnvVarGuard::remove("GH_TOKEN");
        let _github = EnvVarGuard::remove("GITHUB_TOKEN");

        let err = resolve_token().expect_err("expected error");
        assert!(err.contains("gh auth login"));
    }
}
