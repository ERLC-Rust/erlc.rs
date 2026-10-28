//! Error types returned by the client.

use std::error::Error as StdError;
use std::fmt;
use std::time::Duration;

use serde_json::Value;

/// Result type used throughout this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Maximum number of response-body bytes kept on an error.
const MAX_BODY_LEN: usize = 4096;

/// Maximum length of a non-JSON error body used as the error message.
const MAX_TEXT_MESSAGE_LEN: usize = 512;

/// Everything that can go wrong when calling the ER:LC API.
///
/// Error messages never contain server keys or global API keys.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The API answered with an unsuccessful status other than 429.
    #[error(transparent)]
    Api(Box<ApiError>),

    /// The API rejected the request because a rate limit was exceeded.
    ///
    /// Stop sending requests and wait for [`RateLimited::retry_after`] before trying
    /// again. Ignoring this can get your key temporarily blocked.
    #[error(transparent)]
    RateLimited(RateLimited),

    /// No HTTP response was received: connection failure, timeout or TLS error.
    #[error(transparent)]
    Transport(#[from] TransportError),

    /// The API answered successfully, but the body did not have the expected shape.
    #[error(transparent)]
    Decode(Box<DecodeError>),

    /// The client was given invalid input; no request was sent.
    #[error("invalid input: {0}")]
    InvalidInput(&'static str),
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Self {
        Self::Api(Box::new(err))
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self {
        Self::Decode(Box::new(err))
    }
}

/// An error code documented by the ER:LC API.
///
/// See <https://apidocs.erlc.gg/error-codes>.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorCode {
    /// `0`: an unknown error occurred.
    UnknownError,
    /// `1001`: an error occurred communicating with Roblox or the in-game private server.
    RobloxCommunication,
    /// `1002`: an internal system error occurred.
    InternalError,
    /// `2000`: no server key was provided.
    MissingServerKey,
    /// `2001`: the server key is incorrectly formatted.
    MalformedServerKey,
    /// `2002`: the server key is invalid or expired.
    InvalidServerKey,
    /// `2003`: the global API key is invalid.
    InvalidGlobalKey,
    /// `2004`: the server key is banned from accessing the API.
    BannedServerKey,
    /// `2005`: unauthenticated requests are being made on behalf of multiple servers.
    ///
    /// Register a public application and send its global API key.
    UnauthenticatedMultiServer,
    /// `3001`: the request body did not contain a valid command.
    InvalidCommand,
    /// `3002`: the private server is offline (it has no players).
    ServerOffline,
    /// `4000`: you are not authorized to perform this action on this server.
    Unauthorized,
    /// `4001`: you are being rate limited or are temporarily blocked.
    RateLimited,
    /// `4002`: the command is restricted.
    RestrictedCommand,
    /// `4003`: the message is prohibited.
    ProhibitedMessage,
    /// `9998`: the resource is restricted.
    RestrictedResource,
    /// `9999`: the module running on the in-game server is out of date.
    ///
    /// Kick all players from the private server and retry.
    OutdatedModule,
    /// A code not documented when this crate was released.
    Other(i64),
}

impl ErrorCode {
    /// Maps a numeric code from an API response.
    pub fn from_code(code: i64) -> Self {
        match code {
            0 => Self::UnknownError,
            1001 => Self::RobloxCommunication,
            1002 => Self::InternalError,
            2000 => Self::MissingServerKey,
            2001 => Self::MalformedServerKey,
            2002 => Self::InvalidServerKey,
            2003 => Self::InvalidGlobalKey,
            2004 => Self::BannedServerKey,
            2005 => Self::UnauthenticatedMultiServer,
            3001 => Self::InvalidCommand,
            3002 => Self::ServerOffline,
            4000 => Self::Unauthorized,
            4001 => Self::RateLimited,
            4002 => Self::RestrictedCommand,
            4003 => Self::ProhibitedMessage,
            9998 => Self::RestrictedResource,
            9999 => Self::OutdatedModule,
            other => Self::Other(other),
        }
    }

    /// The numeric code as sent by the API.
    pub fn as_i64(self) -> i64 {
        match self {
            Self::UnknownError => 0,
            Self::RobloxCommunication => 1001,
            Self::InternalError => 1002,
            Self::MissingServerKey => 2000,
            Self::MalformedServerKey => 2001,
            Self::InvalidServerKey => 2002,
            Self::InvalidGlobalKey => 2003,
            Self::BannedServerKey => 2004,
            Self::UnauthenticatedMultiServer => 2005,
            Self::InvalidCommand => 3001,
            Self::ServerOffline => 3002,
            Self::Unauthorized => 4000,
            Self::RateLimited => 4001,
            Self::RestrictedCommand => 4002,
            Self::ProhibitedMessage => 4003,
            Self::RestrictedResource => 9998,
            Self::OutdatedModule => 9999,
            Self::Other(code) => code,
        }
    }

    /// A short description of the code, taken from the ER:LC documentation.
    pub fn description(self) -> &'static str {
        match self {
            Self::UnknownError => "an unknown error occurred",
            Self::RobloxCommunication => {
                "an error occurred communicating with Roblox or the in-game private server"
            }
            Self::InternalError => "an internal system error occurred",
            Self::MissingServerKey => "no server key was provided",
            Self::MalformedServerKey => "the server key is incorrectly formatted",
            Self::InvalidServerKey => "the server key is invalid or expired",
            Self::InvalidGlobalKey => "the global API key is invalid",
            Self::BannedServerKey => "the server key is banned from accessing the API",
            Self::UnauthenticatedMultiServer => {
                "unauthenticated requests are being made on behalf of multiple servers"
            }
            Self::InvalidCommand => "the request did not contain a valid command",
            Self::ServerOffline => "the private server is offline (it has no players)",
            Self::Unauthorized => "not authorized to perform this action on this server",
            Self::RateLimited => "rate limited or temporarily blocked",
            Self::RestrictedCommand => "the command is restricted",
            Self::ProhibitedMessage => "the message is prohibited",
            Self::RestrictedResource => "the resource is restricted",
            Self::OutdatedModule => "the in-game server module is out of date",
            Self::Other(_) => "undocumented error code",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.as_i64(), self.description())
    }
}

