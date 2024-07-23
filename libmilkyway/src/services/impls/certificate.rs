use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::Serialized;
use crate::serialization::serializable::Serializable;
use std::collections::HashMap;
use std::path::Path;
use crate::actor::binder::BinderServiceHandler;
use crate::pki::certificate::Certificate;
use crate::pki::impls::certificates::falcon1024::{Falcon1024Certificate, Falcon1024RootCertificate};
use crate::pki::impls::certificates::kyber1024::Kyber1024Certificate;
use crate::services::certificate::{CertificateService, CertificateServiceBinderRequest, CertificateServiceBinderResponse};
use libmilkyway_derive::{Deserializable, Serializable};


#[derive(Serializable, Deserializable)]
pub struct AsyncCertificateServiceImpl {
    storage_file_name: String,
    root_certificate: Option<Falcon1024RootCertificate>,
    signing_certificates: HashMap<u128, Falcon1024Certificate>,
    encryption_certificates: HashMap<u128, Kyber1024Certificate>,
}

impl AsyncCertificateServiceImpl {
    ///
    /// Creates a new CertificateServiceImpl storing data in provided file
    ///
    pub fn new(filename: &str) -> AsyncCertificateServiceImpl {
        AsyncCertificateServiceImpl {
            storage_file_name: filename.to_string(),
            root_certificate: None,
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        }
    }

    #[inline]
    pub fn load_from_file(file: &str) -> AsyncCertificateServiceImpl {
        let mut service = AsyncCertificateServiceImpl::from_file(Path::new(file)).expect("Failed to load certificate storage");
        service.storage_file_name = file.to_string();
        service
    }
}


impl CertificateService for AsyncCertificateServiceImpl {
    #[inline]
    fn set_root_certificate(&mut self, root_cert: Falcon1024RootCertificate) {
        self.root_certificate = Some(root_cert);
    }

    fn add_signing_certificate(&mut self, cert: Falcon1024Certificate) -> bool {
        if cert.get_signature().is_none(){
            // Trying to add unsigned certificate
            println!("Unsigned cert");
            return false;
        }
        if !self.verify_signing_certificate(&cert){
            // Trying to add wrong-signed certificate
            println!("Bad signature");
            return false;
        }
        let parent_serial = cert.get_parent_serial();
        let serial = cert.get_serial();
        if parent_serial.is_none(){
            // Trying to add certificate without parent
            println!("No parent\n");
            return false;
        }
        //let parent_serial = parent_serial.unwrap();
        if self.signing_certificates.contains_key(&serial)
            || self.encryption_certificates.contains_key(&serial) || serial == 0 {
            // Certificate collision
            println!("Collision\n");
            return false;
        }
        self.signing_certificates.insert(serial, cert.clone());
        true
    }

    fn add_encryption_certificate(&mut self, cert: Kyber1024Certificate) -> bool {
        if cert.get_signature().is_none(){
            // Trying to add unsigned certificate
            println!("Unsigned\n");
            return false;
        }
        let parent_serial = cert.get_parent_serial();
        let serial = cert.get_serial();
        if parent_serial.is_none(){
            // Trying to add certificate without parent
            println!("Orphaned\n");
            return false;
        }
        //let parent_serial = parent_serial.unwrap();
        if !self.verify_encryption_certificate(&cert){
            // Tampered certificate?
            println!("Tampered\n");
            return false;
        }
        if self.signing_certificates.contains_key(&serial)
            || self.encryption_certificates.contains_key(&serial) || serial == 0 {
            // Certificate collision
            println!("Collision\n");
            return false;
        }
        self.encryption_certificates.insert(serial, cert.clone());
        true
    }

    fn verify_signing_certificate(&mut self, cert: &Falcon1024Certificate) -> bool {
        let mut current_serial = cert.get_serial();
        let mut current_cert = cert.clone();
        loop{
            let parent_serial = current_cert.get_parent_serial();
            if parent_serial.is_none(){
                // No parent certificate
                println!("No parent");
                return false;
            }
            let parent_serial = parent_serial.unwrap();
            if parent_serial == 0{
                // We reached root certificate
                let root = self.get_root_certificate();
                if root.is_none(){
                    // No certificates are valid w/o root
                    println!("No root\n");
                    return false;
                }
                let root = root.unwrap();
                let signature = current_cert.get_signature();
                if signature.is_none(){
                    // Last certificate in chain is unsigned
                    println!("No signature");
                    return false;
                }
                let signature = signature.unwrap();
                println!("sig={:?}", signature);
                println!("verify {:?} against {:?}", current_cert.get_serial(), root.get_serial());
                return root.verify_signature(&current_cert.clone_without_signature_and_sk(), &signature);
            }
            let parent_cert_result = self.get_signing_certificate(current_serial);
            if parent_cert_result.is_none(){
                // No such certificate
                return false;
            }
            let parent_cert = parent_cert_result.unwrap();
            let signature_result = current_cert.get_signature();
            if signature_result.is_none(){
                // Unsigned certificate
                return false;
            }
            let is_valid = parent_cert.verify_signature(&current_cert.clone_without_signature(),
                                                        &signature_result.unwrap());
            if !is_valid{
                // One of certificates is tampered
                return false;
            }
            current_serial = parent_serial;
            current_cert = parent_cert;
        }
    }

