use std::fs;
use std::path::PathBuf;

/// OS-specific cache directory for GitHub snapshot data: `{cache}/Ironpad/github/{project_id}`.
pub fn github_cache_dir(project_id: &str) -> PathBuf {
    let base = platform_cache_base().expect("OS cache directory unavailable");
    let dir = base.join("Ironpad").join("github").join(project_id);
    fs::create_dir_all(&dir).expect("create github cache directory");
    dir
}

fn platform_cache_base() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        dirs::data_local_dir()
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs::cache_dir()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_cache_dir_contains_ironpad_and_project_id() {
        let path = github_cache_dir("my-project");
        let path_str = path.to_string_lossy();
        assert!(path_str.contains("Ironpad"));
        assert!(path_str.contains("my-project"));
    }
}
