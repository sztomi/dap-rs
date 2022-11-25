#![doc = include_str!("../README.md")]
pub mod adapter;
pub mod client;
pub mod errors;
pub mod events;
#[doc(hidden)]
mod macros;
pub mod requests;
pub mod responses;
pub mod reverse_requests;
pub mod server;
pub mod types;
pub mod prelude;