    fn verify_encryption_certificate(&mut self, cert: &Kyber1024Certificate) -> bool {
        let parent_id = cert.get_parent_serial();
        if parent_id.is_none(){
            // Unsigned certificate
            println!("Unsigned");
            return false;
        }
        let signature = cert.get_signature();
        if signature.is_none(){
            // Unsigned certificate
            println!("Unsigned: bad sig");
            return false;
        }
        let signature = signature.unwrap();
        let parent = self.get_signing_certificate(parent_id.unwrap());
        if parent.is_none(){
            // No such signing certificate
            println!("Orpahned: parent lost");
            return false;
        }
        let parent = parent.unwrap();
        if !self.verify_signing_certificate(&parent){
            // Parent is invalid
            println!("Parent is invalid");
            return false;
        }
        return parent.verify_signature(&cert.clone_without_signature_and_sk(), &signature);
    }

    fn get_signing_certificate(&mut self, serial: u128) -> Option<Falcon1024Certificate> {
        let result = self.signing_certificates.get(&serial);
        if result.is_none(){
            None
        } else {
            Some(result.unwrap().clone())
        }
    }

    fn get_encryption_certificate(&mut self, serial: u128) -> Option<Kyber1024Certificate> {
        let result = self.encryption_certificates.get(&serial);
        if result.is_none(){
            None
        } else {
            Some(result.unwrap().clone())
        }
    }

    #[inline]
    fn get_root_certificate(&mut self) -> Option<Falcon1024RootCertificate> {
        self.root_certificate.clone()
    }

    fn get_signing_certificates(&mut self) -> Vec<Falcon1024Certificate> {
        let mut result = Vec::<Falcon1024Certificate>::new();
        for certificate in self.signing_certificates.values(){
            result.push(certificate.clone());
        }
        result
    }

    fn get_encryption_certificates(&mut self) -> Vec<Kyber1024Certificate> {
        let mut result = Vec::<Kyber1024Certificate>::new();
        for certificate in self.encryption_certificates.values(){
            result.push(certificate.clone());
        }
        result
    }

    fn remove_signing_certificate(&mut self, serial: u128) -> bool {
        if !self.signing_certificates.contains_key(&serial){
            return false;
        }
        self.signing_certificates.remove(&serial);
        true
    }

    fn remove_encryption_certificate(&mut self, serial: u128) -> bool {
        if !self.encryption_certificates.contains_key(&serial){
            return false;
        }
        self.encryption_certificates.remove(&serial);
        true
    }

    #[inline]
    fn commit(&mut self) {
        self.dump(&self.storage_file_name);
    }
}

