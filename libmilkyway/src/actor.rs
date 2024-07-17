pub mod binder;

///
/// The internal trait for handling request to actor-like
/// services.
///
/// # Template arguments
/// * T: message type
/// 
///
pub trait ActorHandler<T: Send + Sync> {
    ///
    /// Handles request and optionally returns response
    ///
    /// # Arguments
    /// * query: Q: a query message received
    ///
    /// # Returns
    /// * Option<R>: response to given query. Maybe None if query does not need
    ///              to be responded
    ///
    fn handle_request(&mut self, query: T) -> Option<T>;
}