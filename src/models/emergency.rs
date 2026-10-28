//! Emergency call models.

use serde::{Deserialize, Serialize};

use super::Timestamp;

/// An in-game emergency call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct EmergencyCall {
    /// The team the call was made to, for example `"Police"`.
    pub team: String,
    /// Roblox user ID of the caller.
    pub caller: u64,
    /// Roblox user IDs of the players responding.
    #[serde(default)]
    pub players: Vec<u64>,
    /// Map position of the call as `[x, z]`, in the same coordinates as
    /// [`Location`](super::Location).
    pub position: [f64; 2],
    /// When the call started.
    pub started_at: Timestamp,
    /// The call number.
    pub call_number: u64,
    /// The caller's description.
    #[serde(default)]
    pub description: String,
    /// A description of the call's location.
    #[serde(default)]
    pub position_descriptor: String,
}