//FIXME: Still no idea why I ever should write this mess
impl BinderServiceHandler<CertificateServiceBinderRequest, CertificateServiceBinderResponse> for AsyncCertificateServiceImpl {
    fn handle_message(&mut self, request: CertificateServiceBinderRequest) -> CertificateServiceBinderResponse {
        let ptr: &mut dyn CertificateService = self;
        ptr.handle_message(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pki::impls::keys::falcon1024::{generate_falcon1024_keypair};
    use crate::pki::impls::keys::kyber1024::{generate_kyber1024_keypair};
    use std::collections::HashMap;
    use crate::pki::hash::HashType;

    fn create_test_root_certificate() -> Falcon1024RootCertificate {
        let (public_key, secret_key) = generate_falcon1024_keypair();
        Falcon1024RootCertificate {
            secret_key: Some(secret_key),
            public_key,
            name: "".to_string(),
        }
    }

    fn create_test_signing_certificate(parent_serial: u128, root_cert: &Falcon1024RootCertificate) -> Falcon1024Certificate {
        let (public_key, secret_key) = generate_falcon1024_keypair();
        let mut cert = Falcon1024Certificate {
            serial_number: parent_serial + 1,
            parent_serial_number: parent_serial,
            secret_key: Some(secret_key),
            public_key,
            signature: None,
            name: "".to_string(),
            flags: 0,
        };
        let signature = root_cert.sign_data(&cert.clone_without_signature_and_sk(), HashType::None).unwrap();
        cert.signature = Some(signature);
        cert
    }

    fn create_test_encryption_certificate(parent_serial: u128, signing_cert: &Falcon1024Certificate) -> Kyber1024Certificate {
        let (public_key, secret_key) = generate_kyber1024_keypair();
        let mut cert = Kyber1024Certificate {
            serial_number: parent_serial + 1,
            parent_serial_number: parent_serial,
            secret_key: Some(secret_key),
            public_key,
            signature: None,
            name: "Test".to_string(),
            flags: 0,
        };
        let signature = signing_cert.sign_data(&cert.clone_without_signature_and_sk(), HashType::None).unwrap();
        cert.signature = Some(signature);
        cert
    }

    #[test]
    fn test_set_root_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: None,
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        service.set_root_certificate(root_cert.clone());
        assert!(service.get_root_certificate() == Some(root_cert));
    }

    #[test]
    fn test_add_signing_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let signing_cert = create_test_signing_certificate(0, &root_cert);
        assert!(service.add_signing_certificate(signing_cert.clone()));
        assert!(service.get_signing_certificate(signing_cert.get_serial()) == Some(signing_cert));
    }

    #[test]
    fn test_add_invalid_signing_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let mut signing_cert = create_test_signing_certificate(0, &root_cert);
        signing_cert.signature = None; // Invalidate the signature
        assert!(!service.add_signing_certificate(signing_cert));
    }

    #[test]
    fn test_add_encryption_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let signing_cert = create_test_signing_certificate(0, &root_cert);
        assert!(service.verify_signing_certificate(&signing_cert));
        assert!(service.add_signing_certificate(signing_cert.clone()));

        let encryption_cert = create_test_encryption_certificate(signing_cert.get_serial(), &signing_cert);
        assert!(service.add_encryption_certificate(encryption_cert.clone()));
        assert!(service.get_encryption_certificate(encryption_cert.get_serial()) == Some(encryption_cert));
    }

    #[test]
    fn test_add_invalid_encryption_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let signing_cert = create_test_signing_certificate(0, &root_cert);
        assert!(service.add_signing_certificate(signing_cert.clone()));

        let mut encryption_cert = create_test_encryption_certificate(signing_cert.get_serial(), &signing_cert);
        encryption_cert.signature = None; // Invalidate the signature
        assert!(!service.add_encryption_certificate(encryption_cert));
    }

    #[test]
    fn test_verify_signing_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let signing_cert = create_test_signing_certificate(0, &root_cert);
        assert!(service.add_signing_certificate(signing_cert.clone()));
        assert!(service.verify_signing_certificate(&signing_cert));
    }

    #[test]
    fn test_verify_invalid_signing_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let mut signing_cert = create_test_signing_certificate(0, &root_cert);
        signing_cert.signature = None; // Invalidate the signature
        assert!(!service.verify_signing_certificate(&signing_cert));
    }

    #[test]
    fn test_verify_encryption_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let signing_cert = create_test_signing_certificate(0, &root_cert);
        assert!(service.add_signing_certificate(signing_cert.clone()));

        let encryption_cert = create_test_encryption_certificate(signing_cert.get_serial(), &signing_cert);
        assert!(service.add_encryption_certificate(encryption_cert.clone()));
        assert!(service.verify_encryption_certificate(&encryption_cert));
    }

    #[test]
    fn test_verify_invalid_encryption_certificate() {
        let root_cert = create_test_root_certificate();
        let mut service = AsyncCertificateServiceImpl {
            storage_file_name: "test_storage.bin".to_string(),
            root_certificate: Some(root_cert.clone()),
            signing_certificates: HashMap::new(),
            encryption_certificates: HashMap::new(),
        };
        let signing_cert = create_test_signing_certificate(0, &root_cert);
        assert!(service.add_signing_certificate(signing_cert.clone()));

        let mut encryption_cert = create_test_encryption_certificate(signing_cert.get_serial(), &signing_cert);
        encryption_cert.signature = None; // Invalidate the signature
        assert!(!service.verify_encryption_certificate(&encryption_cert));
    }
}