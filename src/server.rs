use serde_json;
use std::fmt::Debug;

use crate::adapter::Adapter;
use crate::client::Client;
use crate::client::Context;
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
    pub async fn run(&mut self, input: &mut impl LineReader) -> Result<(), ServerError<A::Error>>
    where
        <A as Adapter>::Error: Debug + Sized,
    {
        let mut state = ServerState::Header;
        let mut content_length: usize = 0;

        loop {
            match input.read_line().await {
                Ok(mut buffer) => {
                    tracing::trace!("read line: {buffer}");
                    if buffer.is_empty() {
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
                                            Err(_) => {
                                                return Err(ServerError::HeaderParseError {
                                                    line: buffer,
                                                })
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
                            if buffer == "\r\n" {
                                state = ServerState::Content;
                            }
                        }
                        ServerState::Content => {
                            // read the content
                            let mut payload = bytes::BytesMut::with_capacity(content_length);
                            let _ = input.read_n_bytes(&mut payload, content_length).await;
                            buffer = String::from_utf8_lossy(&payload).to_string();
                            let request: Request = match serde_json::from_str(&buffer) {
                                Ok(val) => val,
                                Err(e) => {
                                    return Err(ServerError::ParseError(
                                        DeserializationError::SerdeError(e),
                                    ))
                                }
                            };
                            match self.adapter.accept(request, &mut self.client).await {
                                Ok(response) => match response.body {
                                    Some(ResponseBody::Empty) => (),
                                    _ => {
                                        self.client
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
                Err(_) => return Err(ServerError::IoError),
            }
        }
    }
}
