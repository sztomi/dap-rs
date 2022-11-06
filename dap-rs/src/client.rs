use std::{io::{BufWriter, Write}};

use serde_json;

use crate::{events::Event, requests::Request, responses::{Response, self}};

/// Client trait representing a connected DAP client that is able to receive events
/// and reverse requests.
pub trait Client {
  /// Sends a response to the client.
  fn respond(&mut self, response: Response);
  /// Sends an even to the client.
  fn send_event(&mut self, event: Event);
  /// Sends a reverse request to the client.
  fn send_reverse_request(&mut self, request: Request);
}

pub struct BasicClient<W: Write> {
  stream: BufWriter<W>,
}

impl<W: Write> BasicClient<W> {
  pub fn new(stream: W) -> Self {
    Self {
      stream: BufWriter::new(stream),
    }
  }

  fn serialize(&self, response: Response) -> String {
    serde_json::to_string(&response).unwrap()
  }

  fn write_header(&mut self, response: &str) {
    write!(self.stream, "Content-Length: {}\r\n\r\n", response.len()).unwrap() // TODO
  }
}

impl<W: Write> Client for BasicClient<W> {
  fn respond(&mut self, response: Response) {
    let resp_json = self.serialize(response);
    self.write_header(&resp_json);
    println!("{resp_json}\n");
    write!(self.stream, "{}\r\n", resp_json);
  }

  fn send_event(&mut self, event: Event) {
    todo!()
  }

  fn send_reverse_request(&mut self, request: Request) {
    todo!()
  }
}
