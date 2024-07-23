use crate::serialization::error::SerializationError;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::serializable::Serializable;
use libmilkyway_derive::{Deserializable, Serializable};
use crate::actor::binder::Binder;
use crate::get_timestamp_with_milliseconds;
use crate::pki::certificate::{Certificate, FLAG_SIGN_CERTS, FLAG_SIGN_MESSAGES};
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
    timestamp: u128,
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
            timestamp: get_timestamp_with_milliseconds(),
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
        if !signing_certificate.check_flag(FLAG_SIGN_MESSAGES){
            /* Wrong flags */
            println!("Signing certificate can not sign messages");
            return None;
        }
        for cert in &message.signing_chain{
            if !self.certificate_service_binder.add_signing_certificate(cert.clone()){
                // Invalid certificate
                return None;
            }
            if !cert.check_flag(FLAG_SIGN_CERTS){
                // Wrong flags
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::pki::certificate::{Certificate, FLAG_SIGN_CERTS, FLAG_SIGN_MESSAGES};
    use crate::pki::hash::HashType;
    use crate::pki::impls::certificates::falcon1024::{Falcon1024Certificate, Falcon1024RootCertificate};
    use crate::pki::impls::certificates::kyber1024::Kyber1024Certificate;
    use crate::actor::binder::coroutine::BinderAsyncService;
    use crate::pki::impls::keys::falcon1024::generate_falcon1024_keypair;
    use crate::pki::impls::keys::kyber1024::generate_kyber1024_keypair;
    use crate::services::impls::certificate::AsyncCertificateServiceImpl;
    use crate::tokio::init_tokio;
    
    fn create_sample_certificates() -> (Kyber1024Certificate, Falcon1024RootCertificate, Falcon1024Certificate) {
        // Create some sample certificates for testing
        let (root_public_key, root_secret_key) = generate_falcon1024_keypair();
        let root_certificate = Falcon1024RootCertificate {
            secret_key: Some(root_secret_key),
            public_key: root_public_key.clone(),
            name: "test".to_string(),
        };
        let (encipherment_public_key, encipherment_secret_key) = generate_kyber1024_keypair();
        let mut encryption_cert = Kyber1024Certificate {
            serial_number: 2,
            parent_serial_number: 1,
            secret_key: Some(encipherment_secret_key),
            public_key: encipherment_public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: 0,
        };
        let (signing_public_key, signing_secret_key) = generate_falcon1024_keypair();
        let mut signing_certificate = Falcon1024Certificate {
            serial_number: 1,
            parent_serial_number: 0,
            secret_key: Some(signing_secret_key),
            public_key: signing_public_key.clone(),
            signature: None,
            name: "test".to_string(),
            flags: FLAG_SIGN_MESSAGES | FLAG_SIGN_CERTS,
        };
        assert!(signing_certificate.check_flag(FLAG_SIGN_MESSAGES));
        signing_certificate.signature = Some(root_certificate.sign_data(&signing_certificate.clone_without_signature_and_sk(),
                                                                        HashType::None).unwrap());
        encryption_cert.signature = Some(signing_certificate.sign_data(&encryption_cert.clone_without_signature_and_sk(), HashType::None).unwrap());


        (encryption_cert, root_certificate, signing_certificate)
    }

    #[test]
    fn test_generate_authorization_message() {
        init_tokio();
        let mut service = BinderAsyncService::run(Box::new(AsyncCertificateServiceImpl::new("/tmp/test.dat")));
        let mut binder = service.bind();
        let (encryption_cert, root_certificate, signing_cert) = create_sample_certificates();
        binder.set_root_certificate(root_certificate.clone());
        assert!(binder.add_signing_certificate(signing_cert.clone()));
        assert!(binder.add_encryption_certificate(encryption_cert.clone()));

        let mut controller = AuthorizationController::new(binder);

        let result = controller.generate_authorization_message(2, 1, false);
        assert!(result.is_ok());
        let auth_message = result.unwrap();
        assert_eq!(auth_message.encryption_certificate.get_serial(), 2);
        assert_eq!(auth_message.signing_certificate.get_serial(), 1);
        assert!(auth_message.signature.is_some());
    }

    #[test]
    fn test_check_authorization_message() {
        init_tokio();
        let mut service = BinderAsyncService::run(Box::new(AsyncCertificateServiceImpl::new("/tmp/test.dat")));
        let mut binder = service.bind();
        let (encryption_cert, root_certificate, signing_cert) = create_sample_certificates();
        binder.set_root_certificate(root_certificate.clone());
        assert!(binder.add_signing_certificate(signing_cert.clone()));
        assert!(binder.add_encryption_certificate(encryption_cert.clone()));
        assert!(signing_cert.check_flag(FLAG_SIGN_MESSAGES));
        binder.add_encryption_certificate(encryption_cert.clone());
        binder.add_signing_certificate(signing_cert.clone());

        let mut controller = AuthorizationController::new(binder);

        let message = AuthorizationMessage {
            encryption_certificate: encryption_cert.clone(),
            signing_certificate: signing_cert.clone(),
            signing_chain: vec![],
            signature: None,
            timestamp: 0,
        };

        let signature = signing_cert.sign_data(&message.clone_without_signature(), HashType::None).unwrap();
        let mut signed_message = message.clone();
        signed_message.signature = Some(signature);

        let result = controller.check_authorization_message(signed_message);

        assert!(result.is_some());
        let (signing_cert_out, encryption_cert_out) = result.unwrap();
        assert_eq!(signing_cert_out.get_serial(), signing_cert.get_serial());
        assert_eq!(encryption_cert_out.get_serial(), encryption_cert.get_serial());
    }
}