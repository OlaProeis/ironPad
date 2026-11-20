use std::time::Duration;

use chrono::{Local, TimeZone};
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, ETAG, IF_NONE_MATCH};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;

static GITHUB_API_VERSION: Lazy<HeaderValue> = Lazy::new(|| HeaderValue::from_static("2022-11-28"));

const ACCEPT_GITHUB_JSON: &str = "application/vnd.github+json";
const USER_AGENT: &str = concat!("ironpad/", env!("CARGO_PKG_VERSION"));
const MAX_PAGINATION_PAGES: usize = 5;

const MSG_AUTH_NEEDED: &str =
    "GitHub needs a login. Run gh auth login, or set GH_TOKEN for this app. Ironpad does not store a token.";
const MSG_AUTH_REFUSED: &str =
    "GitHub refused the login. Run gh auth login again, or replace GH_TOKEN. A git SSH key is not a GitHub API login.";
const MSG_NOT_FOUND: &str = "That repository was not found, or this login cannot see it.";
const MSG_RATE_LIMIT_BASE: &str = "GitHub rate limit reached. The last snapshot is still shown.";
const MSG_NETWORK: &str = "Could not reach GitHub. The last snapshot is still shown.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GithubError {
    NotModified,
    AuthNeeded,
    AuthRefused,
    NotFound,
    RateLimit { reset_time: Option<String> },
    Network(String),
}

impl GithubError {
    pub fn user_message(&self) -> String {
        match self {
            GithubError::NotModified => MSG_NETWORK.to_string(),
            GithubError::AuthNeeded => MSG_AUTH_NEEDED.to_string(),
            GithubError::AuthRefused => MSG_AUTH_REFUSED.to_string(),
            GithubError::NotFound => MSG_NOT_FOUND.to_string(),
            GithubError::RateLimit { reset_time } => match reset_time {
                Some(when) => format!("{MSG_RATE_LIMIT_BASE} Limit resets at {when}."),
                None => MSG_RATE_LIMIT_BASE.to_string(),
            },
            GithubError::Network(_) => MSG_NETWORK.to_string(),
        }
    }
}

pub struct GithubClient {
    client: Client,
    token: String,
}

impl GithubClient {
    pub fn new(token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static(ACCEPT_GITHUB_JSON));
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {token}")).expect("bearer token header value");
        authorization.set_sensitive(true);
        headers.insert(AUTHORIZATION, authorization);
        headers.insert(
            reqwest::header::HeaderName::from_static("x-github-api-version"),
            GITHUB_API_VERSION.clone(),
        );

        let client = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client");

        Self {
            client,
            token: token.to_string(),
        }
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        url: &str,
        etag: Option<&str>,
    ) -> Result<(T, Option<String>), GithubError> {
        let mut request = self.client.get(url);
        if let Some(tag) = etag {
            request = request.header(IF_NONE_MATCH, tag);
        }

        let response = request
            .send()
            .await
            .map_err(|e| GithubError::Network(e.to_string()))?;

        let status = response.status();
        if status == StatusCode::NOT_MODIFIED {
            return Err(GithubError::NotModified);
        }

        if !status.is_success() {
            return Err(map_error_status(status, response.headers()));
        }

        let etag = header_etag(response.headers());
        let body = response
            .json::<T>()
            .await
            .map_err(|e| GithubError::Network(e.to_string()))?;

        Ok((body, etag))
    }

    pub async fn get_paginated<T: DeserializeOwned>(
        &self,
        url: &str,
        etag: Option<&str>,
    ) -> Result<(Vec<T>, Option<String>, bool), GithubError> {
        let mut current_url = url.to_string();
        let mut merged: Vec<T> = Vec::new();
        let mut last_etag: Option<String> = None;
        let mut truncated = false;

        for page in 0..MAX_PAGINATION_PAGES {
            let mut request = self.client.get(&current_url);
            if page == 0 {
                if let Some(tag) = etag {
                    request = request.header(IF_NONE_MATCH, tag);
                }
            }

            let response = request
                .send()
                .await
                .map_err(|e| GithubError::Network(e.to_string()))?;

            let status = response.status();
            if status == StatusCode::NOT_MODIFIED {
                if page == 0 {
                    return Err(GithubError::NotModified);
                }
                return Err(GithubError::Network(
                    "unexpected 304 during pagination".to_string(),
                ));
            }

            if !status.is_success() {
                return Err(map_error_status(status, response.headers()));
            }

            let headers = response.headers().clone();
            last_etag = header_etag(&headers);

            let page_items = response
                .json::<Vec<T>>()
                .await
                .map_err(|e| GithubError::Network(e.to_string()))?;
            merged.extend(page_items);

            if let Some(next) = link_rel_next(&headers) {
                if page + 1 >= MAX_PAGINATION_PAGES {
                    truncated = true;
                    break;
                }
                current_url = next;
            } else {
                break;
            }
        }

        Ok((merged, last_etag, truncated))
    }

    #[cfg(test)]
    fn token_for_test(&self) -> &str {
        &self.token
    }
}

