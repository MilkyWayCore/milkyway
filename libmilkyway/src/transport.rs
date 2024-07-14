mod stream;

use async_trait::async_trait;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};



///
/// The extensions allow to transform/detransform data.
/// Each Transport SHOULD NOT have more than one transformer.
///
pub trait TransportTransformer: Send{
    ///
    /// Detransforms received data
    /// E.g. decrypts it.
    ///
    /// # Arguments
    /// * data: &Serialized: data to detransform
    ///
    /// # Returns
    /// Detransformed serialized data
    fn detransform(&self, data: &Serialized) -> Serialized;

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
/// Implements a communication with pieces of serialized data between to parties.
///
#[async_trait]
pub trait Transport{
    ///
    /// Sends a data to transport destination
    ///
    /// # Arguments
    /// * data: Serialized: data to send
    ///
    async fn send_raw(&mut self, data: Serialized) -> Result<usize, tokio::io::Error>;

    ///
    /// Receives a data from remote with in a timeout.
    ///
    /// # Arguments
    /// * timeout: Option<u64>: an optional timeout to wait in milliseconds. If unspecified SHOULD
    ///                         wait indefinetely
    ///
    /// # Returns
    /// Serialized received data or None if timed out waiting the remote
    ///
    async fn receive_raw(&mut self, timeout: Option<u64>) -> Option<Serialized>;

    ///
    /// Sends a serializable object over transport
    ///
    /// # Template arguments
    /// * T: Serializable + Send: type of object to send
    ///
    /// # Arguments
    /// * object: T: object itself
    ///
    #[inline]
    async fn send<T: Serializable + Send>(&mut self, object: T) -> Result<usize, tokio::io::Error>{
        self.send_raw(object.serialize()).await
    }

    async fn receive<T: Deserializable>(&mut self,
                                        timeout: Option<u64>) -> Option<Result<T, SerializationError>>{
        let serialized_data = self.receive_raw(timeout).await;
        if serialized_data.is_none(){
            return None;
        }
        let result = T::from_serialized(&serialized_data.unwrap());
        if result.is_err(){
            return Some(Err(result.err().unwrap()));
        }
        let (obj, _) = result.unwrap();
        Some(Ok(obj))
    }

    ///
    /// Adds a transformer to a current transport.
    ///
    /// # Arguments
    /// * transformer: Box<dyn Transformer>: a boxed transformer trait object
    ///
    /// # Returns
    /// Updated transport instance
    fn add_transformer<'a>(&'a mut self, transformer: Box<dyn TransportTransformer>) -> &'a Self;
}