/// An unsuccessful response from the ER:LC API.
#[derive(Debug, Clone)]
pub struct ApiError {
    status: u16,
    code: Option<i64>,
    message: Option<String>,
    command_id: Option<String>,
    bucket: Option<String>,
    body: String,
}

impl ApiError {
    pub(crate) fn new(status: u16, bucket: Option<String>, body: &[u8]) -> Self {
        let parsed = ErrorBody::parse(body);
        Self {
            status,
            code: parsed.code,
            message: parsed.message,
            command_id: parsed.command_id,
            bucket,
            body: truncate_body(body),
        }
    }

    /// The HTTP status code.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// The ER:LC error code, if the response contained one.
    pub fn code(&self) -> Option<ErrorCode> {
        self.code.map(ErrorCode::from_code)
    }

    /// The raw numeric error code, if the response contained one.
    pub fn raw_code(&self) -> Option<i64> {
        self.code
    }

    /// The error message from the response body, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// The command execution ID, returned by some command errors.
    ///
    /// Only useful when reporting a problem to the ER:LC API team.
    pub fn command_id(&self) -> Option<&str> {
        self.command_id.as_deref()
    }

    /// The rate-limit bucket reported in the `X-RateLimit-Bucket` header, if any.
    pub fn bucket(&self) -> Option<&str> {
        self.bucket.as_deref()
    }

    /// The response body, truncated to 4 KiB.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Whether the key was rejected or lacks permission (codes 2000-2005 and 4000, or HTTP 401/403).
    ///
    /// The ER:LC docs advise to stop using a server key that keeps returning 403:
    /// it has probably been regenerated by the server owner.
    pub fn is_auth(&self) -> bool {
        matches!(self.code, Some(2000..=2005 | 4000)) || matches!(self.status, 401 | 403)
    }

    /// Whether the private server is offline (code 3002, or HTTP 422).
    pub fn is_server_offline(&self) -> bool {
        self.code == Some(3002) || self.status == 422
    }

    /// Whether the in-game server module is out of date (code 9999).
    pub fn is_module_outdated(&self) -> bool {
        self.code == Some(9999)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ER:LC API returned HTTP {}", self.status)?;
        if let Some(code) = self.code {
            write!(f, " with code {code}")?;
        }
        match (&self.message, self.code()) {
            (Some(message), _) => write!(f, ": {message}"),
            (None, Some(code)) if !matches!(code, ErrorCode::Other(_)) => {
                write!(f, ": {}", code.description())
            }
            _ => Ok(()),
        }
    }
}

