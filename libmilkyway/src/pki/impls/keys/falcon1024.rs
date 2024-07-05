use pqcrypto::traits::sign::{PublicKey, SecretKey, SignedMessage};
use pqcrypto_falcon::falcon1024;
use crate::pki::hash::{CryptoHashable, HashType};
use crate::pki::impls::{CryptoError, CryptoType};
use crate::pki::key::{CryptoKey, KeyType};
use crate::pki::signature::Signature;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};

#[derive(PartialEq, Clone)]
pub struct Falcon1024PublicKey {
    pub(crate) internal: falcon1024::PublicKey,
}

#[derive(PartialEq, Clone)]
pub struct Falcon1024SecretKey {
    pub(crate) internal: falcon1024::SecretKey,
}

///
/// Generates Falcon1024 keypair
///
pub fn generate_falcon1024_keypair() -> (Falcon1024PublicKey, Falcon1024SecretKey) {
    let (pk_internal, sk_internal) = falcon1024::keypair();
    let pk = Falcon1024PublicKey {
        internal: pk_internal,
    };
    let sk = Falcon1024SecretKey {
        internal: sk_internal,
    };
    (pk, sk)
}

impl Serializable for Falcon1024SecretKey {
    #[inline]
    fn serialize(&self) -> Serialized {
       self.internal.as_bytes().to_vec().serialize()
    }
}

impl Deserializable for Falcon1024SecretKey {
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let result = Vec::<u8>::from_serialized(serialized);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let (raw_key, offset) = result.unwrap();
        let key = falcon1024::SecretKey::from_bytes(&raw_key);
        if key.is_err() {
            return Err(SerializationError::InvalidDataError("Wrong bytes for falcon1024 key"));
        }
        let key = key.unwrap();
        let key = Falcon1024SecretKey {
            internal: key,
        };
        Ok((key, offset))
    }
}


impl Serializable for falcon1024::SignedMessage {
    #[inline]
    fn serialize(&self) -> Serialized {
        self.clone().as_bytes().to_vec().serialize()
    }
}

impl Deserializable for falcon1024::SignedMessage {
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let result = Vec::<u8>::from_serialized(serialized);
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let (result_bytes, offset) = result.unwrap();
        let message = falcon1024::SignedMessage::from_bytes(&result_bytes);
        if message.is_err(){
            return Err(SerializationError::InvalidDataError("Can not create SignedMessage from bytes"));
        }
        let message = message.unwrap();
        Ok((message, offset))
    }
}


impl CryptoKey for Falcon1024SecretKey {
    #[inline]
    fn get_key_type(&self) -> KeyType {
        KeyType::Private
    }

    #[inline]
    fn get_crypto_type(&self) -> CryptoType {
        CryptoType::Falcon1024
    }

    fn encrypt_raw(&self, _data: &Serialized) -> Result<Serialized, CryptoError> {
        panic!("Falcon1024 can not be used for encipherment");
    }

    fn decrypt_raw(&self, _data: &Serialized) -> Result<Serialized, CryptoError> {
        panic!("Falcon1024 can not be used for decipherment");
    }

    fn sign<T: Serializable + CryptoHashable>(&self, data: &T, _hash_type: HashType) -> Result<Signature, CryptoError> {
        if _hash_type != HashType::None {
            panic!("Falcon1024 uses own hashing. hash_type must be None");
        }
        let signed_message = falcon1024::sign(&data.serialize(), &self.internal);
        Ok(Signature {
            algorithm: HashType::None,
            crypto_algorithm: CryptoType::Falcon1024,
            serialized_signature: signed_message.serialize(),
        })
    }

    fn verify_signature<T: Serializable + CryptoHashable>(&self, _data: &T,
                                                          _signature: &Signature) -> bool {
        panic!("Can not verify signature with Falcon1024 secret key");
    }
}



impl Serializable for Falcon1024PublicKey {
    fn serialize(&self) -> Serialized {
        self.internal.as_bytes().to_vec().serialize()
    }
}