fn map_error_status(status: StatusCode, headers: &HeaderMap) -> GithubError {
    if status == StatusCode::UNAUTHORIZED {
        return GithubError::AuthRefused;
    }
    if status == StatusCode::NOT_FOUND {
        return GithubError::NotFound;
    }
    if status == StatusCode::FORBIDDEN || status == StatusCode::TOO_MANY_REQUESTS {
        if is_rate_limited(headers) {
            return GithubError::RateLimit {
                reset_time: rate_limit_reset_local(headers),
            };
        }
        if status == StatusCode::FORBIDDEN {
            return GithubError::AuthRefused;
        }
        return GithubError::Network(format!("HTTP {status}"));
    }
    GithubError::Network(format!("HTTP {status}"))
}

fn is_rate_limited(headers: &HeaderMap) -> bool {
    if headers.contains_key("retry-after") {
        return true;
    }
    headers
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == "0")
}

fn rate_limit_reset_local(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get("x-ratelimit-reset")
        .and_then(|v| v.to_str().ok())?;
    let secs: i64 = raw.parse().ok()?;
    Local
        .timestamp_opt(secs, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M %Z").to_string())
}

fn header_etag(headers: &HeaderMap) -> Option<String> {
    headers
        .get(ETAG)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Parse `Link: <url>; rel="next", ...` and return the next URL if present.
fn link_rel_next(headers: &HeaderMap) -> Option<String> {
    let link = headers.get(reqwest::header::LINK)?.to_str().ok()?;
    for part in link.split(',') {
        let part = part.trim();
        if !part.contains("rel=\"next\"") && !part.contains("rel='next'") {
            continue;
        }
        let start = part.find('<')? + 1;
        let end = part.find('>')?;
        if end > start {
            return Some(part[start..end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Item {
        id: u32,
    }

    fn test_client() -> GithubClient {
        GithubClient::new("test-token-for-mock")
    }

    #[tokio::test]
    async fn get_paginated_follows_link_and_merges_pages() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let url_page1 = format!("{base}/page1");
        let url_page2 = format!("{base}/page2");
        let page1 = server
            .mock("GET", "/page1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_header("link", format!("<{url_page2}>; rel=\"next\"").as_str())
            .with_body(r#"[{"id":1}]"#)
            .create_async()
            .await;

        let page2 = server
            .mock("GET", "/page2")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[{"id":2}]"#)
            .create_async()
            .await;

        let client = test_client();
        let (items, _etag, truncated) = client
            .get_paginated::<Item>(&url_page1, None)
            .await
            .expect("paginated");

        page1.assert_async().await;
        page2.assert_async().await;

        assert_eq!(items, vec![Item { id: 1 }, Item { id: 2 }]);
        assert!(!truncated);
    }

    #[tokio::test]
    async fn get_json_304_returns_not_modified() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/resource")
            .match_header("if-none-match", "W/\"abc\"")
            .with_status(304)
            .create_async()
            .await;

        let client = test_client();
        let url = format!("{}/resource", server.url());
        let err = client
            .get_json::<Item>(&url, Some("W/\"abc\""))
            .await
            .expect_err("304");

        mock.assert_async().await;
        assert_eq!(err, GithubError::NotModified);
    }

    #[tokio::test]
    async fn get_json_sends_user_agent_and_bearer() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/resource")
            .match_header("user-agent", USER_AGENT)
            .match_header("authorization", "Bearer test-token-for-mock")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":7}"#)
            .create_async()
            .await;

        let client = test_client();
        let url = format!("{}/resource", server.url());
        let (item, _) = client.get_json::<Item>(&url, None).await.expect("ok");

        mock.assert_async().await;
        assert_eq!(item.id, 7);
    }

    #[tokio::test]
    async fn get_json_401_auth_refused_user_message() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/resource")
            .with_status(401)
            .create_async()
            .await;

        let client = test_client();
        let url = format!("{}/resource", server.url());
        let err = client.get_json::<Item>(&url, None).await.expect_err("401");

        mock.assert_async().await;
        assert_eq!(err, GithubError::AuthRefused);
        assert_eq!(err.user_message(), MSG_AUTH_REFUSED);
    }

    #[tokio::test]
    async fn get_json_403_rate_limit_when_remaining_zero() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/resource")
            .with_status(403)
            .with_header("x-ratelimit-remaining", "0")
            .with_header("x-ratelimit-reset", "1893456000")
            .create_async()
            .await;

        let client = test_client();
        let url = format!("{}/resource", server.url());
        let err = client
            .get_json::<Item>(&url, None)
            .await
            .expect_err("403 rate");

        mock.assert_async().await;
        assert!(matches!(err, GithubError::RateLimit { .. }));
        assert!(err.user_message().starts_with(MSG_RATE_LIMIT_BASE));
    }

    #[tokio::test]
    async fn get_json_404_not_found() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/resource")
            .with_status(404)
            .create_async()
            .await;

        let client = test_client();
        let url = format!("{}/resource", server.url());
        let err = client.get_json::<Item>(&url, None).await.expect_err("404");

        mock.assert_async().await;
        assert_eq!(err, GithubError::NotFound);
    }

    #[test]
    fn new_sets_github_headers_without_logging_token() {
        let client = GithubClient::new("secret-value");
        assert_eq!(client.token_for_test(), "secret-value");
    }
}
