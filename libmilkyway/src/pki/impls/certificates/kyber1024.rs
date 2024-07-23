use pqcrypto::kem::kyber1024;
use pqcrypto::kem::kyber1024::{PublicKey, SecretKey};
use crate::pki::impls::SerializationError;
use crate::serialization::serializable::Serialized;
use crate::serialization::serializable::Serializable;
use crate::serialization::deserializable::Deserializable;
use libmilkyway_derive::{Deserializable, Serializable};
use crate::pki::certificate::{Certificate, CertificateType, FLAG_SIGN_CERTS, FLAG_SIGN_MESSAGES};
use crate::pki::signature::Signature;


#[derive(Clone, Serializable, Deserializable, PartialEq)]
pub struct Kyber1024Certificate{
    pub serial_number: u128,
    pub parent_serial_number: u128,
    pub secret_key: Option<kyber1024::SecretKey>,
    pub public_key: kyber1024::PublicKey,
    pub signature: Option<Signature>,
    pub name: String,
    pub flags: u128,
}


impl Certificate<kyber1024::PublicKey, kyber1024::SecretKey> for Kyber1024Certificate{
    #[inline]
    fn get_type() -> CertificateType {
        CertificateType::EnciphermentCertificate
    }

    #[inline]
    fn get_serial(&self) -> u128 {
        self.serial_number
    }

    #[inline]
    fn get_parent_serial(&self) -> Option<u128> {
        Some(self.parent_serial_number)
    }

    #[inline]
    fn get_signature(&self) -> Option<Signature> {
        self.signature.clone()
    }

    #[inline]
    fn get_public_key(&self) -> PublicKey {
        self.public_key.clone()
    }

    #[inline]
    fn get_secret_key(&self) -> Option<SecretKey> {
        self.secret_key.clone()
    }

    fn clone_without_signature_and_sk(&self) -> Self {
        let mut m_copy = self.clone();
        m_copy.signature = None;
        m_copy.secret_key = None;
        m_copy
    }

    fn clone_without_signature(&self) -> Self {
        let mut m_copy = self.clone_without_signature_and_sk();
        m_copy.signature = None;
        m_copy
    }
    
    fn clone_without_sk(&self) -> Kyber1024Certificate {
        let mut m_copy = self.clone();
        m_copy.secret_key = None;
        m_copy
    }

    #[inline]
    fn get_name(&self) -> String {
        self.name.clone()
    }

    #[inline]
    fn get_flags(&self) -> u128 {
        self.flags
    }

    fn set_flags(&mut self, flags: u128) {
        if flags & FLAG_SIGN_CERTS != 0 || flags & FLAG_SIGN_MESSAGES != 0{
            panic!("Kyber1024 can not sign anything");
        }
        self.flags = flags;
    }
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use pqcrypto::kem::kyber1024::{self, PublicKey, SecretKey};
    use crate::pki::certificate::Certificate;
    use crate::serialization::serializable::{Serializable, Serialized};
    use crate::serialization::deserializable::Deserializable;
    use crate::pki::hash::HashType;
    use crate::pki::impls::certificates::falcon1024::Falcon1024RootCertificate;
    use crate::pki::impls::keys::falcon1024::generate_falcon1024_keypair;

    #[derive(Clone, Serializable, Deserializable, Debug, PartialEq)]
    struct TestData {
        message: Vec<u8>,
    }

    fn generate_kyber1024_keypair() -> (PublicKey, SecretKey) {
        kyber1024::keypair()
    }

    #[test]
    fn test_full_pki_use_case() {
        let (root_public_key, root_secret_key) = generate_falcon1024_keypair();
        let (encipherment_public_key, encipherment_secret_key) = generate_kyber1024_keypair();

        let root_certificate = Falcon1024RootCertificate {
            secret_key: Some(root_secret_key),
            public_key: root_public_key.clone(),
            name: "test".to_string(),
        };

        let encipherment_certificate = Kyber1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(encipherment_secret_key),
            public_key: encipherment_public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        // Sign the encipherment certificate with the root certificate
        let signature = root_certificate.sign_data(&encipherment_certificate.clone_without_signature_and_sk(),
                                                   HashType::None).unwrap();
        let mut signed_certificate = encipherment_certificate.clone();
        signed_certificate.signature = Some(signature.clone());

        // Verify the signature on the signed certificate
        assert!(root_certificate.verify_signature(&signed_certificate.clone_without_signature_and_sk(),
                                                  &signature));

        // Test message encryption and decryption
        let test_data = TestData {
            message: "Hello, World!".to_string().as_bytes().to_vec(),
        };
        let encrypted_data = signed_certificate.encrypt(&test_data).unwrap();
        let decrypted_data: TestData = signed_certificate.decrypt(&encrypted_data).unwrap();
        assert_eq!(test_data, decrypted_data);

        // Tamper with the message
        let tampered_data = TestData {
            message: "Hello, Universe!".to_string().as_bytes().to_vec(),
        };
        let tampered_encrypted_data = signed_certificate.encrypt(&tampered_data).unwrap();
        assert_ne!(encrypted_data, tampered_encrypted_data);

        // Tamper with the certificate
        let mut tampered_certificate = signed_certificate.clone();
        tampered_certificate.serial_number = 2;
        assert!(!root_certificate.verify_signature(&tampered_certificate.clone_without_signature_and_sk(),
                                                   &signature));
    }

    #[test]
    fn test_certificate_serialization_deserialization() {
        let (public_key, secret_key) = generate_kyber1024_keypair();
        let certificate = Kyber1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key,
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        let serialized = certificate.serialize();
        let (deserialized, _) = Kyber1024Certificate::from_serialized(&serialized).unwrap();
        assert!(certificate == deserialized);
    }

    #[test]
    fn test_clone_without_private() {
        let (public_key, secret_key) = generate_kyber1024_keypair();
        let certificate = Kyber1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key: public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        let cloned_certificate = certificate.clone_without_signature_and_sk();
        assert!(cloned_certificate.secret_key.is_none());
        assert!(cloned_certificate.public_key == public_key);
    }

    #[test]
    fn test_certificate_encryption_and_decryption() {
        let (public_key, secret_key) = generate_kyber1024_keypair();
        let certificate = Kyber1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key: public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        let test_data = TestData {
            message: "Secret message".to_string().as_bytes().to_vec(),
        };

        let encrypted_data = certificate.encrypt(&test_data).unwrap();
        let decrypted_data: TestData = certificate.decrypt(&encrypted_data).unwrap();
        assert_eq!(test_data, decrypted_data);
    }
}