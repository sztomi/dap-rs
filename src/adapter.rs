use crate::{client::Context, requests::Request, responses::Response};

/// Trait for an debug adapter.
///
/// Adapters are the main backbone of a debug server. They get a `accept` call for each
/// incoming request. Responses are the return values of these calls.
pub trait Adapter {
  /// Associated type for the Error that this adapter might return.
  ///
  /// This type is bubbled up into the return type of the [Server::run](crate::Server::run) function.
  type Error;
  /// Accept (and take ownership) of an incoming request.
  ///
  /// This is the primary entry point for debug adapters, where deserialized requests
  /// can be processed.
  ///
  /// The `ctx` reference can be used to send events and reverse requests to the client.
  ///
  /// # Error handling
  ///
  /// This function always returns a valid `Response` object,  however, that response
  /// itself may be an error response. As such, implementors should map their errors to
  /// an error response to allow clients to handle them. This is in the interest of users -
  /// the debug adapter is not something that users directly interact with nor something
  /// that they necessarily know about. From the users' perspective, it's an implementation
  /// detail and they are using their editor to debug something.
  fn accept(&mut self, request: Request, ctx: &mut dyn Context) -> Result<Response, Self::Error>;
}