impl StdError for ApiError {}

/// The API rejected a request because a rate limit was exceeded (HTTP 429).
#[derive(Debug, Clone)]
pub struct RateLimited {
    retry_after: Option<Duration>,
    bucket: Option<String>,
    message: Option<String>,
    local: bool,
}

impl RateLimited {
    pub(crate) fn new(
        retry_after_header: Option<Duration>,
        bucket: Option<String>,
        body: &[u8],
    ) -> Self {
        let parsed = ErrorBody::parse(body);
        Self {
            retry_after: retry_after_header.or(parsed.retry_after),
            bucket,
            message: parsed.message,
            local: false,
        }
    }

    /// A request the client refused to send because a bucket is still cooling down.
    pub(crate) fn local(bucket: String, wait: Duration) -> Self {
        Self {
            retry_after: Some(wait),
            bucket: Some(bucket),
            message: None,
            local: true,
        }
    }

    /// Whether the client stopped the request itself, without sending it, because an
    /// earlier response put this bucket on hold.
    pub fn is_local(&self) -> bool {
        self.local
    }

    /// How long to wait before sending another request, from the `Retry-After`
    /// header or the `retry_after` body field.
    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }

    /// The rate-limit bucket that was exceeded, such as `global` or `command`.
    pub fn bucket(&self) -> Option<&str> {
        self.bucket.as_deref()
    }

    /// The message from the response body, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}

impl fmt::Display for RateLimited {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("rate limited by the ER:LC API")?;
        if let Some(bucket) = &self.bucket {
            write!(f, " (bucket `{bucket}`)")?;
        }
        if let Some(wait) = self.retry_after {
            write!(f, "; retry after {:.1}s", wait.as_secs_f64())?;
        }
        if self.local {
            f.write_str(" (request not sent)")?;
        }
        Ok(())
    }
}

impl StdError for RateLimited {}

/// No HTTP response was received from the API.
///
/// For commands, a timeout does not mean the command failed: it may still have run.
#[derive(Debug)]
pub struct TransportError {
    source: reqwest::Error,
}

impl TransportError {
    pub(crate) fn new(source: reqwest::Error) -> Self {
        Self { source }
    }

    /// Whether the request timed out.
    pub fn is_timeout(&self) -> bool {
        self.source.is_timeout()
    }

    /// Whether the connection could not be established.
    pub fn is_connect(&self) -> bool {
        self.source.is_connect()
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_timeout() {
            f.write_str("request to the ER:LC API timed out")
        } else if self.is_connect() {
            f.write_str("could not connect to the ER:LC API")
        } else {
            f.write_str("request to the ER:LC API failed")
        }
    }
}

impl StdError for TransportError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

/// A successful response whose body did not match the expected shape.
#[derive(Debug)]
pub struct DecodeError {
    status: u16,
    source: serde_json::Error,
    body: String,
}

impl DecodeError {
    pub(crate) fn new(status: u16, source: serde_json::Error, body: &[u8]) -> Self {
        Self {
            status,
            source,
            body: truncate_body(body),
        }
    }

    /// The HTTP status code.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// The response body, truncated to 4 KiB. It may contain server data.
    pub fn body(&self) -> &str {
        &self.body
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "could not decode ER:LC API response (HTTP {}) at line {} column {}",
            self.status,
            self.source.line(),
            self.source.column()
        )
    }
}

impl StdError for DecodeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.source)
    }
}

/// Leniently parsed error body; shapes vary between endpoints.
#[derive(Debug, Default, PartialEq)]
struct ErrorBody {
    code: Option<i64>,
    message: Option<String>,
    command_id: Option<String>,
    retry_after: Option<Duration>,
}

