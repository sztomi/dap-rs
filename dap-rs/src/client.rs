use crate::{events::Event, requests::Request};


/// Client trait representing a connected DAP client that is able to receive events
/// and reverse requests.
pub trait Client {
  /// Sends an even to the client.
  fn send_event(&self, event: Event);
  /// Sends a reverse request to the client.
  fn send_reverse_requests(&self, request: Request);
}