use std::collections::HashMap;
use libmilkyway::pki::impls::certificates::falcon1024::{Falcon1024Certificate, Falcon1024RootCertificate};
use libmilkyway::pki::impls::certificates::kyber1024::Kyber1024Certificate;
use libmilkyway::services::certificate::CertificateService;

pub(crate) struct CertificateServiceImpl{
    root_certificate: Option<Falcon1024RootCertificate>,
    signing_certificates: HashMap<u128, Falcon1024Certificate>,
    encryption_certificates: HashMap<u128, Kyber1024Certificate>,
}


impl CertificateService for CertificateServiceImpl {
    fn set_root_certificate(&mut self, root_cert: Falcon1024RootCertificate) {
        self.root_certificate = Some(root_cert);
    }

    fn add_signing_certificate(&mut self, cert: Falcon1024Certificate) -> bool {
        todo!()
    }

    fn add_encryption_certificate(&mut self, cert: Kyber1024Certificate) {
        todo!()
    }

    fn verify_signing_certificate(&self, cert: Falcon1024Certificate) -> bool {
        todo!()
    }

    fn verify_encryption_certificate(&self, cert: Kyber1024Certificate) -> bool {
        todo!()
    }

    fn get_signing_certificate(&self, serial: u128) -> Option<Falcon1024Certificate> {
        todo!()
    }

    fn get_encryption_certificate(&self) -> Option<Kyber1024Certificate> {
        todo!()
    }

    fn get_root_certificate(&self) -> Option<Falcon1024Certificate> {
        todo!()
    }

    fn commit(&self) {
        todo!()
    }
}