use crate::actor::binder::{Binder, BinderChannel, BinderMessage, BinderServiceHandler};
use crate::actor::binder::coroutine::BinderAsyncService;
use crate::pki::impls::certificates::falcon1024::{Falcon1024Certificate, Falcon1024RootCertificate};
use crate::pki::impls::certificates::kyber1024::Kyber1024Certificate;
use crate::services::certificate::CertificateServiceBinderRequest::SetSigningCertificate;
use crate::services::certificate::CertificateServiceBinderResponse::{Falcon1024Cert, Falcon1024Certs, Kyber1024Cert, Kyber1024Certs, RootCert, Status};
use crate::unwrap_variant;


pub const ROOT_CERTIFICATE_SERIAL: u128 = 0;

///
/// Certificate service is responsible for handling, storing and obtaining certificates
///
pub trait CertificateService: Send + Sync{
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
    fn add_encryption_certificate(&mut self, cert: Kyber1024Certificate) -> bool;


    ///
    /// Verifies certificate and returns whether it is valid
    ///
    /// # Arguments
    /// * cert: certificate to verify
    /// 
    /// returns: bool: whether certificate is valid
    /// 
    fn verify_signing_certificate(&mut self, cert: &Falcon1024Certificate) -> bool;
    
    ///
    /// Verifies encryption certificate
    /// 
    /// # Arguments
    /// * cert: certificate to verify
    /// 
    /// returns: bool: whether certificate is valid
    fn verify_encryption_certificate(&mut self, cert: &Kyber1024Certificate) -> bool;
    
    ///
    /// Gets signing certificate
    /// 
    /// # Arguments
    /// * serial: serial number of certificate to get
    /// 
    /// returns: Option<Falcon1024Certificate>: Either a certificate or None if no such certificate
    /// 
    fn get_signing_certificate(&mut self, serial: u128) -> Option<Falcon1024Certificate>;

    ///
    /// Gets signing certificate
    ///
    /// # Arguments
    /// * serial: serial number of certificate to get
    ///
    /// returns: Option<Kyber1024Certificate>: Either a certificate or None if no such certificate
    ///
    fn get_encryption_certificate(&mut self, serial: u128) -> Option<Kyber1024Certificate>;

    ///
    /// Gets a root certificate
    ///
    /// returns: Option<Falcon1024RootCertificate>: Either a certificate or None if no such certificate
    ///
    fn get_root_certificate(&mut self) -> Option<Falcon1024RootCertificate>;

    ///
    /// Gets all signing certificates
    ///
    /// returns: Vec<Falcon1024Certificate>: a vector of signing certificates
    ///
    fn get_signing_certificates(&mut self) -> Vec<Falcon1024Certificate>;

    ///
    /// Gets all encryption certificates
    ///
    /// returns: Vec<Kyber1024Certificate>: a vector of encryption certificates
    ///
    fn get_encryption_certificates(&mut self) -> Vec<Kyber1024Certificate>;
    
    ///
    /// Commits changes, i.e. writes new certificates to storage/sends to peers/etc.
    /// 
    fn commit(&mut self);
}

pub enum CertificateServiceBinderRequest{
    AddEncryptionCertificate(Kyber1024Certificate),
    AddSigningCertificate(Falcon1024Certificate),
    SetSigningCertificate(Falcon1024RootCertificate),
    VerifySigningCertificate(Falcon1024Certificate),
    VerifyEncryptionCertificate(Kyber1024Certificate),
    GetSigningCertificate(u128),
    GetEncryptionCertificate(u128),
    GetRootCertificate,
    GetEncryptionCertificates,
    GetSigningCertificates,
    Commit,
}

pub enum CertificateServiceBinderResponse{
    Falcon1024Cert(Option<Falcon1024Certificate>),
    Kyber1024Cert(Option<Kyber1024Certificate>),
    RootCert(Option<Falcon1024RootCertificate>),
    Falcon1024Certs(Vec<Falcon1024Certificate>),
    Kyber1024Certs(Vec<Kyber1024Certificate>),
    Status(bool),
}

///
/// A binder type for CertificateServiceBinder
///
pub type CertificateServiceBinder = dyn BinderChannel<BinderMessage<CertificateServiceBinderRequest,
    CertificateServiceBinderResponse>>;

