//! Endpoint methods, implemented on [`Client`](crate::Client) one file per API area.

mod bans;
mod command;
mod server;

pub use server::ServerQuery;
