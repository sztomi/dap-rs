use crate::{requests::Request, responses::Response};

/// Trait for an debug adapter.
///
/// Adapters are the main backbone of a debug server. They get a `accept` call for each
/// incoming request. Responses are the return values of these calls.
pub trait Adapter {
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