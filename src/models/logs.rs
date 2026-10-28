//! Log entry models.

use serde::{Deserialize, Serialize};

use super::{PlayerRef, Timestamp};

/// A player joining or leaving the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct JoinLog {
    /// `true` for a join, `false` for a leave.
    pub join: bool,
    /// When it happened.
    pub timestamp: Timestamp,
    /// The player.
    pub player: PlayerRef,
}

/// A player killing another player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct KillLog {
    /// The player who was killed.
    pub killed: PlayerRef,
    /// When it happened.
    pub timestamp: Timestamp,
    /// The player who killed them.
    pub killer: PlayerRef,
}

/// A command run in the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct CommandLog {
    /// The player who ran the command.
    pub player: PlayerRef,
    /// When it happened.
    pub timestamp: Timestamp,
    /// The command, for example `":h Hello"`.
    pub command: String,
}

/// A call for a moderator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct ModCall {
    /// The player who made the call.
    pub caller: PlayerRef,
    /// The moderator who responded. `None` until a moderator responds.
    #[serde(default)]
    pub moderator: Option<PlayerRef>,
    /// When the call was made.
    pub timestamp: Timestamp,
}
