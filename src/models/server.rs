//! Server status models.

use serde::{Deserialize, Serialize};

use super::{CommandLog, EmergencyCall, JoinLog, KillLog, ModCall, Player, Staff, Vehicle};

/// Basic status of a private server, returned by
/// [`Client::server_info`](crate::Client::server_info).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct ServerInfo {
    /// The server name.
    pub name: String,
    /// Roblox user ID of the server owner.
    pub owner_id: u64,
    /// Roblox user IDs of the server co-owners.
    pub co_owner_ids: Vec<u64>,
    /// Number of players currently in the server.
    pub current_players: u32,
    /// Maximum number of players.
    pub max_players: u32,
    /// The code players use to join the server.
    pub join_key: String,
    /// The account verification requirement, for example `"Disabled"`, `"Email"` or `"Phone/ID"`.
    pub acc_verified_req: String,
    /// Whether team balance is enabled.
    pub team_balance: bool,
}

/// Server status plus the optional data requested with a
/// [`ServerQuery`](crate::ServerQuery), returned by
/// [`Client::server_with`](crate::Client::server_with).
///
/// Each optional field is `Some` only if it was requested and the API returned it,
/// so "not requested" and "empty" can be told apart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct ServerSnapshot {
    /// The basic server status, always present.
    #[serde(flatten)]
    pub info: ServerInfo,
    /// Players in the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<Player>>,
    /// Server staff.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff: Option<Staff>,
    /// Join and leave log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join_logs: Option<Vec<JoinLog>>,
    /// Roblox user IDs of players waiting in the join queue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<Vec<u64>>,
    /// Kill log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kill_logs: Option<Vec<KillLog>>,
    /// Command log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_logs: Option<Vec<CommandLog>>,
    /// Moderator call log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mod_calls: Option<Vec<ModCall>>,
    /// Emergency call log.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emergency_calls: Option<Vec<EmergencyCall>>,
    /// Vehicles spawned in the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vehicles: Option<Vec<Vehicle>>,
}

/// The response to a command, returned by
/// [`Client::run_command`](crate::Client::run_command).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CommandResponse {
    /// The status message from the API, usually `"Success"`.
    #[serde(default)]
    pub message: String,
}