impl Deserializable for Falcon1024PublicKey {
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let result = Vec::<u8>::from_serialized(serialized);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let (raw_key, offset) = result.unwrap();
        let key = falcon1024::PublicKey::from_bytes(&raw_key);
        if key.is_err() {
            return Err(SerializationError::InvalidDataError("Wrong bytes for falcon1024 key"));
        }
        let key = key.unwrap();
        let key = Falcon1024PublicKey {
            internal: key,
        };
        Ok((key, offset))
    }
}

impl CryptoKey for Falcon1024PublicKey {
    fn get_key_type(&self) -> KeyType {
        KeyType::Public
    }

    fn get_crypto_type(&self) -> CryptoType {
        CryptoType::Falcon1024
    }

    fn encrypt_raw(&self, _data: &Serialized) -> Result<Serialized, CryptoError> {
        panic!("Falcon1024 can not be used for encipherment");
    }

    fn decrypt_raw(&self, _data: &Serialized) -> Result<Serialized, CryptoError> {
        panic!("Falcon1024 can not be used for decipherment");
    }

    fn verify_signature<T: Serializable + CryptoHashable>(&self, data: &T, signature: &Signature) -> bool {
        let signed_message_result = falcon1024::SignedMessage::from_serialized(
            &signature.serialized_signature);
        if signed_message_result.is_err(){
            return false;
        }
        let (signed_message, _) = signed_message_result.unwrap();
        let verified_msg = falcon1024::open(&signed_message,
                                           &self.internal);
        if verified_msg.is_err(){
            return false;
        }
        let serialized_msg = data.serialize();
        serialized_msg == verified_msg.unwrap()
    }
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use pqcrypto_falcon::falcon1024;
    use crate::pki::hash::Hash;

    #[test]
    fn test_generate_falcon1024_keypair() {
        let (pk, sk) = generate_falcon1024_keypair();
        assert_eq!(pk.get_key_type(), KeyType::Public);
        assert_eq!(sk.get_key_type(), KeyType::Private);
    }

    #[test]
    fn test_serialize_deserialize_falcon1024_public_key() {
        let (pk, _sk) = generate_falcon1024_keypair();
        let serialized = pk.serialize();
        let (deserialized, size) =
            Falcon1024PublicKey::from_serialized(&serialized).unwrap();
        assert!(pk == deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_falcon1024_secret_key() {
        let (_pk, sk) = generate_falcon1024_keypair();
        let serialized = sk.serialize();
        let (deserialized, size) = Falcon1024SecretKey::from_serialized(&serialized).unwrap();
        assert!(sk == deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_sign_verify_signature_falcon1024() {
        let (pk, sk) = generate_falcon1024_keypair();
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        
        let signature = sk.sign(&data, HashType::None).unwrap();
        let is_valid = pk.verify_signature(&data, &signature);
        assert!(is_valid);
    }

    #[test]
    fn test_verify_signature_falcon1024_invalid_data() {
        let (pk, sk) = generate_falcon1024_keypair();
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        let invalid_data: Vec<u8> = vec![6, 7, 8, 9, 10];
        let signature = sk.sign(&data, HashType::None).unwrap();
        let is_valid = pk.verify_signature(&invalid_data, &signature);
        assert!(!is_valid);
    }

    #[test]
    fn test_serialize_deserialize_signed_message() {
        let data = vec![1, 2, 3, 4, 5];
        let (_pk, sk) = generate_falcon1024_keypair();
        let signed_message = falcon1024::sign(&data, &sk.internal);

        let serialized = signed_message.serialize();
        let (deserialized, size) = falcon1024::SignedMessage::from_serialized(&serialized).unwrap();
        assert_eq!(signed_message.as_bytes().to_vec(), deserialized.as_bytes().to_vec());
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_invalid_signature_verification() {
        let (pk, sk) = generate_falcon1024_keypair();
        let data = vec![1, 2, 3, 4, 5];

        let signature = sk.sign(&data, HashType::None);
        // Tamper with the signature to make it invalid
        let mut tampered_signature = signature.clone().unwrap();
        tampered_signature.serialized_signature[7] ^= 0xFF;

        let is_valid = pk.verify_signature(&data, &tampered_signature);
        assert!(!is_valid);
    }
}

