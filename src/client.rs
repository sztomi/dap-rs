use std::io::{BufWriter, Write};

use serde::Serialize;
use serde_json;

use crate::{
  errors::ClientError, events::Event, responses::Response, reverse_requests::ReverseRequest,
};

pub type Result<T> = std::result::Result<T, ClientError>;

/// Client trait representing a connected DAP client that is able to receive events
/// and reverse requests.
pub trait Client {
  /// Sends a response to the client.
  fn respond(&mut self, response: Response) -> Result<()>;
}

/// Trait for sending events and requests to the connected client.
pub trait Context {
  /// Sends an even to the client.
  fn send_event(&mut self, event: Event) -> Result<()>;
  /// Sends a reverse request to the client.
  fn send_reverse_request(&mut self, request: ReverseRequest) -> Result<()>;
  /// Notifies the server that it should gracefully exit after `accept`
  /// returned.
  ///
  /// It is recommended to send a `Terminated` and/or `Stopped` event to the client.
  fn request_exit(&mut self);
  /// Clears an exit request set by `request_exit` in the same `accept` call.
  /// This cannot be used to clear an exit request that happened during a previous
  /// `accept`.
  fn cancel_exit(&mut self);
  /// Returns `true` if the exiting was requested.
  fn get_exit_state(&self) -> bool;
}

pub struct BasicClient<W: Write> {
  stream: BufWriter<W>,
  should_exit: bool,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum Sendable {
  Response(Response),
  Event(Event),
  ReverseRequest(ReverseRequest),
}

impl<W: Write> BasicClient<W> {
  pub fn new(stream: W) -> Self {
    Self {
      stream: BufWriter::new(stream),
      should_exit: false,
    }
  }

  fn send(&mut self, s: Sendable) -> Result<()> {
    let resp_json = serde_json::to_string(&s).map_err(ClientError::SerializationError)?;
    write!(self.stream, "Content-Length: {}\r\n\r\n", resp_json.len())
      .map_err(ClientError::IoError)?;
    write!(self.stream, "{}\r\n", resp_json).map_err(ClientError::IoError)?;
    Ok(())
  }
}

impl<W: Write> Client for BasicClient<W> {
  fn respond(&mut self, response: Response) -> Result<()> {
    self.send(Sendable::Response(response))
  }
}

impl<W: Write> Context for BasicClient<W> {
  fn send_event(&mut self, event: Event) -> Result<()> {
    self.send(Sendable::Event(event))
  }

  fn send_reverse_request(&mut self, request: ReverseRequest) -> Result<()> {
    self.send(Sendable::ReverseRequest(request))
  }

  fn request_exit(&mut self) {
    self.should_exit = true;
  }

  fn cancel_exit(&mut self) {
    self.should_exit = false;
  }

  fn get_exit_state(&self) -> bool {
    self.should_exit
  }
}
