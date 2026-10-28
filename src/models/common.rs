//! Types shared by several models.

use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A player as the API identifies them: a `"Name:UserId"` string.
///
/// The raw value is kept unchanged; [`name`](Self::name) and
/// [`user_id`](Self::user_id) split it for convenience.
///
/// ```
/// use erlc::models::PlayerRef;
///
/// let player = PlayerRef::new("Builderman:156");
/// assert_eq!(player.name(), "Builderman");
/// assert_eq!(player.user_id(), Some(156));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PlayerRef(String);

impl PlayerRef {
    /// Wraps a raw `"Name:UserId"` value.
    pub fn new(raw: impl Into<String>) -> Self {
        Self(raw.into())
    }

    /// The raw value, exactly as sent by the API.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The Roblox username, or the whole value if it has no `:UserId` suffix.
    pub fn name(&self) -> &str {
        self.split().map_or(&self.0, |(name, _)| name)
    }

    /// The Roblox user ID, if the value ends in `:UserId`.
    pub fn user_id(&self) -> Option<u64> {
        self.split().map(|(_, id)| id)
    }

    fn split(&self) -> Option<(&str, u64)> {
        let (name, id) = self.0.rsplit_once(':')?;
        Some((name, id.parse().ok()?))
    }
}

impl fmt::Display for PlayerRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A Unix timestamp in seconds, as sent by the API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Creates a timestamp from seconds since the Unix epoch.
    pub fn from_secs(secs: i64) -> Self {
        Self(secs)
    }

    /// Seconds since the Unix epoch.
    pub fn as_secs(self) -> i64 {
        self.0
    }

    /// Converts to a [`SystemTime`].
    pub fn to_system_time(self) -> SystemTime {
        let offset = Duration::from_secs(self.0.unsigned_abs());
        if self.0 >= 0 {
            UNIX_EPOCH + offset
        } else {
            UNIX_EPOCH - offset
        }
    }
}

impl From<Timestamp> for SystemTime {
    fn from(ts: Timestamp) -> Self {
        ts.to_system_time()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_name_and_id() {
        let p = PlayerRef::new("PlayerName:123456789");
        assert_eq!(p.name(), "PlayerName");
        assert_eq!(p.user_id(), Some(123_456_789));
        assert_eq!(p.as_str(), "PlayerName:123456789");
    }

    #[test]
    fn falls_back_to_raw_value() {
        for raw in ["NoId", "Name:notanumber", "Name:", ""] {
            let p = PlayerRef::new(raw);
            assert_eq!(p.name(), raw);
            assert_eq!(p.user_id(), None);
        }
    }

    #[test]
    fn splits_on_last_colon() {
        let p = PlayerRef::new("odd:name:42");
        assert_eq!(p.name(), "odd:name");
        assert_eq!(p.user_id(), Some(42));
    }

    #[test]
    fn converts_timestamps() {
        let ts = Timestamp::from_secs(1_704_614_400);
        assert_eq!(
            ts.to_system_time().duration_since(UNIX_EPOCH).unwrap(),
            Duration::from_secs(1_704_614_400)
        );
        assert!(Timestamp::from_secs(-1).to_system_time() < UNIX_EPOCH);
    }
}
