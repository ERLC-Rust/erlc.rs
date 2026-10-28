//! Typed response models.
//!
//! Field names follow Rust conventions; the original ER:LC JSON names are kept when
//! serializing. Unknown fields in responses are ignored, so new API fields do not
//! break existing code.

mod common;
mod emergency;
mod logs;
mod player;
mod server;
mod staff;
mod vehicle;

pub use common::{PlayerRef, Timestamp};
pub use emergency::EmergencyCall;
pub use logs::{CommandLog, JoinLog, KillLog, ModCall};
pub use player::{Location, Permission, Player};
pub use server::{CommandResponse, ServerInfo, ServerSnapshot};
pub use staff::Staff;
pub use vehicle::Vehicle;
