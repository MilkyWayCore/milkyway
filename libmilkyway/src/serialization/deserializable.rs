use crate::serialization::serializable::Serialized;
use crate::serialization::error::SerializationError;

///
/// The structure which may can be created from Serialized data.
///
pub trait Deserializable: Sized {
    ///
    /// Creates structure from serialized data.
    /// Returns either pair (result, offset) or error.
    /// # Arguments
    ///
    /// * `serialized`: Serialized data to make struct from
    ///
    /// returns: Result<(Self, usize), SerializationError>
    ///
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError>;
}