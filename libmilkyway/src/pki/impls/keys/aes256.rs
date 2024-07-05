use aes_gcm::{AeadCore, Aes256Gcm, AesGcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use aes_gcm::aead::generic_array::GenericArray;
use rand::rngs::OsRng;
use crate::pki::impls::{CryptoError, CryptoType};
use crate::pki::key::{CryptoKey, KeyType};
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};

impl Serializable for aes_gcm::Key<Aes256Gcm>{
    #[inline]
    fn serialize(&self) -> Serialized {
        self.to_vec().serialize()
    }
}

impl Deserializable for aes_gcm::Key<Aes256Gcm>{
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let result = Vec::<u8>::from_serialized(serialized);
        if result.is_err(){
            return Err(result.err().unwrap());
        }
        let (key, offset) = result.unwrap();
        Ok((*Self::from_slice(&key), offset))
    }
}

impl CryptoKey for aes_gcm::Key<Aes256Gcm>{
    #[inline]
    fn get_key_type(&self) -> KeyType {
        KeyType::Symmetric
    }

    #[inline]
    fn get_crypto_type(&self) -> CryptoType {
        CryptoType::Aes256GCM
    }

    fn encrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError> {
        let cipher = Aes256Gcm::new(self);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let mut result = Serialized::new();
        result.extend(nonce.to_vec().serialize());
        let ciphertext = cipher.encrypt(&nonce, data.as_ref()).unwrap();
        result.extend(ciphertext.serialize());
        Ok(result)
    }

    fn decrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError> {
        let cipher = Aes256Gcm::new(self);
        let nonce_data_result = Vec::<u8>::from_serialized(data);
        if nonce_data_result.is_err(){

        }
        let (nonce_data, offset) = nonce_data_result.unwrap();
        let nonce = Nonce::from_slice(&nonce_data);
        let (ciphertext, _) = Vec::<u8>::from_serialized(&data[offset..].to_vec())
            .unwrap();
        let decryption_result = cipher.decrypt(nonce, ciphertext.as_ref());
        if decryption_result.is_err(){
            return Err(CryptoError::DataTampered);
        }
        Ok(decryption_result.unwrap().to_vec())
    }
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::{Aes256Gcm, Key, Nonce};
    use aes_gcm::aead::{Aead, KeyInit};
    use rand::rngs::OsRng;

    #[test]
    fn test_serialize_deserialize_aes256gcm_key() {
        let key = Aes256Gcm::generate_key(OsRng);
        let serialized = key.serialize();
        let (deserialized, size) = Key::<Aes256Gcm>::from_serialized(&serialized).unwrap();
        assert_eq!(key, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_encrypt_decrypt_aes256gcm() {
        let key = Aes256Gcm::generate_key(OsRng);;
        let data = b"secret data".to_vec().serialize();

        let encrypted_data = key.encrypt_raw(&data).unwrap();
        let decrypted_data = key.decrypt_raw(&encrypted_data).unwrap();

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_encrypt_decrypt_aes256gcm_with_nonce() {
        let key = Aes256Gcm::generate_key(OsRng);;
        let data = b"secret data".to_vec();
        let cipher = Aes256Gcm::new(&key);
        let nonce = Nonce::from_slice(b"unique nonce");

        let encrypted_data = cipher.encrypt(nonce, data.as_ref()).unwrap();
        let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref()).unwrap();

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_serialize_deserialize_with_encryption() {
        let key = Aes256Gcm::generate_key(OsRng);;
        let data = b"secret datarfnodhgortihjgergjdrjsff".to_vec().serialize();

        let encrypted_data = key.encrypt_raw(&data).unwrap();
        let serialized_encrypted_data = encrypted_data.serialize();
        let (deserialized_encrypted_data, size) = Vec::<u8>::from_serialized(&serialized_encrypted_data).unwrap();

        let decrypted_data = key.decrypt_raw(&deserialized_encrypted_data).unwrap();

        assert_eq!(data, decrypted_data);
        assert_eq!(encrypted_data, deserialized_encrypted_data);
        assert_eq!(size, serialized_encrypted_data.len());
    }

    #[test]
    fn test_invalid_decryption() {
        let key = Aes256Gcm::generate_key(OsRng);
        let data = b"secret data".to_vec();

        let encrypted_data = key.encrypt_raw(&data.serialize());

        let mut tampered_encrypted_data = encrypted_data.unwrap().clone();
        tampered_encrypted_data[0] ^= 0xff; // Tamper with the encrypted data

        let result = std::panic::catch_unwind(|| {
            key.decrypt_raw(&tampered_encrypted_data).unwrap();
        });

        assert!(result.is_err());
    }
}