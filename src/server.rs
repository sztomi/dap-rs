use std::fmt::Debug;
use std::io::BufRead;

use serde_json;

use crate::adapter::Adapter;
use crate::client::Client;
use crate::client::Context;
use crate::errors::{DeserializationError, ServerError};
use crate::prelude::ResponseBody;
use crate::requests::Request;

#[derive(Debug)]
enum ServerState {
  /// Expecting a header
  Header,
  /// Expecting content
  Content,
  /// Wants to exit
  Exiting,
}

/// Ties together an Adapter and a Client.
///
/// The `Server` is responsible for reading the incoming bytestream and constructing deserialized
/// requests from it; calling the `accept` function of the `Adapter` and passing the response
/// to the client.
pub struct Server<A: Adapter, C: Client + Context> {
  adapter: A,
  client: C,
}

impl<A: Adapter, C: Client + Context> Server<A, C> {
  /// Construct a new Server and take ownership of the adapter and client.
  pub fn new(adapter: A, client: C) -> Self {
    Self { adapter, client }
  }

  /// Run the server.
  ///
  /// This will start reading the `input` buffer that is passed to it and will try to interpert
  /// the incoming bytes according to the DAP protocol.
  pub fn run<Buf: BufRead>(&mut self, input: &mut Buf) -> Result<(), ServerError<A::Error>>
  where
    <A as Adapter>::Error: Debug + Sized,
  {
    let mut state = ServerState::Header;
    let mut buffer = String::new();
    let mut content_length: usize = 0;

    loop {
      match input.read_line(&mut buffer) {
        Ok(read_size) => {
          if read_size == 0 {
            break Ok(());
          }
          match state {
            ServerState::Header => {
              let parts: Vec<&str> = buffer.trim_end().split(':').collect();
              if parts.len() == 2 {
                match parts[0] {
                  "Content-Length" => {
                    content_length = match parts[1].trim().parse() {
                      Ok(val) => val,
                      Err(_) => return Err(ServerError::HeaderParseError { line: buffer }),
                    };
                    buffer.clear();
                    buffer.reserve(content_length);
                    state = ServerState::Content;
                  }
                  other => {
                    return Err(ServerError::UnknownHeader {
                      header: other.to_string(),
                    })
                  }
                }
              } else {
                return Err(ServerError::HeaderParseError { line: buffer });
              }
            }
            ServerState::Content => {
              buffer.clear();
              let mut content = vec![0; content_length];
              input
                .read_exact(content.as_mut_slice())
                .map_err(ServerError::IoError)?;
              let request: Request =
                match serde_json::from_str(std::str::from_utf8(content.as_slice()).unwrap()) {
                  Ok(val) => val,
                  Err(e) => {
                    return Err(ServerError::ParseError(DeserializationError::SerdeError(e)));
                  }
                };
              match self.adapter.accept(request, &mut self.client) {
                Ok(response) => match response.body {
                  Some(ResponseBody::Empty) => (),
                  _ => {
                    self
                      .client
                      .respond(response)
                      .map_err(ServerError::ClientError)?;
                  }
                },
                Err(e) => return Err(ServerError::AdapterError(e)),
              }

              if self.client.get_exit_state() {
                state = ServerState::Exiting;
                continue;
              }

              state = ServerState::Header;
              buffer.clear();
            }
            ServerState::Exiting => break Ok(()),
          }
        }
        Err(e) => return Err(ServerError::IoError(e)),
      }
    }
  }
}
