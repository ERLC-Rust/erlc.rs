//! Staff models.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The server's staff, keyed by Roblox user ID with the username as value.
///
/// Owners and co-owners are listed on [`ServerInfo`](super::ServerInfo).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct Staff {
    /// Server administrators.
    #[serde(default)]
    pub admins: BTreeMap<u64, String>,
    /// Server moderators.
    #[serde(default)]
    pub mods: BTreeMap<u64, String>,
    /// Server helpers.
    #[serde(default)]
    pub helpers: BTreeMap<u64, String>,
}
