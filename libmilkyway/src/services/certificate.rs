use crate::pki::certificate::Certificate;
use crate::pki::impls::certificates::falcon1024::{Falcon1024Certificate, Falcon1024RootCertificate};
use crate::pki::impls::certificates::kyber1024::Kyber1024Certificate;


pub const ROOT_CERTIFICATE_SERIAL: u128 = 0;

///
/// Certificate service is responsible for handling, storing and obtaining certificates
///
pub trait CertificateService{
    ///
    /// Sets root certificate
    /// 
    /// # Warning
    /// Currently certificate type is hardcoded to a Falcon1024RootCertificate
    /// 
    /// # Arguments
    /// * root_cert: Falcon1024RootCertificate: a root certificate to store
    /// 
    fn set_root_certificate(&mut self, root_cert: Falcon1024RootCertificate);
    
    ///
    /// Verifies a certificate against known chains of certificates and if
    /// successful adds a signing certificate.
    /// 
    /// # Warning
    /// Currently certificate type is hardcoded to a Falcon1024Certificate
    /// 
    /// # Arguments
    /// * cert: Certificate to add
    /// 
    /// returns: bool: whether certificate was added successfully
    /// 
    fn add_signing_certificate(&mut self, cert: Falcon1024Certificate) -> bool;

    ///
    /// Verifies and adds certificate against known chain and if succesful adds
    /// an encryption certificate
    ///
    /// # Warning
    /// Currently certificate type is hardcoded to a Kyber1024Certificate
    ///
    /// # Arguments
    /// * cert: Certificate to add
    ///
    /// returns: bool: whether certificate was added
    ///
    fn add_encryption_certificate(&mut self, cert: Kyber1024Certificate);


    ///
    /// Verifies certificate and returns whether it is valid
    ///
    /// # Arguments
    /// * cert: certificate to verify
    /// 
    /// returns: bool: whether certificate is valid
    /// 
    fn verify_signing_certificate(&self, cert: Falcon1024Certificate) -> bool;
    
    ///
    /// Verifies encryption certificate
    /// 
    /// # Arguments
    /// * cert: certificate to verify
    /// 
    /// returns: bool: whether certificate is valid
    fn verify_encryption_certificate(&self, cert: Kyber1024Certificate) -> bool;
    
    ///
    /// Gets signing certificate
    /// 
    /// # Arguments
    /// * serial: serial number of certificate to get
    /// 
    /// returns: Option<Falcon1024Certificate>: Either a certificate or None if no such certificate
    /// 
    fn get_signing_certificate(&self, serial: u128) -> Option<Falcon1024Certificate>;

    ///
    /// Gets signing certificate
    ///
    /// # Arguments
    /// * serial: serial number of certificate to get
    ///
    /// returns: Option<Kyber1024Certificate>: Either a certificate or None if no such certificate
    ///
    fn get_encryption_certificate(&self) -> Option<Kyber1024Certificate>;

    ///
    /// Gets a root certificate
    ///
    /// # Arguments
    /// * serial: serial number of certificate to get
    ///
    /// returns: Option<Falcon1024RootCertificate>: Either a certificate or None if no such certificate
    ///
    fn get_root_certificate(&self) -> Option<Falcon1024Certificate>;
    
    ///
    /// Commits changes, i.e. writes new certificates to storage/sends to peers/etc.
    /// 
    fn commit(&self);
}