impl CertificateService for dyn BinderChannel<BinderMessage<CertificateServiceBinderRequest,
    CertificateServiceBinderResponse>>{

    #[inline]
    fn set_root_certificate(&mut self, root_cert: Falcon1024RootCertificate) {
        let result = unwrap_variant!(self.handle_request(SetSigningCertificate(root_cert)),
            Status);
        if !result{
            panic!("Can not set root certificate!");
        }
    }

    #[inline]
    fn add_signing_certificate(&mut self, cert: Falcon1024Certificate) -> bool {
        let result = unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::AddSigningCertificate(cert)), Status);
        result
    }

    #[inline]
    fn add_encryption_certificate(&mut self, cert: Kyber1024Certificate) -> bool {
        let result = unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::AddEncryptionCertificate(cert)),
            Status);
        result
    }

    #[inline]
    fn verify_signing_certificate(&mut self, cert: &Falcon1024Certificate) -> bool {
        let result = unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::VerifySigningCertificate(cert.clone())), Status);
        result
    }

    #[inline]
    fn verify_encryption_certificate(&mut self, cert: &Kyber1024Certificate) -> bool {
        let result = unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::VerifyEncryptionCertificate(cert.clone())), Status);
        result
    }

    #[inline]
    fn get_signing_certificate(&mut self, serial: u128) -> Option<Falcon1024Certificate> {
        unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::GetSigningCertificate(serial)), Falcon1024Cert)
    }

    #[inline]
    fn get_encryption_certificate(&mut self, serial: u128) -> Option<Kyber1024Certificate> {
        unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::GetEncryptionCertificate(serial)), Kyber1024Cert)
    }

    #[inline]
    fn get_root_certificate(&mut self) -> Option<Falcon1024RootCertificate> {
        unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::GetRootCertificate), RootCert)
    }

    fn get_signing_certificates(&mut self) -> Vec<Falcon1024Certificate> {
       unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::GetSigningCertificates), Falcon1024Certs)
    }

    fn get_encryption_certificates(&mut self) -> Vec<Kyber1024Certificate> {
        unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::GetEncryptionCertificates), Kyber1024Certs)
    }

    #[inline]
    fn commit(&mut self) {
        let result = unwrap_variant!(self.handle_request(CertificateServiceBinderRequest::Commit), Status);
        if !result{
            panic!("Remote commit failed");
        }
    }
}

///
/// A common service handler for CertificateService
/// 
pub type CertificateServiceHandler = dyn BinderServiceHandler<CertificateServiceBinderRequest,
    CertificateServiceBinderResponse>;

///
/// Asynchornous certificate service
/// 
pub type CertificateAsyncService = BinderAsyncService<CertificateServiceBinderRequest, 
    CertificateServiceBinderResponse>;

impl BinderServiceHandler<CertificateServiceBinderRequest, 
    CertificateServiceBinderResponse> for dyn CertificateService {
    fn handle_message(&mut self, 
                      request: CertificateServiceBinderRequest) -> CertificateServiceBinderResponse {
        match request {
            CertificateServiceBinderRequest::AddEncryptionCertificate(certificate) => {
                Status(self.add_encryption_certificate(certificate))
            }
            CertificateServiceBinderRequest::AddSigningCertificate(certificate) => {
                Status(self.add_signing_certificate(certificate))
            }
            CertificateServiceBinderRequest::SetSigningCertificate(root_certificate) => {
                self.set_root_certificate(root_certificate);
                Status(true)
            }
            CertificateServiceBinderRequest::VerifySigningCertificate(certificate) => {
                Status(self.verify_signing_certificate(&certificate))
            }
            CertificateServiceBinderRequest::VerifyEncryptionCertificate(certificate) => {
                Status(self.verify_encryption_certificate(&certificate))
            }
            CertificateServiceBinderRequest::GetSigningCertificate(serial) => {
                Falcon1024Cert(self.get_signing_certificate(serial))
            }
            CertificateServiceBinderRequest::GetEncryptionCertificate(serial) => {
                Kyber1024Cert(self.get_encryption_certificate(serial))
            }
            CertificateServiceBinderRequest::GetRootCertificate => {
                RootCert(self.get_root_certificate())
            }
            CertificateServiceBinderRequest::GetSigningCertificates => {
                Falcon1024Certs(self.get_signing_certificates())
            }
            CertificateServiceBinderRequest::GetEncryptionCertificates => {
                Kyber1024Certs(self.get_encryption_certificates())
            }
            CertificateServiceBinderRequest::Commit => {
                self.commit();
                Status(true)
            }
        }
    }
}