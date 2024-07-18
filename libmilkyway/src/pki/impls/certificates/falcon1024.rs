use libmilkyway_derive::{Deserializable, Serializable};
use crate::pki::certificate::{Certificate, CertificateType, FLAG_NO_READ, FLAG_NO_WRITE, FLAG_ROOT_CERT, FLAG_SIGN_CERTS};
use crate::pki::certificate::CertificateType::{RootCertificate,
                                               SigningCertificate};
use crate::pki::impls::keys::falcon1024::{Falcon1024PublicKey, Falcon1024SecretKey, generate_falcon1024_keypair};
use crate::pki::signature::Signature;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};

///
/// A general-usage certificate with Falcon1024 keys encapsulated
///
#[derive(Clone, Serializable, Deserializable, PartialEq)]
pub struct Falcon1024Certificate {
    pub serial_number: u128,
    pub parent_serial_number: u128,
    pub secret_key: Option<Falcon1024SecretKey>,
    pub public_key: Falcon1024PublicKey,
    pub signature: Option<Signature>,
    pub name: String,
    pub flags: u128,
}

impl Certificate<Falcon1024PublicKey, Falcon1024SecretKey> for Falcon1024Certificate {
    #[inline]
    fn get_type() -> CertificateType {
        SigningCertificate
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
    fn get_public_key(&self) -> Falcon1024PublicKey {
        self.public_key.clone()
    }

    #[inline]
    fn get_secret_key(&self) -> Option<Falcon1024SecretKey> {
        self.secret_key.clone()
    }

    fn clone_without_signature_and_sk(&self) -> Self {
        let mut m_copy = self.clone();
        m_copy.secret_key = None;
        m_copy.signature = None;
        m_copy
    }

    fn clone_without_signature(&self) -> Self {
        if self.signature.is_none(){
            return self.clone_without_signature_and_sk();
        }
        let mut m_copy = self.clone_without_signature_and_sk();
        m_copy.signature = None;
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

    #[inline]
    fn set_flags(&mut self, flags: u128) {
        self.flags = flags;
    }
}

///
/// A root certificate which may be used only for signing other certificates
///
#[derive(Clone, Serializable, Deserializable, PartialEq)]
pub struct Falcon1024RootCertificate {
    pub secret_key: Option<Falcon1024SecretKey>,
    pub public_key: Falcon1024PublicKey,
    pub name: String,
}


impl Certificate<Falcon1024PublicKey, Falcon1024SecretKey> for Falcon1024RootCertificate {
    #[inline]
    fn get_type() -> CertificateType {
        RootCertificate
    }

    #[inline]
    fn get_serial(&self) -> u128 {
        0
    }

    #[inline]
    fn get_parent_serial(&self) -> Option<u128> {
        None
    }

    #[inline]
    fn get_signature(&self) -> Option<Signature> {
        None
    }

    #[inline]
    fn get_public_key(&self) -> Falcon1024PublicKey {
        self.public_key.clone()
    }

    #[inline]
    fn get_secret_key(&self) -> Option<Falcon1024SecretKey> {
        self.secret_key.clone()
    }

    fn clone_without_signature_and_sk(&self) -> Self {
        let mut m_copy = self.clone();
        m_copy.secret_key = None;
        m_copy
    }

    fn clone_without_signature(&self) -> Self {
        panic!("Root key does not have signature");
    }

    #[inline]
    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_flags(&self) -> u128 {
        FLAG_ROOT_CERT | FLAG_NO_READ | FLAG_NO_WRITE | FLAG_SIGN_CERTS
    }

    fn set_flags(&mut self, _flags: u128) {
        panic!("Can not set flags for root certificate");
    }
}

pub fn generate_falcon1024_root_certificate(name: String) -> Falcon1024RootCertificate{
    let (root_public_key, root_secret_key) = generate_falcon1024_keypair();
    Falcon1024RootCertificate {
        secret_key: Some(root_secret_key),
        public_key: root_public_key.clone(),
        name,
    }
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pki::certificate::Certificate;
    use crate::pki::impls::keys::falcon1024::generate_falcon1024_keypair;
    use crate::serialization::error::SerializationError;
    use crate::serialization::serializable::{Serializable, Serialized};
    use crate::pki::hash::HashType;
    use crate::pki::key::CryptoKey;

    #[derive(Clone, Serializable, Deserializable)]
    struct TestData {
        message: Vec<u8>,
    }

    #[test]
    fn test_full_pki_use_case() {
        let (root_public_key, root_secret_key) = generate_falcon1024_keypair();
        let (signing_public_key, signing_secret_key) = generate_falcon1024_keypair();

        let root_certificate = Falcon1024RootCertificate {
            secret_key: Some(root_secret_key),
            public_key: root_public_key.clone(),
            name: "test".to_string(),
        };

        let signing_certificate = Falcon1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(signing_secret_key),
            public_key: signing_public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        let signature = root_certificate.sign_data(&signing_certificate.clone_without_signature_and_sk(),
                                                   HashType::None).unwrap();
        let mut signed_certificate = signing_certificate.clone();
        signed_certificate.signature = Some(signature.clone());

        assert!(root_certificate.verify_signature(&signed_certificate.clone_without_signature_and_sk(),
                                                  &signature));

        // Test message signing and verification
        let test_data = TestData {
            message: "Hello, World!".to_string().as_bytes().to_vec(),
        };
        let message_signature = signed_certificate.sign_data(&test_data,
                                                             HashType::None).unwrap();
        assert!(signing_public_key.verify_signature(&test_data, &message_signature));

        // Tamper with the message
        let tampered_data = TestData {
            message: "Hello, Universe!".to_string().as_bytes().to_vec(),
        };
        assert!(!signing_public_key.verify_signature(&tampered_data, &message_signature));

        // Tamper with the certificate
        let mut tampered_certificate = signed_certificate.clone();
        tampered_certificate.serial_number = 2;
        assert!(!root_certificate.verify_signature(&tampered_certificate, &signature));
    }

    #[test]
    fn test_certificate_serialization_deserialization() {
        let (public_key, secret_key) = generate_falcon1024_keypair();
        let certificate = Falcon1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key,
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        let serialized = certificate.serialize();
        let (deserialized, _) = Falcon1024Certificate::from_serialized(&serialized).unwrap();
        assert!(certificate == deserialized);
    }

    #[test]
    fn test_clone_without_private() {
        let (public_key, secret_key) = generate_falcon1024_keypair();
        let certificate = Falcon1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key: public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        let cloned_certificate = certificate.clone_without_signature_and_sk();
        assert!(cloned_certificate.secret_key == None);
        assert!(cloned_certificate.public_key == public_key);
    }

    #[test]
    fn test_certificate_signing_and_verification() {
        let (public_key, secret_key) = generate_falcon1024_keypair();
        let certificate = Falcon1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(secret_key),
            public_key: public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };

        let test_data = TestData {
            message: "Test message".to_string().as_bytes().to_vec(),
        };

        let signature = certificate.sign_data(&test_data, HashType::None).unwrap();
        assert!(public_key.verify_signature(&test_data, &signature));
    }
}

