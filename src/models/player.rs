//! Player models.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::PlayerRef;

/// A player currently in the server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct Player {
    /// The player, as `"Name:UserId"`.
    pub player: PlayerRef,
    /// The player's team, for example `"Police"`, `"Sheriff"` or `"Civilian"`.
    pub team: String,
    /// The player's callsign. Only set for players on a non-civilian team.
    #[serde(default)]
    pub callsign: Option<String>,
    /// The player's in-game permission level.
    pub permission: Permission,
    /// The player's wanted level.
    #[serde(default)]
    pub wanted_stars: u32,
    /// Where the player is on the map.
    #[serde(default)]
    pub location: Option<Location>,
}

/// A player's position on the ER:LC map.
///
/// `0, 0` is the centre of the map. Positive `location_x` is right (east), and
/// positive `location_z` is down (south) on the official map images.
/// See <https://apidocs.erlc.gg/how-do-i-interpret-the-xyz>.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[non_exhaustive]
pub struct Location {
    /// Horizontal position.
    pub location_x: f64,
    /// Vertical position.
    pub location_z: f64,
    /// The postal code of the area.
    #[serde(default)]
    pub postal_code: String,
    /// The street name.
    #[serde(default)]
    pub street_name: String,
    /// The building number, empty if none.
    #[serde(default)]
    pub building_number: String,
}

/// A player's in-game permission level.
///
/// Values not known when this crate was released are kept in [`Permission::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Permission {
    /// `"Normal"`: a regular player.
    Normal,
    /// `"Server Moderator"`.
    ServerModerator,
    /// `"Server Administrator"`.
    ServerAdministrator,
    /// `"Server Owner"`.
    ServerOwner,
    /// Any other value.
    Other(String),
}

impl Permission {
    /// The value as sent by the API.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Normal => "Normal",
            Self::ServerModerator => "Server Moderator",
            Self::ServerAdministrator => "Server Administrator",
            Self::ServerOwner => "Server Owner",
            Self::Other(value) => value,
        }
    }
}

impl From<String> for Permission {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Normal" => Self::Normal,
            "Server Moderator" => Self::ServerModerator,
            "Server Administrator" => Self::ServerAdministrator,
            "Server Owner" => Self::ServerOwner,
            _ => Self::Other(value),
        }
    }
}

impl Serialize for Permission {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Permission {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self::from)
    }
}
