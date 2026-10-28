//! Vehicle models.

use serde::{Deserialize, Serialize};

/// A vehicle spawned in the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct Vehicle {
    /// The vehicle model, for example `"Redline Fire Engine"`.
    pub name: String,
    /// The owner's Roblox username.
    pub owner: String,
    /// The licence plate.
    #[serde(default)]
    pub plate: String,
    /// The livery, if one is applied.
    #[serde(default)]
    pub texture: Option<String>,
    /// The colour as a hex string, for example `"#ff4444"`.
    #[serde(default)]
    pub color_hex: String,
    /// The colour name, for example `"Super Red"`.
    #[serde(default)]
    pub color_name: String,
}
