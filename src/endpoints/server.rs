//! `GET /v2/server`.

use crate::Client;
use crate::error::Result;
use crate::models::{
    CommandLog, EmergencyCall, JoinLog, KillLog, ModCall, Player, ServerInfo, ServerSnapshot,
    Staff, Vehicle,
};

const PATH: &str = "/v2/server";

/// Selects the optional data to include in [`Client::server_with`].
///
/// Requesting several fields in one call uses one request instead of several,
/// which matters under the API's rate limits.
///
/// ```
/// let query = erlc::ServerQuery::new().players().kill_logs();
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[must_use]
pub struct ServerQuery {
    players: bool,
    staff: bool,
    join_logs: bool,
    queue: bool,
    kill_logs: bool,
    command_logs: bool,
    mod_calls: bool,
    emergency_calls: bool,
    vehicles: bool,
}

impl ServerQuery {
    /// An empty query: only the basic server status.
    pub fn new() -> Self {
        Self::default()
    }

    /// Requests every optional field.
    pub fn all() -> Self {
        Self {
            players: true,
            staff: true,
            join_logs: true,
            queue: true,
            kill_logs: true,
            command_logs: true,
            mod_calls: true,
            emergency_calls: true,
            vehicles: true,
        }
    }

    /// Includes the players in the server.
    pub fn players(mut self) -> Self {
        self.players = true;
        self
    }

    /// Includes the server staff.
    pub fn staff(mut self) -> Self {
        self.staff = true;
        self
    }

    /// Includes the join and leave log.
    pub fn join_logs(mut self) -> Self {
        self.join_logs = true;
        self
    }

    /// Includes the join queue.
    pub fn queue(mut self) -> Self {
        self.queue = true;
        self
    }

    /// Includes the kill log.
    pub fn kill_logs(mut self) -> Self {
        self.kill_logs = true;
        self
    }

    /// Includes the command log.
    pub fn command_logs(mut self) -> Self {
        self.command_logs = true;
        self
    }

    /// Includes the moderator call log.
    pub fn mod_calls(mut self) -> Self {
        self.mod_calls = true;
        self
    }

    /// Includes the emergency call log.
    pub fn emergency_calls(mut self) -> Self {
        self.emergency_calls = true;
        self
    }

    /// Includes the spawned vehicles.
    pub fn vehicles(mut self) -> Self {
        self.vehicles = true;
        self
    }

    fn params(self) -> Vec<(&'static str, &'static str)> {
        [
            ("Players", self.players),
            ("Staff", self.staff),
            ("JoinLogs", self.join_logs),
            ("Queue", self.queue),
            ("KillLogs", self.kill_logs),
            ("CommandLogs", self.command_logs),
            ("ModCalls", self.mod_calls),
            ("EmergencyCalls", self.emergency_calls),
            ("Vehicles", self.vehicles),
        ]
        .into_iter()
        .filter(|&(_, enabled)| enabled)
        .map(|(name, _)| (name, "true"))
        .collect()
    }
}

