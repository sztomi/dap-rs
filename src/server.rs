use serde_json;
use std::fmt::Debug;

use crate::adapter::Adapter;
use crate::client::StdoutWriter;
use crate::errors::{DeserializationError, ServerError};
use crate::line_reader::LineReader;
use crate::prelude::ResponseBody;
use crate::requests::Request;

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
pub struct Server<A: Adapter> {
    adapter: A,
    client: StdoutWriter,
}

fn escape_crlf(instr: &String) -> String {
    let mut str = instr.replace("\n", "\\n");
    str = str.replace("\r", "\\r");
    str
}

impl<A: Adapter> Server<A> {
    /// Construct a new Server and take ownership of the adapter and client.
    pub fn new(adapter: A, client: StdoutWriter) -> Self {
        Self { adapter, client }
    }

    /// Run the server.
    ///
    /// This will start reading the `input` buffer that is passed to it and will try to interpert
    /// the incoming bytes according to the DAP protocol.
    pub async fn run(&mut self, input: &mut impl LineReader) -> Result<(), ServerError<A::Error>>
    where
        <A as Adapter>::Error: Debug + Sized,
    {
        let mut state = ServerState::Header;
        let mut content_length: usize = 0;

        loop {
            match state {
                ServerState::Header => {
                    let Ok(mut buffer) = input.read_line().await else {
                        return Err(ServerError::IoError)
                    };

                    tracing::trace!("HEADER: read line: {}", escape_crlf(&buffer));
                    if buffer.is_empty() {
                        break Ok(());
                    }

                    let parts: Vec<&str> = buffer.trim_end().split(':').collect();
                    if parts.len() == 2 {
                        match parts[0] {
                            "Content-Length" => {
                                content_length = match parts[1].trim().parse() {
                                    Ok(val) => val,
                                    Err(_) => {
                                        return Err(ServerError::HeaderParseError { line: buffer })
                                    }
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
                    let Ok(buffer) = input.read_line().await else {
                        return Err(ServerError::IoError)
                    };

                    tracing::trace!("SEP: read line: {}", escape_crlf(&buffer));
                    if buffer == "\r\n" {
                        state = ServerState::Content;
                    } else {
                        // expecting separator
                        return Err(ServerError::ProtocolError {
                            reason: "failed to read separator".to_string(),
                            line: "0".to_string(),
                        });
                    }
                }
                ServerState::Content => {
                    // read the payload
                    let mut payload = bytes::BytesMut::with_capacity(content_length);
                    if let Err(_) = input.read_n_bytes(&mut payload, content_length).await {
                        return Err(ServerError::IoError);
                    }

                    let payload = String::from_utf8_lossy(&payload).to_string();
                    tracing::trace!("CONTENT: read content: {}", escape_crlf(&payload));
                    let request: Request = match serde_json::from_str(&payload) {
                        Ok(val) => val,
                        Err(e) => {
                            return Err(ServerError::ParseError(DeserializationError::SerdeError(
                                e,
                            )))
                        }
                    };
                    // pass it to the adapter
                    match self.adapter.handle_request(request, &mut self.client).await {
                        Ok(response) => match response.body {
                            Some(ResponseBody::Empty) => (),
                            _ => {
                                self.client
                                    .send_response(response)
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
                    content_length = 0;
                }
                ServerState::Exiting => break Ok(()),
            }
        }
    }
}
