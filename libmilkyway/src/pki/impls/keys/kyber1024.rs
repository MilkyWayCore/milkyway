use aes_gcm::aead::generic_array::GenericArray;
use aes_gcm::Aes256Gcm;
use pqcrypto::kem::kyber1024;
use pqcrypto::traits::kem::{Ciphertext, PublicKey, SecretKey, SharedSecret};
use crate::pki::hash::{CryptoHashable, HashType};
use crate::pki::impls::{CryptoError, CryptoType};
use crate::pki::key::{CryptoKey, KeyType};
use crate::pki::signature::Signature;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};

impl Serializable for kyber1024::PublicKey{
    #[inline]
    fn serialize(&self) -> Serialized {
        self.as_bytes().to_vec().serialize()
    }
}

impl Deserializable for kyber1024::PublicKey{
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let deserialized_bytes_result = Vec::<u8>::from_serialized(serialized);
        if deserialized_bytes_result.is_err(){
            return Err(deserialized_bytes_result.err().unwrap());
        }
        let (deserialized_bytes, offset) = deserialized_bytes_result.unwrap();
        let key_result = kyber1024::PublicKey::from_bytes(&deserialized_bytes);
        if key_result.is_err(){
            return Err(SerializationError::InvalidDataError("Invalid Kyber1024 public key"));
        }
        Ok((key_result.unwrap(), offset))
    }
}

impl Serializable for kyber1024::SecretKey{
    #[inline]
    fn serialize(&self) -> Serialized {
        self.as_bytes().to_vec().serialize()
    }
}
impl Deserializable for kyber1024::SecretKey{
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let deserialized_bytes_result = Vec::<u8>::from_serialized(serialized);
        if deserialized_bytes_result.is_err(){
            return Err(deserialized_bytes_result.err().unwrap());
        }
        let (deserialized_bytes, offset) = deserialized_bytes_result.unwrap();
        let key_result = kyber1024::SecretKey::from_bytes(&deserialized_bytes);
        if key_result.is_err(){
            return Err(SerializationError::InvalidDataError("Invalid Kyber1024 secret key"));
        }
        Ok((key_result.unwrap(), offset))
    }
}

impl CryptoKey for kyber1024::PublicKey {
    #[inline]
    fn get_key_type(&self) -> KeyType {
        KeyType::Public
    }

    #[inline]
    fn get_crypto_type(&self) -> CryptoType {
        CryptoType::Kyber1024Aes256GCM
    }

    fn encrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError> {
        let (shared_secret, cipher_text) = kyber1024::encapsulate(self);
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&shared_secret.as_bytes()[..32]);
        let mut result = Serialized::new();
        //FIXME: Do we really need cipher text every time in encryption?
        result.extend(cipher_text.as_bytes().to_vec().serialize());
        let encryption_result = key.encrypt_raw(data);
        if encryption_result.is_err(){
            // It is already an error
            return encryption_result;
        }
        result.extend(encryption_result.unwrap().serialize());
        Ok(result)
    }

    fn decrypt_raw(&self, _data: &Serialized) -> Result<Serialized, CryptoError> {
        panic!("Public key can not be used for decryption");
    }

    fn sign<T: Serializable + CryptoHashable>(&self, _data: &T, _hash_type: HashType) -> Result<Signature, CryptoError> {
        panic!("Kyber1024 can not be used for digital signature");
    }

    fn verify_signature<T: Serializable + CryptoHashable>(&self, _data: &T, _signature: &Signature) -> bool {
        panic!("Kyber1024 can not be used for digital signature");
    }
}

impl CryptoKey for kyber1024::SecretKey {
    #[inline]
    fn get_key_type(&self) -> KeyType {
        KeyType::Private
    }

    #[inline]
    fn get_crypto_type(&self) -> CryptoType {
        CryptoType::Kyber1024Aes256GCM
    }

    fn encrypt_raw(&self, _data: &Serialized) -> Result<Serialized, CryptoError> {
        panic!("Private key can not be used for encryption");
    }

