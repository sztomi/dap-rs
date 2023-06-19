use std::fmt::Debug;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use serde_json;

use crate::errors::{DeserializationError, ServerError};
use crate::protocol_message::Sendable;
use crate::requests::Request;
use crate::{
  events::Event, protocol_message::DAPMessage, responses::Response,
  reverse_requests::ReverseRequest,
};

#[derive(Debug)]
enum ServerState {
  /// Expecting a header
  Header,
  /// Expecting content
  Content,
}

/// Ties together an Adapter and a Client.
///
/// The `Server` is responsible for reading the incoming bytestream and constructing deserialized
/// requests from it; calling the `accept` function of the `Adapter` and passing the response
/// to the client.
pub struct Server<R: Read, W: Write> {
  input_buffer: BufReader<R>,
  output_buffer: BufWriter<W>,
  seq: i64,
}

impl<R: Read, W: Write> Server<R, W> {
  /// Construct a new Server and take ownership of the adapter and client.
  pub fn new(input: BufReader<R>, output: BufWriter<W>) -> Self {
    Self {
      input_buffer: input,
      output_buffer: output,
      seq: 0,
    }
  }

  fn next_seq(&mut self) -> i64 {
    self.seq += 1;
    self.seq
  }

  /// Run the server.
  ///
  /// This will start reading the `input` buffer that is passed to it and will try to interpert
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
              if self
                .input_buffer
                .read_exact(content.as_mut_slice())
                .is_err()
              {
                return Err(ServerError::IoError);
              }
              let request: Request =
                match serde_json::from_str(std::str::from_utf8(content.as_slice()).unwrap()) {
                  Ok(val) => val,
                  Err(e) => {
                    return Err(ServerError::ParseError(DeserializationError::SerdeError(e)));
                  }
                };

              return Ok(Some(request));
            }
          }
        }
        Err(_) => return Err(ServerError::IoError),
      }
    }
  }

  pub fn send(&mut self, message: DAPMessage) -> Result<(), ServerError> {
    let resp_json = serde_json::to_string(&message).map_err(ServerError::SerializationError)?;
    write!(
      self.output_buffer,
      "Content-Length: {}\r\n\r\n",
      resp_json.len()
    )
    .unwrap();

    write!(self.output_buffer, "{}\r\n", resp_json).unwrap();
    self.output_buffer.flush().unwrap();
    Ok(())
  }

  pub fn respond(&mut self, response: Response) -> Result<(), ServerError> {
    let message = DAPMessage {
      seq: self.next_seq(),
      message: Sendable::Response(response),
    };
    self.send(message)
  }

  pub fn send_event(&mut self, event: Event) -> Result<(), ServerError> {
    let message = DAPMessage {
      seq: self.next_seq(),
      message: Sendable::Event(event),
    };
    self.send(message)
  }

  pub fn send_reverse_request(&mut self, request: ReverseRequest) -> Result<(), ServerError> {
    let message = DAPMessage {
      seq: self.next_seq(),
      message: Sendable::ReverseRequest(request),
    };
    self.send(message)
  }
}

#[cfg(test)]
mod tests {

  use crate::requests::Command;

  use super::*;
  use std::sync::mpsc;
  use std::sync::mpsc::{Receiver, Sender};

  struct TestSender {
    sender: Sender<u8>,
  }

  impl Write for TestSender {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
      let mut i = 0;
      let loop_to = buf.len();

      while i < loop_to {
        self.sender.send(buf[i]).unwrap();
        i += 1;
      }

      Ok(loop_to)
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
      Ok(())
    }
  }

  struct TestRecv {
    recv: Receiver<u8>,
  }

  impl Read for TestRecv {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
      let mut i = 0;
      while let Ok(d) = self.recv.try_recv() {
        buf[i] = d;
        i += 1;
      }
      Ok(i)
    }
  }

  fn create_test_pair() -> (TestRecv, TestSender) {
    let (sender, recv) = mpsc::channel();
    (TestRecv { recv }, TestSender { sender })
  }

  #[test]
  fn test_server_init_request() {
    let (mut server_in, mut test_out) = create_test_pair();
    let (mut test_in, mut server_out) = create_test_pair();

    let mut server = Server::new(BufReader::new(server_in), BufWriter::new(server_out));

    write!(test_out, "Content-Length: 515\r\n\r\n");
    test_out.write(b"{\"command\":\"initialize\",\"arguments\":{\"clientID\":\"vscode\",\"clientName\":\"Visual Studio Code\",\"adapterID\":\"retread\",\"pathFormat\":\"path\",\"linesStartAt1\":true,\"columnsStartAt1\":true,\"supportsVariableType\":true,\"supportsVariablePaging\":true,\"supportsRunInTerminalRequest\":true,\"locale\":\"en\",\"supportsProgressReporting\":true,\"supportsInvalidatedEvent\":true,\"supportsMemoryReferences\":true,\"supportsArgsCanBeInterpretedByShell\":true,\"supportsMemoryEvent\":true,\"supportsStartDebuggingRequest\":true},\"type\":\"request\",\"seq\":1}");

    let req = server.poll_request().unwrap().unwrap();
    assert_eq!(req.seq, 1);
    assert!(matches!(req.command, Command::Initialize { .. }));
  }
}
