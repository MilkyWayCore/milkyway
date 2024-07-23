use crate::serialization::error::SerializationError;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::serializable::Serializable;
use libmilkyway_derive::{Deserializable, Serializable};
use crate::actor::binder::Binder;
use crate::pki::certificate::{Certificate, FLAG_SIGN_MESSAGES};
use crate::pki::hash::HashType;
use crate::pki::impls::certificates::falcon1024::Falcon1024Certificate;
use crate::pki::impls::certificates::kyber1024::Kyber1024Certificate;
use crate::pki::signature::Signature;
use crate::serialization::serializable::Serialized;
use crate::services::certificate::{CertificateService, CertificateServiceBinder, ROOT_CERTIFICATE_SERIAL};

///
/// Controls authorization process.
///
/// # Protocol
/// 1. Client sends it encryption certificate(w/o secret keys) to server signed by its signing certificate
/// 2. Server verifies authenticity of signature and certificate against its chains
/// 3. If verification is OK, server replies with its encryption certificate signed by its signing certificate
/// 4. Client verifies server response against local chain of certificates
/// 5. Now secure communication is established with help of above certificates
///
/// ## Note
/// Additionally each party can share own certificate chain, so it would be no gaps in verification
///
pub struct AuthorizationController{
    certificate_service_binder: Box<CertificateServiceBinder>,
}


///
/// Authorization message provides encryption certificate
/// and optionally a chain of signing certificates.
///
#[derive(Clone, Serializable, Deserializable)]
pub struct AuthorizationMessage{
    encryption_certificate: Kyber1024Certificate,
    signing_certificate: Falcon1024Certificate,
    signing_chain: Vec<Falcon1024Certificate>,
    signature: Option<Signature>,
}


impl AuthorizationMessage {
    pub fn clone_without_signature(&self) -> AuthorizationMessage{
        let mut m_copy = self.clone();
        m_copy.signature = None;
        m_copy
    }
}


impl AuthorizationController {
    ///
    /// Creates a new AuthorizationController
    ///
    /// # Arguments
    /// * binder: A binder to a certificate service
    ///
    pub fn new(binder: Box<CertificateServiceBinder>) -> AuthorizationController{
        AuthorizationController{
            certificate_service_binder: binder,
        }
    }

    ///
    /// Finalizes authorization procedure and cleans up
    ///
    pub fn finalize(&mut self){
        self.certificate_service_binder.unbind();
    }

    ///
    /// Generates authorization message given particular encryption certificate and signing certificate
    /// 
    /// # Arguments
    /// * serial: a ceritficate which should be used for encryption
    /// * signing_serial: a certificate which would be used for signing messages
    /// * fullchain: whether a send whole chain of certificate in authorication message
    /// 
    /// returns: either an authorization message or error with str description
    ///
    pub fn generate_authorization_message(&mut self, serial: u128, signing_serial: u128,
                                                     fullchain: bool) -> Result<AuthorizationMessage, &'static str>{
        let certificate = self.certificate_service_binder.get_encryption_certificate(serial);
        if certificate.is_none(){
            return Err("Can not find a certificate used for encryption with provided serial");
        }
        let mut chain = Vec::<Falcon1024Certificate>::new();
        let certificate = certificate.unwrap();
        if fullchain{
            let current_serial = certificate.get_serial();
            if current_serial == ROOT_CERTIFICATE_SERIAL{
                // Something strange is going on
                return Err("Serial of encryption certificate can not be serial of root certificate");
            }
            let mut parent_serial = certificate.get_parent_serial().expect("Must have a parent serial");
            while parent_serial != ROOT_CERTIFICATE_SERIAL {
                let certificate = self.certificate_service_binder.get_signing_certificate(parent_serial);
                if certificate.is_none(){
                    return Err("Can not trust chain: parent is missing");
                }
                let certificate = certificate.unwrap().clone_without_sk();
                chain.insert(0, certificate.clone());
                parent_serial = certificate.get_parent_serial().expect("Must have a parent serial");
            }
        }
        let signing_certificate = self.certificate_service_binder.get_signing_certificate(signing_serial);
        if signing_certificate.is_none(){
            return Err("Can not find a certificate used for signing with provided serial");
        }
        let signing_certificate = signing_certificate.unwrap();
        let mut message = AuthorizationMessage{
            encryption_certificate: certificate.clone_without_sk(),
            signing_certificate: signing_certificate.clone(),
            signing_chain: chain,
            signature: None,
        };
        if !signing_certificate.check_flag(FLAG_SIGN_MESSAGES){
            return Err("Provided signing certificate is not allowed to sign messages");
        }
        let signature = signing_certificate.sign_data(&message, HashType::None);
        if signature.is_err(){
            return Err("Can not sign message");
        }
        message.signature = Some(signature.unwrap());
        Ok(message)
    }
    
    
    ///
    /// Checks an authorization message
    /// 
    /// # Arguments
    /// * message: a message to verify
    /// 
    /// returns: None if verification failed, pair of signing and encryption certificates otherwise
    /// 
    pub fn check_authorization_message(&mut self, 
                                       message: AuthorizationMessage) -> Option<(Falcon1024Certificate, Kyber1024Certificate)>{
        let signing_certificate  = message.signing_certificate.clone();
        if signing_certificate.signature.is_none(){
            /* Unsigned certificate */
            return None;
        }
        for cert in &message.signing_chain{
            if !self.certificate_service_binder.add_signing_certificate(cert.clone()){
                // Invalid certificate
                return None;
            }
        }
        if !self.certificate_service_binder.verify_signing_certificate(&signing_certificate){
            /* Certificate is invalid event though chain was updated */
            return None;
        }
        let message_no_signature = message.clone_without_signature();
        if !signing_certificate.verify_signature(&message_no_signature, &message.signature.unwrap()){
            /* Message signature invalid */
            return None;
        }
        if !self.certificate_service_binder.verify_encryption_certificate(&message.encryption_certificate){
            /* The encryption certificate is invalid */
            return None;
        }
        self.certificate_service_binder.add_encryption_certificate(message.encryption_certificate.clone());
        return Some((message.signing_certificate, message.encryption_certificate));
    }
}