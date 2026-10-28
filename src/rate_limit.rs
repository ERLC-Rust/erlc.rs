//! Client-side rate-limit tracking, so blocked requests fail locally.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use reqwest::header::HeaderMap;

use crate::error::seconds_to_duration;

/// Buckets shared by every server key.
const APP_WIDE: &[&str] = &["global", "unauthenticated-global", "invalid-requests"];

/// Buckets covering all requests for one server key.
const SERVER_WIDE: &[&str] = &["unauthenticated-server", "invalid-requests-server"];

/// Buckets known before any response has been seen, keyed by route.
const KNOWN_ROUTES: &[(&str, &str)] = &[("POST /v2/server/command", "command")];

/// Longest wait derived from `X-RateLimit-Reset`, to limit the effect of clock skew.
const MAX_RESET_WAIT: Duration = Duration::from_secs(300);

/// Wait applied after a 429 that carried no `Retry-After` value.
const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(5);

const BUCKET_HEADER: &str = "x-ratelimit-bucket";
const LIMIT_HEADER: &str = "x-ratelimit-limit";
const REMAINING_HEADER: &str = "x-ratelimit-remaining";
const RESET_HEADER: &str = "x-ratelimit-reset";

/// The last rate-limit headers seen for a bucket.
///
/// Returned by [`Client::rate_limit`](crate::Client::rate_limit).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RateLimitInfo {
    /// The bucket name, such as `global` or `command`.
    pub bucket: String,
    /// Requests allowed in the current window (`X-RateLimit-Limit`).
    pub limit: Option<u32>,
    /// Requests left in the current window (`X-RateLimit-Remaining`).
    pub remaining: Option<u32>,
    /// When the current window resets (`X-RateLimit-Reset`).
    pub reset: Option<SystemTime>,
}

/// Identifies a bucket: shared by the whole application, or scoped to one server key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Key {
    App(String),
    Server(u64, String),
}

impl Key {
    fn new(scope: u64, bucket: &str) -> Self {
        if APP_WIDE.contains(&bucket) {
            Self::App(bucket.to_owned())
        } else {
            Self::Server(scope, bucket.to_owned())
        }
    }
}

#[derive(Debug, Default)]
struct BucketState {
    info: Option<RateLimitInfo>,
    blocked_until: Option<Instant>,
}

impl BucketState {
    fn block_for(&mut self, wait: Duration) {
        let until = Instant::now() + wait;
        self.blocked_until = Some(
            self.blocked_until
                .map_or(until, |current| current.max(until)),
        );
    }
}

#[derive(Debug, Default)]
struct State {
    buckets: HashMap<Key, BucketState>,
    routes: HashMap<String, String>,
}

impl State {
    fn route_bucket(&self, route: &str) -> Option<&str> {
        self.routes.get(route).map(String::as_str).or_else(|| {
            KNOWN_ROUTES
                .iter()
                .find(|(known, _)| *known == route)
                .map(|(_, bucket)| *bucket)
        })
    }
}

/// A request that must not be sent yet.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Blocked {
    pub(crate) bucket: String,
    pub(crate) wait: Duration,
}

/// Rate-limit state shared by every [`Client`](crate::Client) built from the same builder.
#[derive(Debug, Default)]
pub(crate) struct RateLimits {
    state: Mutex<State>,
}

impl RateLimits {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Returns the longest active block affecting a request on `route` for server `scope`.
    pub(crate) fn check(&self, scope: u64, route: &str) -> Option<Blocked> {
        let state = self.lock();
        let now = Instant::now();
        let route_bucket = state.route_bucket(route);
        APP_WIDE
            .iter()
            .chain(SERVER_WIDE)
            .copied()
            .chain(route_bucket)
            .filter_map(|bucket| {
                let until = state.buckets.get(&Key::new(scope, bucket))?.blocked_until?;
                let wait = until.checked_duration_since(now).filter(|w| !w.is_zero())?;
                Some(Blocked {
                    bucket: bucket.to_owned(),
                    wait,
                })
            })
            .max_by_key(|blocked| blocked.wait)
    }

    /// Records a response's headers; `rate_limited` is `Some` for a 429.
    pub(crate) fn record(
        &self,
        scope: u64,
        route: &str,
        headers: &HeaderMap,
        rate_limited: Option<Option<Duration>>,
    ) {
        let mut state = self.lock();
        let header_bucket = header_str(headers, BUCKET_HEADER);
        if let Some(bucket) = &header_bucket {
            state.routes.insert(route.to_owned(), bucket.clone());
            let info = RateLimitInfo {
                bucket: bucket.clone(),
                limit: header_parse(headers, LIMIT_HEADER),
                remaining: header_parse(headers, REMAINING_HEADER),
                reset: header_parse::<f64>(headers, RESET_HEADER)
                    .and_then(seconds_to_duration)
                    .map(|since_epoch| UNIX_EPOCH + since_epoch),
            };
            let exhausted_for = (info.remaining == Some(0))
                .then(|| info.reset?.duration_since(SystemTime::now()).ok())
                .flatten()
                .filter(|wait| *wait <= MAX_RESET_WAIT);
            let entry = state.buckets.entry(Key::new(scope, bucket)).or_default();
            entry.info = Some(info);
            if let Some(wait) = exhausted_for {
                entry.block_for(wait);
            }
        }

        if let Some(retry_after) = rate_limited {
            let bucket = header_bucket
                .or_else(|| state.route_bucket(route).map(str::to_owned))
                .unwrap_or_else(|| route.to_owned());
            state
                .routes
                .entry(route.to_owned())
                .or_insert_with(|| bucket.clone());
            state
                .buckets
                .entry(Key::new(scope, &bucket))
                .or_default()
                .block_for(retry_after.unwrap_or(DEFAULT_RETRY_AFTER));
        }
    }

