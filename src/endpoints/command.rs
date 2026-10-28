//! `POST /v2/server/command`.

use serde::Serialize;

use crate::Client;
use crate::error::{Error, Result};
use crate::models::CommandResponse;

#[derive(Serialize)]
struct CommandRequest<'a> {
    command: &'a str,
}

impl Client {
    /// Runs a command in the server as virtual server management, for example
    /// `":h Server restart in 5 minutes"`.
    ///
    /// Calls `POST /v2/server/command`. The command text is sent as given (after
    /// trimming surrounding whitespace), including its leading `:`.
    ///
    /// # Delivery
    ///
    /// The request is sent **exactly once** and never retried, so a moderation
    /// action cannot run twice. If the request times out
    /// ([`TransportError::is_timeout`](crate::TransportError::is_timeout)), the
    /// command may or may not have run.
    ///
    /// # Rate limits and policy
    ///
    /// Commands use the API's `command` rate-limit bucket, which is much stricter
    /// than reads (documented as 1 request per 5 seconds, subject to change).
    /// The ER:LC acceptable use policy forbids using `:pm` as a replacement for
    /// in-game chat, the phone or the radio.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidInput`] if `command` is empty; nothing is sent.
    /// - [`Error::Api`] with [`ApiError::is_server_offline`](crate::ApiError::is_server_offline)
    ///   if the server has no players, or with codes such as
    ///   [`InvalidCommand`](crate::ErrorCode::InvalidCommand),
    ///   [`RestrictedCommand`](crate::ErrorCode::RestrictedCommand) or
    ///   [`ProhibitedMessage`](crate::ErrorCode::ProhibitedMessage).
    /// - [`Error::RateLimited`] if the `command` bucket is exhausted.
    ///
    /// ```no_run
    /// # async fn run(client: erlc::Client) -> erlc::Result<()> {
    /// let response = client.run_command(":h Server restart in 5 minutes").await?;
    /// println!("{}", response.message);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_command(&self, command: impl AsRef<str>) -> Result<CommandResponse> {
        let command = command.as_ref().trim();
        if command.is_empty() {
            return Err(Error::InvalidInput("command must not be empty"));
        }
        self.post_json("/v2/server/command", &CommandRequest { command })
            .await
    }
}
