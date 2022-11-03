use crate::{requests::Request, responses::Response, client::Client};

/// Trait for an debug adapter.
///
/// Adapters are the main backbone of a debug server. They get a `accept` call for each
/// incoming request. Responses are the return values of these calls.
pub trait Adapter {
  /// Construct an Adapter while taking ownership of the Client which can be used
  /// for communicating with the client directly (sending [`Event`]s and reverse requests)
  fn new(client: Box<dyn Client>) -> Self;
  /// Accept (and take ownership) of an incoming request.
  ///
  /// This is the primary entry point for debug adapters, where deserialized requests
  /// can be processed.
  ///
  /// # Arguments
  ///
  ///   * `request`: A request, containing a [`Command`] and sequence number.
  fn accept(&mut self, request: Request) -> Response;
}