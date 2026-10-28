//! Async, type-safe client for the [ER:LC Private Server API](https://apidocs.erlc.gg/).
//!
//! Step-by-step guides live in the [`docs/` folder](https://github.com/ERLC-Rust/erlc.rs/tree/main/docs).
//!
//! # Quick start
//!
//! ```no_run
//! # async fn run() -> erlc::Result<()> {
//! let key = std::env::var("ERLC_SERVER_KEY").unwrap_or_default();
//! let client = erlc::Client::new(key)?;
//!
//! let info = client.server_info().await?;
//! println!("{}: {}/{} players", info.name, info.current_players, info.max_players);
//! # Ok(())
//! # }
//! ```
//!
//! # Keeping your server key safe
//!
//! A server key grants the same access as a server Co-Owner, without the in-game
//! audit trail. Load it from an environment variable or a secret store, never
//! commit it, and regenerate it in your private server settings if it leaks.
//! The client never includes keys in `Debug` output or error messages.
//!
//! # Errors
//!
//! Every method returns [`Result`]. See [`Error`] for the possible failures and
//! [`ErrorCode`] for the documented ER:LC error codes.
//!
//! # Rate limits
//!
//! The client follows the ER:LC rate-limit rules: after a 429 it refuses to send
//! requests to the same bucket until the `Retry-After` delay has passed, and it
//! never retries a request on its own. See [`Client`] for details.
//!
//! # Public applications
//!
//! An application that serves several servers must use a global API key: set
//! [`ClientBuilder::global_key`] and call [`Client::with_server_key`] for each server.
//!
//! # Cargo features
//!
//! - `rustls` (default): TLS through rustls.
//! - `native-tls`: TLS through the platform's native library.
//!
//! At least one TLS feature must be enabled to reach the real API over HTTPS.

mod auth;
mod client;
mod endpoints;
mod error;
pub mod models;
mod rate_limit;
mod transport;

pub use client::{Client, ClientBuilder};
pub use endpoints::ServerQuery;
pub use error::{ApiError, DecodeError, Error, ErrorCode, RateLimited, Result, TransportError};
pub use rate_limit::RateLimitInfo;
/// The HTTP client this crate uses, re-exported for
/// [`ClientBuilder::http_client`]. Its major version is part of this crate's public API.
pub use reqwest;

/// Compiles the README and guide code samples as doctests.
#[cfg(doctest)]
mod doc_samples {
    #[doc = include_str!("../README.md")]
    struct Readme;
    #[doc = include_str!("../docs/getting-started.md")]
    struct GettingStarted;
    #[doc = include_str!("../docs/client.md")]
    struct ClientGuide;
    #[doc = include_str!("../docs/server-data.md")]
    struct ServerData;
    #[doc = include_str!("../docs/commands.md")]
    struct Commands;
    #[doc = include_str!("../docs/errors.md")]
    struct Errors;
    #[doc = include_str!("../docs/rate-limits.md")]
    struct RateLimits;
    #[doc = include_str!("../docs/public-apps.md")]
    struct PublicApps;
    #[doc = include_str!("../docs/testing.md")]
    struct Testing;
}
