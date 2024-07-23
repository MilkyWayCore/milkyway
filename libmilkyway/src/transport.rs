pub mod crypto;
mod async_stream;

use crate::message::common::Message;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};

/** This is a constant address for a main server/broker **/
pub const TRANSPORT_TARGET_SERVER: u128 = 1;

///
/// The extensions allow to transform/detransform data.
/// Each Transport SHOULD NOT have more than one transformer.
///
pub trait TransportTransformer: Send + Sync{
    ///
    /// Detransforms received data
    /// E.g. decrypts it.
    ///
    /// # Arguments
    /// * data: &Serialized: data to detransform
    ///
    /// # Returns
    /// Detransformed serialized data
    fn detransform(&self, data: &Serialized) -> Result<Serialized, SerializationError>;

    ///
    /// Transforms data before sending.
    /// E.g. encrypts it.
    ///
    /// # Arguments
    /// * data: &Serialized: data to transform.
    ///
    /// # Returns
    /// Transformed data
    fn transform(&self, data: &Serialized) -> Serialized;
}

///
/// Listens and handles messages
///
pub trait TransportListener: Send + Sync{
    ///
    /// Handles messages
    ///
    fn on_message(&mut self, message: Message);
}

///
/// Allows sending messages
/// 
pub trait TransportSender: Send + Sync{
    ///
    /// Sends a message. MUST NOT block thread/coroutine
    /// 
    /// # Arguments
    /// * message: a message to send
    ///
    fn send_message(&mut self, message: Message);
}
