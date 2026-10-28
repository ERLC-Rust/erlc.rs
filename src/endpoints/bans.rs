//! `GET /v1/server/bans`.

use std::collections::BTreeMap;

use crate::Client;
use crate::error::Result;

impl Client {
    /// Fetches the players banned from the server, keyed by Roblox user ID with
    /// the username as value.
    ///
    /// Calls `GET /v1/server/bans`, the only v1 endpoint with no v2 equivalent.
    ///
    /// # Errors
    ///
    /// See [`server_info`](Self::server_info).
    ///
    /// ```no_run
    /// # async fn run(client: erlc::Client) -> erlc::Result<()> {
    /// for (user_id, name) in client.bans().await? {
    ///     println!("{name} ({user_id})");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn bans(&self) -> Result<BTreeMap<u64, String>> {
        self.get("/v1/server/bans", &[]).await
    }
}