impl ErrorBody {
    fn parse(body: &[u8]) -> Self {
        let Ok(Value::Object(map)) = serde_json::from_slice::<Value>(body) else {
            return Self {
                message: text_message(body),
                ..Self::default()
            };
        };
        Self {
            code: map
                .get("code")
                .and_then(|v| v.as_i64().or_else(|| v.as_str()?.trim().parse().ok())),
            message: map
                .get("message")
                .or_else(|| map.get("error"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            command_id: map
                .get("commandId")
                .and_then(Value::as_str)
                .map(str::to_owned),
            retry_after: map
                .get("retry_after")
                .and_then(Value::as_f64)
                .and_then(seconds_to_duration),
        }
    }
}

/// Uses a short plain-text body as the message; ignores HTML and long bodies.
fn text_message(body: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(body);
    let text = text.trim();
    let usable = !text.is_empty() && text.len() <= MAX_TEXT_MESSAGE_LEN && !text.starts_with('<');
    usable.then(|| text.to_owned())
}

pub(crate) fn seconds_to_duration(secs: f64) -> Option<Duration> {
    Duration::try_from_secs_f64(secs)
        .ok()
        .filter(|d| !d.is_zero())
}

fn truncate_body(body: &[u8]) -> String {
    let text = String::from_utf8_lossy(body);
    if text.len() <= MAX_BODY_LEN {
        return text.into_owned();
    }
    let mut end = MAX_BODY_LEN;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_round_trip() {
        for code in [
            0, 1001, 1002, 2000, 2001, 2002, 2003, 2004, 2005, 3001, 3002, 4000, 4001, 4002, 4003,
            9998, 9999, 1234,
        ] {
            assert_eq!(ErrorCode::from_code(code).as_i64(), code);
        }
        assert_eq!(ErrorCode::from_code(1234), ErrorCode::Other(1234));
        assert_eq!(ErrorCode::from_code(3002), ErrorCode::ServerOffline);
    }

    #[test]
    fn parses_code_and_message_body() {
        let body =
            ErrorBody::parse(br#"{"code":2001,"message":"Duplicate query parameter: Players"}"#);
        assert_eq!(body.code, Some(2001));
        assert_eq!(
            body.message.as_deref(),
            Some("Duplicate query parameter: Players")
        );
    }

    #[test]
    fn parses_error_field_as_message() {
        let body = ErrorBody::parse(br#"{"error":"Error communicating with Roblox","code":1001}"#);
        assert_eq!(body.code, Some(1001));
        assert_eq!(
            body.message.as_deref(),
            Some("Error communicating with Roblox")
        );
    }

    #[test]
    fn parses_command_id_and_retry_after() {
        let body = ErrorBody::parse(
            br#"{"message":"offline","commandId":"ecfd9342","retry_after":2.5,"code":"4001"}"#,
        );
        assert_eq!(body.command_id.as_deref(), Some("ecfd9342"));
        assert_eq!(body.retry_after, Some(Duration::from_millis(2500)));
        assert_eq!(body.code, Some(4001));
    }

    #[test]
    fn plain_text_bodies_become_messages_but_html_does_not() {
        assert_eq!(
            ErrorBody::parse(b"Forbidden\n").message.as_deref(),
            Some("Forbidden")
        );
        assert_eq!(ErrorBody::parse(b"<html>blocked</html>").message, None);
        assert_eq!(ErrorBody::parse(b"").message, None);
    }

    #[test]
    fn invalid_retry_after_values_are_ignored() {
        assert_eq!(seconds_to_duration(-1.0), None);
        assert_eq!(seconds_to_duration(0.0), None);
        assert_eq!(seconds_to_duration(f64::NAN), None);
        assert_eq!(seconds_to_duration(5.0), Some(Duration::from_secs(5)));
    }

    #[test]
    fn truncates_long_bodies_on_char_boundaries() {
        let body = "é".repeat(MAX_BODY_LEN);
        let truncated = truncate_body(body.as_bytes());
        assert!(truncated.len() <= MAX_BODY_LEN + '…'.len_utf8());
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn api_error_display_prefers_api_message() {
        let err = ApiError::new(
            403,
            None,
            br#"{"code":2002,"message":"Invalid server-key"}"#,
        );
        assert_eq!(
            err.to_string(),
            "ER:LC API returned HTTP 403 with code 2002: Invalid server-key"
        );
        let err = ApiError::new(403, None, br#"{"code":2002}"#);
        assert_eq!(
            err.to_string(),
            "ER:LC API returned HTTP 403 with code 2002: the server key is invalid or expired"
        );
        let err = ApiError::new(500, None, b"");
        assert_eq!(err.to_string(), "ER:LC API returned HTTP 500");
    }
}
