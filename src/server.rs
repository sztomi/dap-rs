use std::fmt::Debug;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use serde_json;

use crate::{
  base_message::{BaseMessage, Sendable},
  errors::{DeserializationError, ServerError},
  events::Event,
  requests::Request,
  responses::Response,
  reverse_requests::ReverseRequest,
};

#[derive(Debug)]
enum ServerState {
  /// Expecting a header
  Header,
  /// Expecting content
  Content,
}

/// Handles message encoding and decoding of messages.
///
/// The `Server` is responsible for reading the incoming bytestream and constructing deserialized
/// requests from it, as well as constructing and serializing outgoing messages.
pub struct Server<R: Read, W: Write> {
  input_buffer: BufReader<R>,
  output_buffer: BufWriter<W>,
  sequence_number: i64,
}

impl<R: Read, W: Write> Server<R, W> {
  /// Construct a new Server using the given input and output streams.
  pub fn new(input: BufReader<R>, output: BufWriter<W>) -> Self {
    Self {
      input_buffer: input,
      output_buffer: output,
      sequence_number: 0,
    }
  }

  /// Wait for a request from the development tool
  ///
  /// This will start reading the `input` buffer that is passed to it and will try to interpret
  /// the incoming bytes according to the DAP protocol.
  pub fn poll_request(&mut self) -> Result<Option<Request>, ServerError> {
    let mut state = ServerState::Header;
    let mut buffer = String::new();
    let mut content_length: usize = 0;

    loop {
      match self.input_buffer.read_line(&mut buffer) {
        Ok(read_size) => {
          if read_size == 0 {
            break Ok(None);
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
              self
                .input_buffer
                .read_exact(content.as_mut_slice())
                .map_err(ServerError::IoError)?;

              let content = std::str::from_utf8(content.as_slice())
                .map_err(|e| ServerError::ParseError(DeserializationError::DecodingError(e)))?;
              let request: Request = serde_json::from_str(content)
                .map_err(|e| ServerError::ParseError(DeserializationError::SerdeError(e)))?;
              return Ok(Some(request));
            }
          }
        }
        Err(e) => return Err(ServerError::IoError(e)),
      }
    }
  }

  pub fn send(&mut self, body: Sendable) -> Result<(), ServerError> {
    self.sequence_number += 1;

    let message = BaseMessage {
      seq: self.sequence_number,
      message: body,
    };

    let resp_json = serde_json::to_string(&message).map_err(ServerError::SerializationError)?;
    write!(
      self.output_buffer,
      "Content-Length: {}\r\n\r\n",
      resp_json.len()
    )
    .map_err(ServerError::IoError)?;

    write!(self.output_buffer, "{}\r\n", resp_json).map_err(ServerError::IoError)?;
    self.output_buffer.flush().map_err(ServerError::IoError)?;
    Ok(())
  }

  pub fn respond(&mut self, response: Response) -> Result<(), ServerError> {
    self.send(Sendable::Response(response))
  }

  pub fn send_event(&mut self, event: Event) -> Result<(), ServerError> {
    self.send(Sendable::Event(event))
  }

  pub fn send_reverse_request(&mut self, request: ReverseRequest) -> Result<(), ServerError> {
    self.send(Sendable::ReverseRequest(request))
  }
}
