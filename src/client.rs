//! The API client and its builder.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use reqwest::header::HeaderValue;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::auth::Secret;
use crate::error::{Error, Result, TransportError};
use crate::rate_limit::{RateLimitInfo, RateLimits};
use crate::transport::{self, ServerContext, Transport};

/// A client for the ER:LC Private Server API.
///
/// Cloning is cheap: clones share the same connection pool and rate-limit state.
///
/// ```no_run
/// # async fn run() -> erlc::Result<()> {
/// let client = erlc::Client::new(std::env::var("ERLC_SERVER_KEY").unwrap_or_default())?;
/// let info = client.server_info().await?;
/// println!("{}", info.name);
/// # Ok(())
/// # }
/// ```
///
/// # Rate limits
///
/// After the API answers 429, the client remembers the `Retry-After` delay. Until it
/// has passed, requests that would hit the same bucket fail immediately with
/// [`Error::RateLimited`] (where [`RateLimited::is_local`](crate::RateLimited::is_local)
/// is `true`) without being sent, as the ER:LC docs require. The same happens when a
/// bucket reports zero remaining requests. Requests are never retried automatically.
#[derive(Clone)]
pub struct Client {
    transport: Arc<Transport>,
    server: ServerContext,
}

impl Client {
    /// Creates a client for one private server with the default settings.
    ///
    /// Surrounding whitespace is trimmed from the key.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if the key is empty or contains characters that
    /// cannot be sent in an HTTP header, and [`Error::Transport`] if the HTTP client
    /// cannot be initialised.
    pub fn new(server_key: impl Into<String>) -> Result<Self> {
        Self::builder().server_key(server_key).build()
    }

    /// Returns a builder for configuring a client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Returns a client for another private server that shares this client's
    /// settings, connection pool, global API key and rate-limit state.
    ///
    /// This is how a public application serves many servers: build one client with
    /// [`ClientBuilder::global_key`], then call this once per server key.
    ///
    /// ```no_run
    /// # fn run(client: erlc::Client) -> erlc::Result<()> {
    /// let other_server = client.with_server_key(std::env::var("OTHER_SERVER_KEY").unwrap_or_default())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if the key is empty or not header-safe.
    pub fn with_server_key(&self, server_key: impl Into<String>) -> Result<Self> {
        Ok(Self {
            transport: Arc::clone(&self.transport),
            server: ServerContext::new(Secret::server_key(server_key.into())?),
        })
    }

    /// The last rate-limit headers the API sent for `bucket` (such as `"global"` or
    /// `"command"`), or `None` if no response for that bucket has been seen yet.
    pub fn rate_limit(&self, bucket: &str) -> Option<RateLimitInfo> {
        self.transport.limits.info(self.server.scope, bucket)
    }

    pub(crate) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        self.transport.get(&self.server, path, query).await
    }

    pub(crate) async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        self.transport.post_json(&self.server, path, body).await
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.transport.base_url.as_str())
            .field("global_key", &self.transport.global_key.is_some())
            .field("timeout", &self.transport.timeout)
            .finish_non_exhaustive()
    }
}

/// Configures and builds a [`Client`].
///
/// ```no_run
/// # fn run() -> erlc::Result<()> {
/// use std::time::Duration;
///
/// let client = erlc::Client::builder()
///     .server_key(std::env::var("ERLC_SERVER_KEY").unwrap_or_default())
///     .timeout(Duration::from_secs(15))
///     .user_agent("my-bot/1.0")
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
#[must_use]
pub struct ClientBuilder {
    server_key: Option<String>,
    global_key: Option<String>,
    base_url: Option<String>,
    user_agent: Option<String>,
    http_client: Option<reqwest::Client>,
    timeout: Duration,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            server_key: None,
            global_key: None,
            base_url: None,
            user_agent: None,
            http_client: None,
            timeout: transport::DEFAULT_TIMEOUT,
        }
    }
}

impl ClientBuilder {
    /// Sets the private server key, sent in the `Server-Key` header. Required.
    pub fn server_key(mut self, key: impl Into<String>) -> Self {
        self.server_key = Some(key.into());
        self
    }

    /// Sets a global API key, sent in the `Authorization` header.
    ///
    /// Only needed for public applications that serve more than one server; register
    /// one at <https://apidocs.erlc.gg/creating-public-applications>. Requests made
    /// for several servers without a global key are refused with code `2005`.
    pub fn global_key(mut self, key: impl Into<String>) -> Self {
        self.global_key = Some(key.into());
        self
    }

    /// Overrides the API base URL. Defaults to `https://api.erlc.gg`.
    ///
    /// Useful for tests and proxies. A path prefix is kept.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Identifies your application in the `User-Agent` header, for example
    /// `"my-bot/1.0"`. The crate's own identifier is appended.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Uses your own [`reqwest::Client`], for example to share a connection pool or
    /// configure a proxy. The timeout and headers set by this builder still apply.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Sets the timeout for each request, including reading the body. Defaults to 10 seconds.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Builds the client.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidInput`] if the server key is missing or invalid, the global
    /// key, user agent or base URL is invalid, or the timeout is zero, and
    /// [`Error::Transport`] if the default HTTP client cannot be initialised.
    pub fn build(self) -> Result<Client> {
        let server_key = self
            .server_key
            .ok_or(Error::InvalidInput("a server key is required"))?;
        let server_key = Secret::server_key(server_key)?;
        let global_key = self.global_key.map(Secret::global_key).transpose()?;
        let base_url = transport::parse_base_url(
            self.base_url
                .as_deref()
                .unwrap_or(transport::DEFAULT_BASE_URL),
        )?;
        if self.timeout.is_zero() {
            return Err(Error::InvalidInput("timeout must be greater than zero"));
        }
        let user_agent = match self.user_agent {
            Some(custom) => {
                HeaderValue::try_from(format!("{} {}", custom.trim(), transport::USER_AGENT))
                    .map_err(|_| {
                        Error::InvalidInput(
                            "user agent contains characters that are not allowed in an HTTP header",
                        )
                    })?
            }
            None => HeaderValue::from_static(transport::USER_AGENT),
        };
        let http = match self.http_client {
            Some(client) => client,
            None => reqwest::Client::builder()
                .build()
                .map_err(TransportError::new)?,
        };

        let transport = Transport {
            http,
            base_url,
            global_key,
            user_agent,
            timeout: self.timeout,
            limits: RateLimits::default(),
        };
        Ok(Client {
            transport: Arc::new(transport),
            server: ServerContext::new(server_key),
        })
    }
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let redact = |key: &Option<String>| key.as_ref().map(|_| "<redacted>");
        f.debug_struct("ClientBuilder")
            .field("server_key", &redact(&self.server_key))
            .field("global_key", &redact(&self.global_key))
            .field("base_url", &self.base_url)
            .field("user_agent", &self.user_agent)
            .field("http_client", &self.http_client.is_some())
            .field("timeout", &self.timeout)
            .finish()
    }
}
