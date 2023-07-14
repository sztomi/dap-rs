//! # dap-rs, a Rust implementation of the Debug Adapter Protocol
//!
//! ## Introduction
//!
//! This crate is a Rust implementation of the [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/)
//! (or DAP for short).
//!
//! The best way to think of DAP is to compare it to [LSP](https://microsoft.github.io/language-server-protocol/)
//! (Language Server Protocol) but for debuggers. The core idea is the same: a protocol that serves
//! as *lingua franca* for editors and debuggers to talk to each other. This means that an editor
//! that implements DAP can use a debugger that also implements DAP.
//!
//! In practice, the adapter might be separate from the actual debugger. For example, one could
//! implement an adapter that writes commands to the stdin of a gdb subprocess, then parses
//! the output it receives (this is why it's called an "adapter" - it adapts the debugger to
//! editors that know DAP).
//!
//! ## Minimal example
//!
//! To get started, create a binary project and add `dap` to your Cargo.toml:
//!
//! ```toml
//! [package]
//! name = "dummy-server"
//! version = "*"
//! edition = "2021"
//!
//! [dependencies]
//! dap = "*"
//! ```
//!
//! Our dummy server is going to read its input from a text file and write the output to stdout.
//!
//! ```rust
//! use std::fs::File;
//! use std::io::{BufReader, BufWriter};
//!
//! use thiserror::Error;
//!
//! use dap::prelude::*;
//!
//! #[derive(Error, Debug)]
//! enum MyAdapterError {
//!   #[error("Unhandled command")]
//!   UnhandledCommandError,
//!
//!   #[error("Missing command")]
//!   MissingCommandError,
//! }
//!
//! type DynResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;
//!
//! fn main() -> DynResult<()> {
//!   let output = BufWriter::new(std::io::stdout());
//!   let f = File::open("testinput.txt")?;
//!   let input = BufReader::new(f);
//!   let mut server = Server::new(input, output);
//!
//!   let req = match server.poll_request()? {
//!     Some(req) => req,
//!     None => return Err(Box::new(MyAdapterError::MissingCommandError)),
//!   };
//!   if let Command::Initialize(_) = req.command {
//!     let rsp = req.success(
//!       ResponseBody::Initialize(Some(types::Capabilities {
//!         ..Default::default()
//!       })),
//!     );
//!
//!     // When you call respond, send_event etc. the message will be wrapped
//!     // in a base message with a appropriate seq number, so you don't have to keep track of that yourself
//!     server.respond(rsp)?;
//!
//!     server.send_event(Event::Initialized)?;
//!   } else {
//!     return Err(Box::new(MyAdapterError::UnhandledCommandError));
//!   }
//!   Ok(())
//! }
//! ```
pub mod base_message;
pub mod errors;
pub mod events;
#[doc(hidden)]
mod macros;
pub mod prelude;
pub mod requests;
pub mod responses;
pub mod reverse_requests;
pub mod server;
pub mod types;