    /// The last headers seen for `bucket`, from the server's own scope or the app-wide scope.
    pub(crate) fn info(&self, scope: u64, bucket: &str) -> Option<RateLimitInfo> {
        self.lock()
            .buckets
            .get(&Key::new(scope, bucket))?
            .info
            .clone()
    }
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(name)?.to_str().ok().map(str::to_owned)
}

fn header_parse<T: std::str::FromStr>(headers: &HeaderMap, name: &str) -> Option<T> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    const ROUTE: &str = "GET /v2/server";
    const COMMAND: &str = "POST /v2/server/command";

    fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for (name, value) in pairs {
            map.insert(*name, HeaderValue::from_str(value).unwrap());
        }
        map
    }

    #[test]
    fn nothing_is_blocked_initially() {
        let limits = RateLimits::default();
        assert_eq!(limits.check(1, ROUTE), None);
        assert_eq!(limits.check(1, COMMAND), None);
    }

    #[test]
    fn records_header_info() {
        let limits = RateLimits::default();
        let reset = SystemTime::now() + Duration::from_secs(30);
        let reset_secs = reset
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        limits.record(
            1,
            ROUTE,
            &headers(&[
                (BUCKET_HEADER, "global"),
                (LIMIT_HEADER, "35"),
                (REMAINING_HEADER, "34"),
                (RESET_HEADER, &reset_secs),
            ]),
            None,
        );
        let info = limits.info(1, "global").unwrap();
        assert_eq!(info.limit, Some(35));
        assert_eq!(info.remaining, Some(34));
        assert!(info.reset.is_some());
        assert_eq!(limits.info(2, "global"), Some(info), "global is app-wide");
        assert_eq!(limits.check(1, ROUTE), None);
    }

    #[test]
    fn rate_limited_route_is_blocked_for_retry_after() {
        let limits = RateLimits::default();
        limits.record(
            1,
            ROUTE,
            &headers(&[(BUCKET_HEADER, "global")]),
            Some(Some(Duration::from_secs(10))),
        );
        let blocked = limits.check(1, ROUTE).unwrap();
        assert_eq!(blocked.bucket, "global");
        assert!(blocked.wait > Duration::from_secs(9));
        assert!(
            limits.check(2, COMMAND).is_some(),
            "global blocks every server and route"
        );
    }

    #[test]
    fn command_bucket_is_scoped_to_the_server_and_route() {
        let limits = RateLimits::default();
        limits.record(
            1,
            COMMAND,
            &headers(&[(BUCKET_HEADER, "command")]),
            Some(Some(Duration::from_secs(5))),
        );
        assert!(limits.check(1, COMMAND).is_some());
        assert_eq!(limits.check(1, ROUTE), None);
        assert_eq!(limits.check(2, COMMAND), None);
    }

    #[test]
    fn known_route_bucket_applies_without_headers() {
        let limits = RateLimits::default();
        limits.record(1, COMMAND, &HeaderMap::new(), Some(None));
        assert_eq!(limits.check(1, COMMAND).unwrap().bucket, "command");
    }

    #[test]
    fn unknown_bucket_falls_back_to_route() {
        let limits = RateLimits::default();
        limits.record(
            1,
            ROUTE,
            &HeaderMap::new(),
            Some(Some(Duration::from_secs(5))),
        );
        assert_eq!(limits.check(1, ROUTE).unwrap().bucket, ROUTE);
        assert_eq!(limits.check(1, COMMAND), None);
    }

    #[test]
    fn exhausted_bucket_blocks_until_reset() {
        let limits = RateLimits::default();
        let reset = SystemTime::now() + Duration::from_secs(20);
        let reset_secs = reset
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        limits.record(
            1,
            ROUTE,
            &headers(&[
                (BUCKET_HEADER, "global"),
                (REMAINING_HEADER, "0"),
                (RESET_HEADER, &reset_secs),
            ]),
            None,
        );
        assert!(limits.check(1, ROUTE).unwrap().wait > Duration::from_secs(15));
    }

    #[test]
    fn far_future_reset_is_ignored() {
        let limits = RateLimits::default();
        let reset = SystemTime::now() + Duration::from_secs(3600);
        let reset_secs = reset
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        limits.record(
            1,
            ROUTE,
            &headers(&[
                (BUCKET_HEADER, "global"),
                (REMAINING_HEADER, "0"),
                (RESET_HEADER, &reset_secs),
            ]),
            None,
        );
        assert_eq!(limits.check(1, ROUTE), None);
    }

    #[test]
    fn invalid_requests_server_block_is_scoped() {
        let limits = RateLimits::default();
        limits.record(
            1,
            ROUTE,
            &headers(&[(BUCKET_HEADER, "invalid-requests-server")]),
            Some(Some(Duration::from_secs(60))),
        );
        assert!(limits.check(1, COMMAND).is_some());
        assert_eq!(limits.check(2, ROUTE), None);
    }
}
