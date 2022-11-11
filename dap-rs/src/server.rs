use std::io::BufRead;

use serde_json;

use crate::ClientContext;
use crate::adapter::Adapter;
use crate::client::Client;
use crate::errors::{DeserializationError, ServerError};
use crate::requests::Request;

#[derive(Debug)]
enum InputState {
  /// Expecting a header
  Header,
  /// Expecting a separator between header and content, i.e. "\r\n"
  Sep,
  /// Expecting content
  Content,
}

pub struct Server<A: Adapter, C: Client> {
  adapter: A,
  client: C,
}

impl<A: Adapter, C: Client + ClientContext> Server<A, C> {
  pub fn new(adapter: A, client: C) -> Self {
    Self { adapter, client }
  }

  pub fn run<Buf: BufRead>(&mut self, input: &mut Buf) -> Result<(), ServerError> {
    let mut state = InputState::Header;
    let mut buffer = String::new();
    let mut content_length: usize = 0;

    loop {
      match input.read_line(&mut buffer) {
        Ok(mut read_size) => {
          if read_size == 0 {
            break Ok(());
          }
          match state {
            InputState::Header => {
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
                    state = InputState::Sep;
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
            InputState::Sep => {
              if buffer == "\r\n" {
                state = InputState::Content;
                buffer.clear();
              }
            }
            InputState::Content => {
              while read_size < content_length {
                read_size += input.read_line(&mut buffer).unwrap();
              }
              let request: Request = match serde_json::from_str(&buffer) {
                Ok(val) => val,
                Err(e) => return Err(ServerError::ParseError(DeserializationError::SerdeError(e))),
              };
              let response = self.adapter.accept(request, &mut self.client);
              self.client.respond(response).map_err(ServerError::ClientError)?;
              state = InputState::Header;
              buffer.clear();
            }
          }
        }
        Err(_) => return Err(ServerError::IoError),
      }
    }
  }
}
