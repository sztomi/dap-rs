use serde::Serialize;

use crate::{events::Event, responses::Response, reverse_requests::ReverseRequest};

/// Represents the base protocol message, in which all other messages are wrapped.
///
/// Specification: [Response](https://microsoft.github.io/debug-adapter-protocol/specification)
#[derive(Serialize, Debug)]
pub struct BaseMessage {
  /// Sequence number of the message. The `seq` for
  /// the first message is 1, and for each message is incremented by 1.
  pub seq: i64,
  #[serde(flatten)]
  pub message: Sendable,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Sendable {
  Response(Response),
  Event(Event),
  ReverseRequest(ReverseRequest),
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_message_serialize() {
    let message = BaseMessage {
      seq: 10,
      message: Sendable::Event(Event::Initialized),
    };
    let json = serde_json::to_string(&message).unwrap();

    let expected = "{\"seq\":10,\"type\":\"event\",\"event\":\"initialized\"}";
    assert_eq!(json, expected);
  }
}
