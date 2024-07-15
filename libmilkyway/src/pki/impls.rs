use libmilkyway_derive::{EnumDeserializable, EnumSerializable};
use crate::serialization::serializable::{Serialized, Serializable};
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;

pub mod keys;
pub mod certificates;
pub mod hashable;

///
/// Crypto alogrithm type
///
#[derive(PartialEq, Debug, Clone, EnumSerializable, EnumDeserializable)]
pub enum CryptoType {
    Falcon1024,
    Kyber1024Aes256GCM,
    Aes256GCM,
}


///
/// Encryption errors
///
#[derive(PartialEq, Debug, Clone)]
pub enum CryptoError {
    ///
    /// Can not decrypt data, probably something nasty is going on
    ///
    DataTampered,

    ///
    /// Something wrong with format of data(e.g. data headers are missing)
    ///
    FormatError,

    ///
    /// Argument error(e.g. wrong certificate type)
    ///
    ArgumentError(&'static str),
}
