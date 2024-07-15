use crate::pki::impls::certificates::falcon1024::{Falcon1024Certificate, Falcon1024RootCertificate};

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
    fn add_certificate(&mut self, cert: Falcon1024Certificate) -> bool;
}