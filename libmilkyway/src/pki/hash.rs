use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};

///
/// Type of hashing algorithm to use
///
#[derive(Clone, PartialEq, Debug)]
pub enum HashType {
    ///
    /// Used for algorithms which are strictly require own hashing
    ///
    None,
    ///
    /// Traditional SHA512 hash
    ///
    SHA512,
}


impl Serializable for HashType {
    fn serialize(&self) -> Serialized {
        let tp: u8 = match self {
            HashType::None => { 0 },
            HashType::SHA512 => { 1 }
        };
        tp.serialize()
    }
}

impl Deserializable for HashType {
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let tp: u8 = serialized[0];
        match tp {
            0 => { Ok((HashType::None, 1))},
            1 => { Ok((HashType::SHA512, 1)) }
            _ => Err(SerializationError::InvalidDataError("Unknown type of hash"))
        }
    }
}

///
/// Hash structure
/// * algorithm: Algorithm which was used for hashing
/// * hash: A serialized version of hash
///
#[derive(Clone, PartialEq, Debug)]
pub struct Hash {
    pub algorithm: HashType,
    pub hash: Vec<u8>,
}

impl Serializable for Hash {
    fn serialize(&self) -> Serialized {
        let mut result = Serialized::new();
        result.extend(self.algorithm.serialize());
        result.extend(self.hash.serialize());
        result
    }
}

impl Deserializable for Hash {
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let mut offset = 0;
        let algorithm_result = HashType::from_serialized(serialized);
        if algorithm_result.is_err(){
            return Err(algorithm_result.err().unwrap());
        }
        let (algorithm, algorithm_offset) = algorithm_result.unwrap();
        offset += algorithm_offset;
        let hash_data_result = Vec::<u8>::from_serialized(&serialized[offset..].to_vec());
        if hash_data_result.is_err(){
            return Err(hash_data_result.err().unwrap());
        }
        let (hash_data, hash_data_offset) = hash_data_result.unwrap();
        offset += hash_data_offset;
        Ok((Hash {
            algorithm,
            hash: hash_data,
        }, offset))
    }
}

///
/// Hashable type abstraction
///
pub trait CryptoHashable {
    fn crypto_hash(&self, hash_type: HashType) -> Hash;
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_hashtype_sha512() {
        let hash_type = HashType::SHA512;
        let serialized = hash_type.serialize();
        let (deserialized, size) = HashType::from_serialized(&serialized).unwrap();
        assert_eq!(hash_type, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_invalid_data_error_hashtype() {
        let serialized = vec![255u8]; // Invalid hash type
        let result = HashType::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::InvalidDataError(_))));
    }

    #[test]
    fn test_serialize_deserialize_hash() {
        let hash = Hash {
            algorithm: HashType::SHA512,
            hash: vec![1, 2, 3, 4, 5],
        };
        let serialized = hash.serialize();
        let (deserialized, size) = Hash::from_serialized(&serialized).unwrap();
        assert_eq!(hash, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_empty_hash() {
        let hash = Hash {
            algorithm: HashType::SHA512,
            hash: vec![],
        };
        let serialized = hash.serialize();
        let (deserialized, size) = Hash::from_serialized(&serialized).unwrap();
        assert_eq!(hash, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_length_error_hash() {
        let serialized = vec![0u8]; // Only includes the hash type, no hash data
        let result = Hash::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::LengthError)));
    }
}