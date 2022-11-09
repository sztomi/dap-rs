use std::io::{BufWriter, Write};

use serde::Serialize;
use serde_json;

use crate::{errors::ClientError, events::Event, requests::Request, responses::Response};

pub type Result<T> = std::result::Result<T, ClientError>;

/// Client trait representing a connected DAP client that is able to receive events
/// and reverse requests.
pub trait Client {
  /// Sends a response to the client.
  fn respond(&mut self, response: Response) -> Result<()>;
  /// Sends an even to the client.
  fn send_event(&mut self, event: Event) -> Result<()>;
  /// Sends a reverse request to the client.
  fn send_reverse_request(&mut self, request: Request) -> Result<()>;
}

pub struct BasicClient<W: Write> {
  stream: BufWriter<W>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum Sendable {
  Response(Response),
  Event(Event),
  //Request(Request),
}

impl<W: Write> BasicClient<W> {
  pub fn new(stream: W) -> Self {
    Self {
      stream: BufWriter::new(stream),
    }
  }

  fn send(&mut self, s: Sendable) -> Result<()> {
    let resp_json = serde_json::to_string(&s).map_err(ClientError::SerializationError)?;
    write!(self.stream, "Content-Length: {}\r\n\r\n", resp_json.len()).map_err(ClientError::IoError)?;
    println!("{resp_json}\n");
    write!(self.stream, "{}\r\n", resp_json).map_err(ClientError::IoError)?;
    Ok(())
  }
}

impl<W: Write> Client for BasicClient<W> {
  fn respond(&mut self, response: Response) -> Result<()> {
    self.send(Sendable::Response(response))
  }

  fn send_event(&mut self, event: Event) -> Result<()> {
    self.send(Sendable::Event(event))
  }

  fn send_reverse_request(&mut self, request: Request) -> Result<()> {
    todo!()
  }
}
