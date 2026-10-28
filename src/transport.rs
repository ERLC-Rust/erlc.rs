//! HTTP transport: builds requests, applies authentication and maps responses.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use reqwest::header::{self, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Method, StatusCode, Url};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::auth::Secret;
use crate::error::{
    ApiError, DecodeError, Error, RateLimited, Result, TransportError, seconds_to_duration,
};
use crate::rate_limit::RateLimits;

pub(crate) const DEFAULT_BASE_URL: &str = "https://api.erlc.gg";
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) const USER_AGENT: &str = concat!(
    "erlc-rs/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/ERLC-Rust/erlc.rs)"
);

const SERVER_KEY_HEADER: &str = "server-key";
const BUCKET_HEADER: &str = "x-ratelimit-bucket";

/// Settings and state shared by every client built from the same builder.
pub(crate) struct Transport {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: Url,
    pub(crate) global_key: Option<Secret>,
    pub(crate) user_agent: HeaderValue,
    pub(crate) timeout: Duration,
    pub(crate) limits: RateLimits,
}

/// The private server a request is for.
#[derive(Clone)]
pub(crate) struct ServerContext {
    key: Secret,
    /// Identifies this server key in the rate-limit state.
    pub(crate) scope: u64,
}

impl ServerContext {
    pub(crate) fn new(key: Secret) -> Self {
        static NEXT_SCOPE: AtomicU64 = AtomicU64::new(0);
        Self {
            key,
            scope: NEXT_SCOPE.fetch_add(1, Ordering::Relaxed),
        }
    }
}

impl Transport {
    /// Sends a GET request and decodes a JSON response.
    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        server: &ServerContext,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        self.send(server, Method::GET, path, query, None).await
    }

    /// Sends a JSON POST once, never retried.
    pub(crate) async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        server: &ServerContext,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let body = serde_json::to_vec(body).expect("request bodies always serialize to JSON");
        self.send(server, Method::POST, path, &[], Some(body)).await
    }

    async fn send<T: DeserializeOwned>(
        &self,
        server: &ServerContext,
        method: Method,
        path: &str,
        query: &[(&str, &str)],
        body: Option<Vec<u8>>,
    ) -> Result<T> {
        let route = format!("{method} {path}");
        if let Some(blocked) = self.limits.check(server.scope, &route) {
            return Err(Error::RateLimited(RateLimited::local(
                blocked.bucket,
                blocked.wait,
            )));
        }

        let mut request = self
            .http
            .request(method, self.url(path, query))
            .header(
                HeaderName::from_static(SERVER_KEY_HEADER),
                server.key.header_value().clone(),
            )
            .header(header::ACCEPT, HeaderValue::from_static("application/json"))
            .header(header::USER_AGENT, self.user_agent.clone())
            .timeout(self.timeout);
        if let Some(global_key) = &self.global_key {
            request = request.header(header::AUTHORIZATION, global_key.header_value().clone());
        }
        if let Some(body) = body {
            request = request
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                )
                .body(body);
        }
        let response = request.send().await.map_err(TransportError::new)?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await.map_err(TransportError::new)?;
        let bucket = header_str(&headers, BUCKET_HEADER);

        if status == StatusCode::TOO_MANY_REQUESTS {
            let limited = RateLimited::new(retry_after_header(&headers), bucket, &body);
            self.limits
                .record(server.scope, &route, &headers, Some(limited.retry_after()));
            return Err(Error::RateLimited(limited));
        }
        self.limits.record(server.scope, &route, &headers, None);

        if status.is_success() {
            return serde_json::from_slice(&body)
                .map_err(|source| DecodeError::new(status.as_u16(), source, &body).into());
        }
        Err(ApiError::new(status.as_u16(), bucket, &body).into())
    }

    /// Appends `path` to the base URL (keeping any base path prefix) and adds the query.
    fn url(&self, path: &str, query: &[(&str, &str)]) -> Url {
        let mut url = self.base_url.clone();
        let full_path = format!("{}{}", url.path().trim_end_matches('/'), path);
        url.set_path(&full_path);
        if !query.is_empty() {
            url.query_pairs_mut().extend_pairs(query);
        }
        url
    }
}

/// Validates a user-supplied base URL.
pub(crate) fn parse_base_url(raw: &str) -> Result<Url> {
    const INVALID: &str =
        "base URL must be an absolute http or https URL without a query or fragment";
    let url = Url::parse(raw).map_err(|_| Error::InvalidInput(INVALID))?;
    let valid = matches!(url.scheme(), "http" | "https")
        && url.has_host()
        && url.query().is_none()
        && url.fragment().is_none();
    if valid {
        Ok(url)
    } else {
        Err(Error::InvalidInput(INVALID))
    }
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(name)?.to_str().ok().map(str::to_owned)
}

fn retry_after_header(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(header::RETRY_AFTER)?.to_str().ok()?;
    seconds_to_duration(value.trim().parse().ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transport(base: &str) -> Transport {
        Transport {
            http: reqwest::Client::new(),
            base_url: parse_base_url(base).unwrap(),
            global_key: None,
            user_agent: HeaderValue::from_static(USER_AGENT),
            timeout: DEFAULT_TIMEOUT,
            limits: RateLimits::default(),
        }
    }

    #[test]
    fn builds_urls_with_and_without_query() {
        let t = transport("https://api.erlc.gg");
        assert_eq!(
            t.url("/v2/server", &[]).as_str(),
            "https://api.erlc.gg/v2/server"
        );
        assert_eq!(
            t.url("/v2/server", &[("Players", "true"), ("Staff", "true")])
                .as_str(),
            "https://api.erlc.gg/v2/server?Players=true&Staff=true"
        );
    }

    #[test]
    fn keeps_base_path_prefix() {
        let t = transport("http://127.0.0.1:8080/proxy/erlc/");
        assert_eq!(
            t.url("/v2/server", &[]).as_str(),
            "http://127.0.0.1:8080/proxy/erlc/v2/server"
        );
    }

    #[test]
    fn rejects_invalid_base_urls() {
        for bad in [
            "api.erlc.gg",
            "ftp://api.erlc.gg",
            "https://api.erlc.gg/?a=1",
            "not a url",
        ] {
            assert!(
                matches!(parse_base_url(bad), Err(Error::InvalidInput(_))),
                "{bad}"
            );
        }
    }

    #[test]
    fn parses_retry_after_seconds() {
        let mut headers = HeaderMap::new();
        headers.insert(header::RETRY_AFTER, HeaderValue::from_static("5"));
        assert_eq!(retry_after_header(&headers), Some(Duration::from_secs(5)));
        headers.insert(header::RETRY_AFTER, HeaderValue::from_static("1.5"));
        assert_eq!(
            retry_after_header(&headers),
            Some(Duration::from_millis(1500))
        );
        headers.insert(header::RETRY_AFTER, HeaderValue::from_static("soon"));
        assert_eq!(retry_after_header(&headers), None);
    }

    #[test]
    fn server_contexts_get_distinct_scopes() {
        let key = Secret::server_key("key".to_owned()).unwrap();
        let a = ServerContext::new(key.clone());
        let b = ServerContext::new(key);
        assert_ne!(a.scope, b.scope);
    }
}
