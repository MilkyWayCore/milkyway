use crate::transport::Deserializable;
use crate::transport::Serializable;
use libmilkyway_derive::{Deserializable, Serializable};
use crate::pki::certificate::Certificate;
use crate::pki::hash::HashType;
use crate::pki::impls::certificates::falcon1024::Falcon1024Certificate;
use crate::pki::impls::certificates::kyber1024::Kyber1024Certificate;
use crate::pki::impls::CryptoError;
use crate::pki::signature::Signature;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::Serialized;
use crate::transport::TransportTransformer;

///
/// Transforms and detransforms encrypted and signed data
///
pub struct CryptoTransformer{
    local_signing_cert: Falcon1024Certificate,
    local_encryption_cert: Kyber1024Certificate,
    remote_signing_cert: Falcon1024Certificate,
    remote_encryption_cert: Kyber1024Certificate,
}

///
/// An encrypted message struct
///
#[derive(Serializable, Deserializable, Debug)]
pub struct CryptoMessage{
    signature: Signature,
    data: Serialized,
}

impl CryptoTransformer {
    #[inline]
    pub fn new(local_signing_cert: Falcon1024Certificate,
               local_encryption_cert: Kyber1024Certificate,
               remote_signing_cert: Falcon1024Certificate,
               remote_encryption_cert: Kyber1024Certificate) -> CryptoTransformer{
        CryptoTransformer{
            local_signing_cert,
            local_encryption_cert,
            remote_signing_cert,
            remote_encryption_cert,
        }
    }
}

impl TransportTransformer for CryptoTransformer{
    fn detransform(&self, data: &Serialized) -> Result<Serialized, SerializationError> {
        let message_result = CryptoMessage::from_serialized(data);
        if message_result.is_err(){
            return Err(message_result.err().unwrap());
        }
        let (message, _) = message_result.unwrap();
        if !self.remote_signing_cert.verify_signature(&message.data, &message.signature){
            return Err(SerializationError::CryptographicError(CryptoError::DataTampered));
        }
        let decrypted_data_result =
            self.local_encryption_cert.decrypt::<Vec<u8>>(&message.data);
        decrypted_data_result
    }

    fn transform(&self, data: &Serialized) -> Serialized {
        let encrypted_data = self.remote_encryption_cert.encrypt(data)
            .expect("Can not encrypt local packet");
        let signature = self.local_signing_cert
            .sign_data(&encrypted_data, HashType::None).expect("Can not sign local packet");
        let message = CryptoMessage{
            signature,
            data: encrypted_data,
        };
        message.serialize()
    }
}


/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pki::impls::certificates::falcon1024::Falcon1024Certificate;
    use crate::pki::impls::certificates::kyber1024::Kyber1024Certificate;
    use crate::serialization::serializable::{Serializable, Serialized};
    use crate::serialization::deserializable::Deserializable;
    use crate::serialization::error::SerializationError;
    use crate::pki::impls::CryptoError;
    use crate::pki::impls::keys::falcon1024::generate_falcon1024_keypair;
    use crate::pki::impls::keys::kyber1024::generate_kyber1024_keypair;
    use crate::transport::TransportTransformer;

    #[derive(Serializable, Deserializable, PartialEq, Debug)]
    struct TestData {
        message: String,
    }

    fn generate_falcon1024_certificate() -> Falcon1024Certificate {
        let (public_key, secret_key) = generate_falcon1024_keypair();
        Falcon1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key,
            signature: None,
            name: "test".to_string(),
            flags: 0,
        }
    }

    fn generate_kyber1024_certificate() -> Kyber1024Certificate {
        let (public_key, secret_key) = generate_kyber1024_keypair();
        Kyber1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key,
            signature: None,
            name: "test".to_string(),
            flags: 0,
        }
    }

    #[test]
    fn test_crypto_transformer_transform_and_detransform() {
        // Generate certificates
        let local_signing_cert = generate_falcon1024_certificate();
        let local_encryption_cert = generate_kyber1024_certificate();
        let remote_signing_cert = generate_falcon1024_certificate();
        let remote_encryption_cert = generate_kyber1024_certificate();

        // Initialize the CryptoTransformer
        let transformer = CryptoTransformer::new(
            local_signing_cert.clone(),
            local_encryption_cert.clone(),
            remote_signing_cert.clone_without_signature_and_sk(),
            remote_encryption_cert.clone_without_signature_and_sk(),
        );

        let detransformer = CryptoTransformer::new(
            remote_signing_cert.clone(),
            remote_encryption_cert.clone(),
            local_signing_cert.clone_without_signature_and_sk(),
            local_encryption_cert.clone_without_signature_and_sk(),
        );

        // Create test data
        let test_data = "Hello, world!".as_bytes().to_vec();
        let serialized_data = test_data.serialize();

        // Transform (encrypt and sign) the data
        let transformed_data = transformer.transform(&serialized_data);

        // Detransform (verify and decrypt) the data
        let detransformed_data = detransformer.detransform(&transformed_data).unwrap();
        let deserialized_data = Vec::<u8>::from_serialized(&detransformed_data).unwrap().0;

        // Ensure the original and detransformed data are the same
        assert_eq!(test_data, deserialized_data);
    }

    #[test]
    fn test_crypto_transformer_detransform_with_tampering() {
        // Generate certificates
        let local_signing_cert = generate_falcon1024_certificate();
        let local_encryption_cert = generate_kyber1024_certificate();
        let remote_signing_cert = generate_falcon1024_certificate();
        let remote_encryption_cert = generate_kyber1024_certificate();

        // Initialize the CryptoTransformer
        let transformer = CryptoTransformer::new(
            local_signing_cert.clone(),
            local_encryption_cert.clone(),
            remote_signing_cert.clone_without_signature_and_sk(),
            remote_encryption_cert.clone_without_signature_and_sk(),
        );

        let detransformer = CryptoTransformer::new(
            remote_signing_cert.clone(),
            remote_encryption_cert.clone(),
            local_signing_cert.clone_without_signature_and_sk(),
            local_encryption_cert.clone_without_signature_and_sk(),
        );

        // Create test data
        let test_data = TestData {
            message: "Hello, World!".to_string(),
        };
        let serialized_data = test_data.serialize();

        // Transform (encrypt and sign) the data
        let mut transformed_data = transformer.transform(&serialized_data);

        // Tamper with the transformed data
        transformed_data[10] ^= 0xFF;

        // Detransform (verify and decrypt) the data and expect failure
        let detransform_result = detransformer.detransform(&transformed_data);
        assert!(detransform_result.is_err());
        assert_eq!(detransform_result.err().unwrap(), SerializationError::CryptographicError(CryptoError::DataTampered));
    }

    #[test]
    fn test_crypto_transformer_detransform_with_invalid_data() {
        // Generate certificates
        let local_signing_cert = generate_falcon1024_certificate();
        let local_encryption_cert = generate_kyber1024_certificate();
        let remote_signing_cert = generate_falcon1024_certificate();
        let remote_encryption_cert = generate_kyber1024_certificate();

        // Initialize the CryptoTransformer
        let detransformer = CryptoTransformer::new(
            remote_signing_cert.clone(),
            remote_encryption_cert.clone(),
            local_signing_cert.clone_without_signature_and_sk(),
            local_encryption_cert.clone_without_signature_and_sk(),
        );

        // Create invalid data
        let invalid_data: Serialized = vec![1, 2, 3, 4, 5];

        // Detransform (verify and decrypt) the invalid data and expect failure
        let detransform_result = detransformer.detransform(&invalid_data);
        assert!(detransform_result.is_err());
    }
}