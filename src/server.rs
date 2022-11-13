use std::io::BufRead;

use serde_json;

use crate::adapter::Adapter;
use crate::client::Client;
use crate::errors::{DeserializationError, ServerError};
use crate::requests::Request;
use crate::Context;

#[derive(Debug)]
enum ServerState {
  /// Expecting a header
  Header,
  /// Expecting a separator between header and content, i.e. "\r\n"
  Sep,
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
  pub fn run<Buf: BufRead>(&mut self, input: &mut Buf) -> Result<(), ServerError> {
    let mut state = ServerState::Header;
    let mut buffer = String::new();
    let mut content_length: usize = 0;

    loop {
      match input.read_line(&mut buffer) {
        Ok(mut read_size) => {
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
                    state = ServerState::Sep;
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
            ServerState::Sep => {
              if buffer == "\r\n" {
                state = ServerState::Content;
                buffer.clear();
              }
            }
            ServerState::Content => {
              while read_size < content_length {
                read_size += input.read_line(&mut buffer).unwrap();
              }
              let request: Request = match serde_json::from_str(&buffer) {
                Ok(val) => val,
                Err(e) => return Err(ServerError::ParseError(DeserializationError::SerdeError(e))),
              };
              let response = self.adapter.accept(request, &mut self.client);
              self
                .client
                .respond(response)
                .map_err(ServerError::ClientError)?;

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
        Err(_) => return Err(ServerError::IoError),
      }
    }
  }
}