    fn decrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError> {
        let deserialized_data = Vec::<u8>::from_serialized(data);
        if deserialized_data.is_err(){
            return Err(CryptoError::FormatError);
        }
        let (cipher_text_bytes, offset) = deserialized_data.unwrap();
        let cipher_text_result =
            kyber1024::Ciphertext::from_bytes(&cipher_text_bytes);
        let cipher_text = cipher_text_result.unwrap();
        let shared_secret = kyber1024::decapsulate(&cipher_text, self);
        let key = GenericArray::from_slice(&shared_secret.as_bytes());
        let encrypted_data_result = Vec::<u8>::from_serialized(
            &data[offset..].to_vec());
        if encrypted_data_result.is_err(){
            return Err(CryptoError::FormatError)
        }
        let (encrypted_data, _encrypted_data_offset) = encrypted_data_result.unwrap();
        key.decrypt_raw(&encrypted_data)
    }

    fn sign<T: Serializable + CryptoHashable>(&self, _data: &T, _hash_type: HashType) -> Result<Signature, CryptoError> {
        panic!("Kyber1024 can not be used for digital signature");
    }

    fn verify_signature<T: Serializable + CryptoHashable>(&self, _data: &T, _signature: &Signature) -> bool {
        panic!("Kyber1024 can not be used for digital signature");
    }
}

#[inline]
pub fn generate_kyber1024_keypair() -> (kyber1024::PublicKey, kyber1024::SecretKey) {
    kyber1024::keypair()
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use pqcrypto::kem::kyber1024;
    use libmilkyway_derive::{Deserializable, Serializable};

    #[derive(Clone, Debug, Serializable, Deserializable, PartialEq)]
    pub struct TestStruct{
        data: Vec<u8>
    }
    #[test]
    fn test_serialize_deserialize_kyber1024_public_key() {
        let (public_key, _secret_key) = kyber1024::keypair();
        let serialized = public_key.serialize();
        let (deserialized, size) = kyber1024::PublicKey::from_serialized(&serialized).unwrap();
        assert!(public_key == deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_kyber1024_secret_key() {
        let (_public_key, secret_key) = kyber1024::keypair();
        let serialized = secret_key.serialize();
        let (deserialized, size) = kyber1024::SecretKey::from_serialized(&serialized).unwrap();
        assert!(secret_key == deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_encrypt_decrypt_kyber1024_aes256gcm() {
        let (public_key, secret_key) = kyber1024::keypair();
        let data = b"secret datafj".to_vec().serialize();

        let encrypted_data = public_key.encrypt_raw(&data).unwrap();
        let decrypted_data = secret_key.decrypt_raw(&encrypted_data).unwrap();

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_encrypt_decrypt_highlevel_kyber1024_aes256gcm(){
        let (public_key, secret_key) = kyber1024::keypair();
        let data = TestStruct {
            data: "A quick brown fox jumps over a lazy dog".as_bytes().to_vec(),
        };
        let encrypted_data = public_key.encrypt(&data).unwrap();
        println!("encrypted data: {:?}\n raw data: {:?}\n", encrypted_data, data.serialize());
        let decrypted_data = secret_key.decrypt::<TestStruct>(&encrypted_data).unwrap();
        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_invalid_encryption_with_private_key() {
        let (_public_key, secret_key) = kyber1024::keypair();
        let data = b"secret datax".to_vec();

        let result = std::panic::catch_unwind(|| {
            secret_key.encrypt_raw(&data.serialize()).unwrap();
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_decryption_with_public_key() {
        let (public_key, _secret_key) = kyber1024::keypair();
        let data = b"secret data".to_vec();

        let result = std::panic::catch_unwind(|| {
            public_key.decrypt_raw(&data.serialize()).unwrap();
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_kyber1024_encryption_decryption_with_signature() {
        let (public_key, secret_key) = kyber1024::keypair();
        let data = b"secret data".to_vec().serialize();
        let encrypted_data = public_key.encrypt_raw(&data).unwrap();
        let decrypted_data = secret_key.decrypt_raw(&encrypted_data).unwrap();
        assert_eq!(data, decrypted_data);
    }
}