use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubLabel {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubIssue {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub updated_at: String,
    pub author_login: String,
    pub labels: Vec<GithubLabel>,
    pub comments_count: u64,
    pub milestone_due: Option<String>,
    pub assignee_logins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GithubPull {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub updated_at: String,
    pub author_login: String,
    pub labels: Vec<GithubLabel>,
    pub comments_count: u64,
    pub milestone_due: Option<String>,
    pub draft: bool,
    pub requested_reviewer_logins: Vec<String>,
    pub head_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub repo: String,
    pub login: Option<String>,
    pub refreshed_at: Option<String>,
    pub issue_etag: Option<String>,
    pub pull_etag: Option<String>,
    pub issues: Vec<GithubIssue>,
    pub pulls: Vec<GithubPull>,
    pub issues_truncated: bool,
    pub pulls_truncated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn snapshot_round_trip_json() {
        let snapshot = Snapshot {
            repo: "owner/repo".to_string(),
            login: Some("octocat".to_string()),
            refreshed_at: Some("2026-01-01T00:00:00Z".to_string()),
            issue_etag: Some("W/\"issues\"".to_string()),
            pull_etag: None,
            issues: vec![GithubIssue {
                number: 1,
                title: "Bug".to_string(),
                html_url: "https://github.com/owner/repo/issues/1".to_string(),
                updated_at: "2026-01-02T00:00:00Z".to_string(),
                author_login: "alice".to_string(),
                labels: vec![GithubLabel {
                    name: "bug".to_string(),
                    color: Some("ff0000".to_string()),
                }],
                comments_count: 3,
                milestone_due: Some("2026-06-01".to_string()),
                assignee_logins: vec!["bob".to_string()],
            }],
            pulls: vec![GithubPull {
                number: 2,
                title: "Fix".to_string(),
                html_url: "https://github.com/owner/repo/pull/2".to_string(),
                updated_at: "2026-01-03T00:00:00Z".to_string(),
                author_login: "carol".to_string(),
                labels: vec![],
                comments_count: 0,
                milestone_due: None,
                draft: true,
                requested_reviewer_logins: vec!["dave".to_string()],
                head_sha: "abc123".to_string(),
            }],
            issues_truncated: false,
            pulls_truncated: true,
        };

        let json = serde_json::to_string(&snapshot).expect("serialize");
        let decoded: Snapshot = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded, snapshot);
    }
}
