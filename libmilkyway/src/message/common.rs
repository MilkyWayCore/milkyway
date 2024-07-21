use crate::serialization::error::SerializationError;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::serializable::Serializable;
use libmilkyway_derive::{Deserializable, Serializable};
use crate::get_timestamp_with_milliseconds;
use crate::message::types::MessageType;
use crate::pki::hash::HashType;
use crate::pki::key::CryptoKey;
use crate::pki::signature::Signature;
use crate::serialization::serializable::Serialized;

///
/// A common message structure which may contain other messages
///
#[derive(Clone, Serializable, Deserializable, PartialEq)]
pub struct Message{
    pub id: u128,
    pub timestamp: u128,
    pub message_type: MessageType,
    pub data: Option<Serialized>,
    pub signature: Option<Signature>,
    pub source: u128,
    pub destination: u128,
    pub module_id: u64,
}

impl<'a> Message {
    ///
    /// Builder-like function for setting id of message.
    ///
    /// # Arguments
    ///
    /// * id: ID of message to set
    ///
    pub fn set_id(&'a mut self, id: u128) -> &'a mut Message{
        self.id = id;
        self
    }

    ///
    /// Sets specified timestamp and returns update reference to same message
    ///
    /// # Arguments
    /// * timestamp: specific timestamp to set for message
    ///
    #[inline]
    pub fn set_timestamp(&'a mut self, timestamp: u128) -> &'a mut Message{
        self.timestamp = timestamp;
        self
    }

    ///
    /// Builder-like function to set a timestamp for message.
    ///
    #[inline]
    pub fn set_current_timestamp(&'a mut self) -> &'a mut Message{
        self.set_timestamp(get_timestamp_with_milliseconds())
    }

    ///
    /// Clones and strips signature, allowing to sign/verify signature
    ///
    pub fn as_signable(&self) -> Message{
        let mut m_copy = self.clone();
        m_copy.signature = None;
        m_copy
    }

    ///
    /// Builder-like signing function.
    ///
    /// # Arguments
    /// * key: CryptoKey: A key capable of signing data
    /// * hash_type: HashType: A type of hash to use for signature
    ///
    pub fn sign<T: CryptoKey>(&'a mut self, key: &T, hash_type: HashType) -> &'a Message{
        let signature = key.sign(&self.as_signable(), hash_type);
        self.signature = Some(signature.unwrap());
        self
    }

    ///
    /// Function
    pub fn verify_signature<T: CryptoKey>(&'a mut self, key: &T) -> bool{
        let signature = self.signature.clone().unwrap();
        key.verify_signature(&self.as_signable(), &signature)
    }

    ///
    /// Builder-like function to set a source of message
    ///
    /// # Arguments
    /// * source: u128: ID of source
    ///
    #[inline]
    pub fn set_source(&'a mut self, source: u128) -> &'a Message{
        self.source = source;
        self
    }

    ///
    /// Builder-like function to set a destination of message
    ///
    /// # Arguments
    /// * destination: u128: ID of destination
    ///
    #[inline]
    pub fn set_destination(&'a mut self, destination: u128) -> &'a Message{
        self.destination = destination;
        self
    }

    ///
    /// Clones message without signature, which allows to verify whole-message signature
    ///
    /// returns: cloned Message with signature equal to None
    ///
    #[inline]
    pub fn clone_without_signature(&self) -> Message{
        let mut cloned = self.clone();
        cloned.signature = None;
        cloned
    }
}

pub trait AsMessage{
    ///
    /// Converts struct to Message struct.
    /// Any fields which are unknown from current struct MUST be
    /// field with reasonable defaults.
    ///
    fn as_message(&self) -> Message;
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::pki::hash::CryptoHashable;
    use crate::pki::impls::{CryptoError, CryptoType};
    use crate::pki::key::KeyType;
    
    #[derive(Debug, PartialEq, Clone, Serializable, Deserializable)]
    struct TestKey;

    impl CryptoKey for TestKey {
        fn get_key_type(&self) -> KeyType {
            KeyType::Symmetric
        }

        fn get_crypto_type(&self) -> CryptoType {
            CryptoType::Aes256GCM
        }

        fn encrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError> {
            Ok(data.clone())
        }

        fn decrypt_raw(&self, data: &Serialized) -> Result<Serialized, CryptoError> {
            Ok(data.clone())
        }

        fn sign<T: Serializable + CryptoHashable>(&self, data: &T, _hash_type: HashType) -> Result<Signature, CryptoError> {
            Ok(Signature {
                algorithm: HashType::SHA512,
                crypto_algorithm: CryptoType::Aes256GCM,
                serialized_signature: data.serialize(),
            })
        }

        fn verify_signature<T: Serializable + CryptoHashable>(&self, data: &T, signature: &Signature) -> bool {
            data.serialize() == signature.serialized_signature
        }
    }

    fn get_timestamp_with_milliseconds() -> u128 {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis()
    }

    #[test]
    fn test_set_id() {
        let mut message = Message {
            id: 0,
            timestamp: 0,
            message_type: MessageType::LogMessage,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        };
        message.set_id(12345);
        assert_eq!(message.id, 12345);
    }

    #[test]
    fn test_set_timestamp() {
        let mut message = Message {
            id: 0,
            timestamp: 0,
            message_type: MessageType::LogMessage,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        };
        message.set_timestamp(1234567890);
        assert_eq!(message.timestamp, 1234567890);
    }

    #[test]
    fn test_set_current_timestamp() {
        let mut message = Message {
            id: 0,
            timestamp: 0,
            message_type: MessageType::LogMessage,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        };
        message.set_current_timestamp();
        assert!(message.timestamp > 0);
    }

    #[test]
    fn test_set_source() {
        let mut message = Message {
            id: 0,
            timestamp: 0,
            message_type: MessageType::LogMessage,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        };
        message.set_source(42);
        assert_eq!(message.source, 42);
    }

    #[test]
    fn test_set_destination() {
        let mut message = Message {
            id: 0,
            timestamp: 0,
            message_type: MessageType::LogMessage,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        };
        message.set_destination(84);
        assert_eq!(message.destination, 84);
    }

    #[test]
    fn test_sign_and_verify_signature() {
        let mut message = Message {
            id: 0,
            timestamp: 0,
            message_type: MessageType::LogMessage,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        };
        let key = TestKey;

        message.sign(&key, HashType::SHA512);
        assert!(message.signature.is_some());

        let is_valid = message.verify_signature(&key);
        assert!(is_valid);
    }

    #[test]
    fn test_sign_and_verify_signature_with_tampering() {
        let mut message = Message {
            id: 0,
            timestamp: 0,
            message_type: MessageType::LogMessage,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        };
        let key = TestKey;

        message.sign(&key, HashType::SHA512);
        assert!(message.signature.is_some());

        // Tamper with the message
        message.set_id(99999);

        let is_valid = message.verify_signature(&key);
        assert!(!is_valid);
    }

    #[test]
    fn test_message_serialization_deserialization() {
        let message = Message {
            id: 12345,
            timestamp: get_timestamp_with_milliseconds(),
            message_type: MessageType::LogMessage,
            data: Some(vec![1, 2, 3, 4].serialize()),
            signature: None,
            source: 42,
            destination: 84,
            module_id: 0,
        };

        let serialized = message.serialize();
        let (deserialized, _) = Message::from_serialized(&serialized).unwrap();
        assert!(message == deserialized);
    }
}