use serde::Serialize;

use crate::{events::Event, responses::Response, reverse_requests::ReverseRequest};

#[derive(Serialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Sendable {
  Response(Response),
  Event(Event),
  ReverseRequest(ReverseRequest),
}

#[derive(Serialize, Debug)]
pub struct DAPMessage {
  pub seq: i64,
  #[serde(flatten)]
  pub message: Sendable,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_message_serialize() {
    let message = DAPMessage {
      seq: 10,
      message: Sendable::Event(Event::Initialized),
    };
    let json = serde_json::to_string(&message).unwrap();

    let expected = "{\"seq\":10,\"type\":\"event\",\"event\":\"initialized\"}";
    assert_eq!(json, expected);
  }
}