impl Client {
    /// Fetches the server's status: name, owners, player counts and settings.
    ///
    /// Calls `GET /v2/server` with no optional fields.
    ///
    /// # Errors
    ///
    /// See [`Error`](crate::Error). Common cases are [`Error::Api`](crate::Error::Api)
    /// with [`ApiError::is_auth`](crate::ApiError::is_auth) for a bad key, and
    /// [`Error::RateLimited`](crate::Error::RateLimited).
    ///
    /// ```no_run
    /// # async fn run(client: erlc::Client) -> erlc::Result<()> {
    /// let info = client.server_info().await?;
    /// println!("{}: {}/{} players", info.name, info.current_players, info.max_players);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn server_info(&self) -> Result<ServerInfo> {
        self.get(PATH, &[]).await
    }

    /// Fetches the server's status plus the optional data selected in `query`,
    /// in a single request.
    ///
    /// Calls `GET /v2/server` with one query parameter per selected field.
    ///
    /// # Errors
    ///
    /// See [`server_info`](Self::server_info).
    ///
    /// ```no_run
    /// # async fn run(client: erlc::Client) -> erlc::Result<()> {
    /// use erlc::ServerQuery;
    ///
    /// let snapshot = client.server_with(ServerQuery::new().players().kill_logs()).await?;
    /// for kill in snapshot.kill_logs.unwrap_or_default() {
    ///     println!("{} killed {}", kill.killer.name(), kill.killed.name());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn server_with(&self, query: ServerQuery) -> Result<ServerSnapshot> {
        self.get(PATH, &query.params()).await
    }

    /// Fetches the players in the server.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with
    /// [`ServerQuery::players`]. Returns an empty list if the API omits the field.
    pub async fn players(&self) -> Result<Vec<Player>> {
        Ok(self
            .server_with(ServerQuery::new().players())
            .await?
            .players
            .unwrap_or_default())
    }

    /// Fetches the server staff.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with [`ServerQuery::staff`].
    pub async fn staff(&self) -> Result<Staff> {
        Ok(self
            .server_with(ServerQuery::new().staff())
            .await?
            .staff
            .unwrap_or_default())
    }

    /// Fetches the join and leave log.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with [`ServerQuery::join_logs`].
    pub async fn join_logs(&self) -> Result<Vec<JoinLog>> {
        Ok(self
            .server_with(ServerQuery::new().join_logs())
            .await?
            .join_logs
            .unwrap_or_default())
    }

    /// Fetches the Roblox user IDs of players in the join queue.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with [`ServerQuery::queue`].
    pub async fn queue(&self) -> Result<Vec<u64>> {
        Ok(self
            .server_with(ServerQuery::new().queue())
            .await?
            .queue
            .unwrap_or_default())
    }

    /// Fetches the kill log.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with [`ServerQuery::kill_logs`].
    pub async fn kill_logs(&self) -> Result<Vec<KillLog>> {
        Ok(self
            .server_with(ServerQuery::new().kill_logs())
            .await?
            .kill_logs
            .unwrap_or_default())
    }

    /// Fetches the command log.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with [`ServerQuery::command_logs`].
    pub async fn command_logs(&self) -> Result<Vec<CommandLog>> {
        Ok(self
            .server_with(ServerQuery::new().command_logs())
            .await?
            .command_logs
            .unwrap_or_default())
    }

    /// Fetches the moderator call log.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with [`ServerQuery::mod_calls`].
    pub async fn mod_calls(&self) -> Result<Vec<ModCall>> {
        Ok(self
            .server_with(ServerQuery::new().mod_calls())
            .await?
            .mod_calls
            .unwrap_or_default())
    }

    /// Fetches the emergency call log.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with
    /// [`ServerQuery::emergency_calls`].
    pub async fn emergency_calls(&self) -> Result<Vec<EmergencyCall>> {
        Ok(self
            .server_with(ServerQuery::new().emergency_calls())
            .await?
            .emergency_calls
            .unwrap_or_default())
    }

    /// Fetches the vehicles spawned in the server.
    ///
    /// Shortcut for [`server_with`](Self::server_with) with [`ServerQuery::vehicles`].
    pub async fn vehicles(&self) -> Result<Vec<Vehicle>> {
        Ok(self
            .server_with(ServerQuery::new().vehicles())
            .await?
            .vehicles
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_has_no_params() {
        assert!(ServerQuery::new().params().is_empty());
    }

    #[test]
    fn params_follow_documented_names_in_fixed_order() {
        let params = ServerQuery::new().vehicles().players().staff().params();
        assert_eq!(
            params,
            [("Players", "true"), ("Staff", "true"), ("Vehicles", "true")]
        );
        assert_eq!(ServerQuery::all().params().len(), 9);
    }
}
