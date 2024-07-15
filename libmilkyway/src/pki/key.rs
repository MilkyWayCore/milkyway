use crate::pki::hash::{CryptoHashable, Hash, HashType};
use crate::pki::impls::{CryptoError, CryptoType};
use crate::pki::signature::Signature;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::error::SerializationError::CryptographicError;
use crate::serialization::serializable::{Serializable, Serialized};

#[derive(PartialEq, Debug)]
pub enum KeyType{
    Symmetric,
    Private,
    Public,
}

///
/// Key abstraction
///
pub trait CryptoKey: Serializable + Deserializable{
    ///
    /// Gets type of key
    ///
    /// returns: KeyType: type of key
    ///
    fn get_key_type(&self) -> KeyType;

    ///
    /// Gets algorithm type
    ///
    /// returns: CryptoType: type of encipherment algorithm
    ///
    fn get_crypto_type(&self) -> CryptoType;

    ///
    ///
    /// # Arguments
    ///
    /// * `data`: serialized data to encrypt
    ///
    /// returns: Serialized: serialized data
    ///
    fn encrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError>;

    ///
    /// Encrypts data and serializes it.
    ///
    /// # Arguments
    ///
    /// * `data`: Serializable data to encrypt
    ///
    /// returns: Serialized: serialized encrypted data
    ///
    #[inline]
    fn encrypt<T: Serializable>(&self, data: &T) -> Result<Serialized, CryptoError>{
        let data_serialized = data.serialize();
        self.encrypt_raw(&data_serialized)
    }

    ///
    /// Decrypts data and returns decrypted Serialized data
    ///
    /// # Arguments
    ///
    /// * `data`: &Serialized: data to decrypt and deserialize
    ///
    /// returns: T: deserialized and decrypted structure
    fn decrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError>;

    ///
    /// Decrypts and deserializes data
    ///
    /// # Arguments
    ///
    /// * `data`: &Serialized: data to decrypt and deserialize
    ///
    /// returns: T: deserialized and decrypted structure
    #[inline]
    fn decrypt<T: Deserializable>(&self, data: &Serialized) -> Result<T, SerializationError>{
        let decrypted_data_result = self.decrypt_raw(data);
        if decrypted_data_result.is_err(){
            return Err(CryptographicError(decrypted_data_result.err().unwrap()));
        }
        let decrypted_data = decrypted_data_result.unwrap();
        //println!("Decrypted data: {:?}", decrypted_data);
        let deserialization_result = T::from_serialized(&decrypted_data);
        if deserialization_result.is_err(){
            Err(deserialization_result.err().unwrap())
        } else {
            let (data, _) = deserialization_result.unwrap();
            Ok(data)
        }
    }

    ///
    /// Signs data with key
    ///
    ///
    /// # Template arguments
    /// * `T`: Serializable + CryptoHashable: data which can be serialized and hashed
    ///
    /// # Arguments
    /// * `data`: &T: data for which hash may be computed
    /// * `hash_type`: HashType: hashing algorithm type
    ///
    /// returns:
    fn sign<T: Serializable + CryptoHashable>(&self, data: &T, hash_type: HashType) -> Result<Signature, CryptoError>{
        let m_type = self.get_key_type();
        if m_type != KeyType::Private {
            panic!("Signing data with non-private key");
        }
        let hash = data.crypto_hash(hash_type.clone());
        let encrypted = self.encrypt(&hash);
        if encrypted.is_err(){
            return Err(encrypted.err().unwrap());
        }
        Ok(Signature {
            algorithm: hash_type,
            crypto_algorithm: self.get_crypto_type(),
            serialized_signature: encrypted.unwrap(),
        })
    }

    ///
    /// Verifies signature of data against key
    ///
    /// # Arguments
    /// * `data`: &dyn CryptoHashable: data for which hash may be computed
    /// * `signature`: the signature against data must be verified
    ///
    /// returns: bool: wether the signature is valid for data
    ///
    fn verify_signature<T: Serializable + CryptoHashable>(&self, data: &T, signature: &Signature) -> bool{
        let m_type = self.get_key_type();
        let m_crypto_type = self.get_crypto_type();
        if m_type != KeyType::Public{
            panic!("Verifying data signature with non-public key");
        }
        if m_crypto_type != signature.crypto_algorithm{
            panic!("Verifying data signature with wrong algorithm");
        }
        let data_hash = data.crypto_hash(signature.algorithm.clone());
        let original_hash = self.decrypt::<Hash>(&signature.serialized_signature);
        original_hash.unwrap() == data_hash
    }
}