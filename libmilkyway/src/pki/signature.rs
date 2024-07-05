use crate::serialization::error::SerializationError;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::serializable::Serializable;
use libmilkyway_derive::{Deserializable, Serializable};
use crate::pki::hash::HashType;
use crate::pki::impls::CryptoType;
use crate::serialization::serializable::Serialized;


///
/// Signature with metadata
///
#[derive(Clone, Serializable, Deserializable, PartialEq)]
pub struct Signature {
    pub algorithm: HashType,
    pub crypto_algorithm: CryptoType,
    pub serialized_signature: Serialized,
}