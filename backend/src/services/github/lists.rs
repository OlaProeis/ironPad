use std::cell::RefCell;

use serde_json::Value;

use super::client::{GithubClient, GithubError};
use super::snapshot::{GithubIssue, GithubLabel, GithubPull};

#[cfg(test)]
thread_local! {
    static TEST_API_BASE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Override the GitHub API origin for tests (`http://127.0.0.1:<port>`).
#[cfg(test)]
pub fn set_test_api_base(base: Option<String>) {
    TEST_API_BASE.with(|cell| *cell.borrow_mut() = base);
}

fn api_root() -> String {
    #[cfg(test)]
    {
        let override_base = TEST_API_BASE.with(|cell| cell.borrow().clone());
        if let Some(base) = override_base {
            return base.trim_end_matches('/').to_string();
        }
    }
    "https://api.github.com".to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchResult {
    pub issues: Vec<GithubIssue>,
    pub issue_etag: Option<String>,
    pub issues_truncated: bool,
    pub pulls: Vec<GithubPull>,
    pub pull_etag: Option<String>,
    pub pulls_truncated: bool,
    pub issues_304: bool,
    pub pulls_304: bool,
}

fn issues_url(owner: &str, repo: &str) -> String {
    format!(
        "{}/repos/{owner}/{repo}/issues?state=open&sort=updated&direction=desc&per_page=100",
        api_root()
    )
}

fn pulls_url(owner: &str, repo: &str) -> String {
    format!(
        "{}/repos/{owner}/{repo}/pulls?state=open&sort=updated&direction=desc&per_page=100",
        api_root()
    )
}

fn user_url() -> String {
    format!("{}/user", api_root())
}

pub(crate) fn github_api_url(path: &str) -> String {
    format!("{}/{}", api_root(), path.trim_start_matches('/'))
}

pub async fn fetch_lists(
    client: &GithubClient,
    owner: &str,
    repo: &str,
    issue_etag: Option<&str>,
    pull_etag: Option<&str>,
) -> Result<FetchResult, GithubError> {
    let issues_url = issues_url(owner, repo);
    let pulls_url = pulls_url(owner, repo);

    let (issues, issue_etag, issues_truncated, issues_304) =
        match client.get_paginated::<Value>(&issues_url, issue_etag).await {
            Ok((raw, etag, truncated)) => (issues_from_page(raw)?, etag, truncated, false),
            Err(GithubError::NotModified) => {
                (Vec::new(), issue_etag.map(|s| s.to_string()), false, true)
            }
            Err(e) => return Err(e),
        };

    let (pulls, pull_etag, pulls_truncated, pulls_304) =
        match client.get_paginated::<Value>(&pulls_url, pull_etag).await {
            Ok((raw, etag, truncated)) => (pulls_from_page(raw)?, etag, truncated, false),
            Err(GithubError::NotModified) => {
                (Vec::new(), pull_etag.map(|s| s.to_string()), false, true)
            }
            Err(e) => return Err(e),
        };

    Ok(FetchResult {
        issues,
        issue_etag,
        issues_truncated,
        pulls,
        pull_etag,
        pulls_truncated,
        issues_304,
        pulls_304,
    })
}

pub async fn fetch_login(client: &GithubClient) -> Result<String, GithubError> {
    let (body, _etag) = client.get_json::<Value>(&user_url(), None).await?;
    login_from_user_body(&body)
}

fn login_from_user_body(body: &Value) -> Result<String, GithubError> {
    body.get("login")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| GithubError::Network("missing login in /user response".to_string()))
}

fn issues_from_page(raw: Vec<Value>) -> Result<Vec<GithubIssue>, GithubError> {
    let mut issues = Vec::new();
    for item in raw {
        if item
            .as_object()
            .is_some_and(|obj| obj.contains_key("pull_request"))
        {
            continue;
        }
        issues.push(normalize_issue(&item)?);
    }
    Ok(issues)
}

fn pulls_from_page(raw: Vec<Value>) -> Result<Vec<GithubPull>, GithubError> {
    raw.iter().map(normalize_pull).collect()
}

fn normalize_issue(value: &Value) -> Result<GithubIssue, GithubError> {
    Ok(GithubIssue {
        number: required_u64(value, "number")?,
        title: required_str(value, "title")?,
        html_url: required_str(value, "html_url")?,
        updated_at: required_str(value, "updated_at")?,
        author_login: nested_required_str(value, &["user", "login"])?,
        labels: parse_labels(value),
        comments_count: value.get("comments").and_then(|v| v.as_u64()).unwrap_or(0),
        milestone_due: parse_milestone_due(value),
        assignee_logins: parse_assignee_logins(value),
    })
}

fn normalize_pull(value: &Value) -> Result<GithubPull, GithubError> {
    Ok(GithubPull {
        number: required_u64(value, "number")?,
        title: required_str(value, "title")?,
        html_url: required_str(value, "html_url")?,
        updated_at: required_str(value, "updated_at")?,
        author_login: nested_required_str(value, &["user", "login"])?,
        labels: parse_labels(value),
        comments_count: value.get("comments").and_then(|v| v.as_u64()).unwrap_or(0),
        milestone_due: parse_milestone_due(value),
        draft: value
            .get("draft")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        requested_reviewer_logins: parse_requested_reviewer_logins(value),
        head_sha: nested_required_str(value, &["head", "sha"])?,
    })
}

fn parse_labels(value: &Value) -> Vec<GithubLabel> {
    value
        .get("labels")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|label| {
                    let name = label.get("name")?.as_str()?.to_string();
                    let color = label
                        .get("color")
                        .and_then(|c| c.as_str())
                        .map(|s| s.to_string());
                    Some(GithubLabel { name, color })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_milestone_due(value: &Value) -> Option<String> {
    let due_on = value
        .get("milestone")
        .and_then(|m| if m.is_null() { None } else { m.get("due_on") })
        .and_then(|d| d.as_str())?;
    if due_on.len() >= 10 {
        Some(due_on[..10].to_string())
    } else {
        Some(due_on.to_string())
    }
}

fn parse_assignee_logins(value: &Value) -> Vec<String> {
    value
        .get("assignees")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("login").and_then(|l| l.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_requested_reviewer_logins(value: &Value) -> Vec<String> {
    value
        .get("requested_reviewers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|reviewer| {
                    if reviewer
                        .as_object()
                        .is_some_and(|obj| obj.contains_key("slug"))
                    {
                        return None;
                    }
                    reviewer
                        .get("login")
                        .and_then(|l| l.as_str())
                        .map(str::to_string)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn required_str(value: &Value, key: &str) -> Result<String, GithubError> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| GithubError::Network(format!("missing or invalid `{key}`")))
}

fn required_u64(value: &Value, key: &str) -> Result<u64, GithubError> {
    value
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| GithubError::Network(format!("missing or invalid `{key}`")))
}

fn nested_required_str(value: &Value, path: &[&str]) -> Result<String, GithubError> {
    let mut current = value;
    for key in path {
        current = current
            .get(key)
            .ok_or_else(|| GithubError::Network(format!("missing `{key}`")))?;
    }
    current
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| GithubError::Network(format!("invalid value at {}", path.join("."))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ISSUES_PAGE_FIXTURE: &str = r#"[
  {
    "number": 42,
    "title": "Real issue",
    "html_url": "https://github.com/o/r/issues/42",
    "updated_at": "2026-03-01T12:00:00Z",
    "user": { "login": "alice" },
    "labels": [
      { "name": "bug", "color": "ff0000" },
      { "name": "triage", "color": null }
    ],
    "comments": 7,
    "milestone": { "due_on": "2026-06-01T00:00:00Z" },
    "assignees": [{ "login": "bob" }, { "login": "carol" }]
  },
  {
    "number": 99,
    "title": "Looks like issue but is PR",
    "pull_request": { "url": "https://api.github.com/repos/o/r/pulls/99" }
  }
]"#;

    #[test]
    fn issues_page_drops_pull_request_entries_and_normalizes_issue() {
        let raw: Vec<Value> = serde_json::from_str(ISSUES_PAGE_FIXTURE).expect("fixture");
        let issues = issues_from_page(raw).expect("normalize");

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Real issue");
        assert_eq!(issue.html_url, "https://github.com/o/r/issues/42");
        assert_eq!(issue.updated_at, "2026-03-01T12:00:00Z");
        assert_eq!(issue.author_login, "alice");
        assert_eq!(issue.comments_count, 7);
        assert_eq!(issue.milestone_due.as_deref(), Some("2026-06-01"));
        assert_eq!(issue.assignee_logins, vec!["bob", "carol"]);
        assert_eq!(issue.labels.len(), 2);
        assert_eq!(issue.labels[0].name, "bug");
        assert_eq!(issue.labels[0].color.as_deref(), Some("ff0000"));
        assert_eq!(issue.labels[1].name, "triage");
        assert_eq!(issue.labels[1].color, None);
    }

    #[test]
    fn pull_requested_reviewers_skip_team_with_slug() {
        let raw = json!({
            "number": 10,
            "title": "Feature",
            "html_url": "https://github.com/o/r/pull/10",
            "updated_at": "2026-04-01T00:00:00Z",
            "user": { "login": "dev" },
            "labels": [],
            "comments": 0,
            "milestone": null,
            "draft": false,
            "requested_reviewers": [
                { "login": "reviewer1" },
                { "slug": "org-team", "name": "Org Team" }
            ],
            "head": { "sha": "deadbeef" }
        });
        let pull = normalize_pull(&raw).expect("pull");
        assert_eq!(pull.requested_reviewer_logins, vec!["reviewer1"]);
        assert_eq!(pull.head_sha, "deadbeef");
    }

    #[test]
    fn login_from_user_body_extracts_login() {
        let body: Value = serde_json::from_str(r#"{"login":"octocat"}"#).expect("json");
        assert_eq!(login_from_user_body(&body).expect("login"), "octocat");
    }
}
