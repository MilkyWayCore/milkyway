use libmilkyway_derive::{EnumDeserializable, EnumSerializable};
use crate::pki::hash::{CryptoHashable, HashType};
use crate::pki::impls::CryptoError;
use crate::pki::key::CryptoKey;
use crate::pki::signature::Signature;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::serializable::{Serializable, Serialized};

///
/// Ceritificate types
///
#[derive(PartialEq, Clone, Debug, EnumSerializable, EnumDeserializable)]
pub enum CertificateType{
    ///
    /// The certificate used only to sign other certificates
    ///
    RootCertificate,

    ///
    /// Certificate for signing data or other certificates
    ///
    SigningCertificate,

    ///
    /// Certificate with key that can be used for encipherment
    ///
    EnciphermentCertificate,
}


///
/// A PKI Certificate base trait
///
pub trait Certificate<PK: CryptoKey, SK: CryptoKey>: Serializable + Deserializable{
    ///
    /// Gets type of certificate
    ///
    fn get_type() -> CertificateType;

    ///
    /// Gets serial number of certificate
    ///
    fn get_serial(&self) -> u128;

    ///
    /// Get serial number of certificate with which this certificate is signed.
    /// Returns `None` if certificate is root.
    ///
    fn get_parent_serial(&self) -> Option<u128>;

    ///
    /// Get signature for certificate or gives `None` if certificate is root
    ///
    fn get_signature(&self) -> Option<Signature>;

    ///
    /// Gets certificate public key
    ///
    fn get_public_key(&self) -> PK;

    ///
    /// Gets certificate secret key if it is available
    ///
    fn get_secret_key(&self) -> Option<SK>;

    ///
    /// Clones certificate without private data
    ///
    fn clone_without_signature_and_sk(&self) -> Self;


    ///
    /// Clones certificate without signature(used for verifying signature)
    ///
    fn clone_without_signature(&self) -> Self;

    ///
    /// Signs piece of data with certificate secret key
    /// # Arguments
    ///
    /// * `data`: Data to sign
    /// * `hash_type`: Hash type to use during signature
    ///
    /// returns: Result<Signature, CryptoError>
    ///
    fn sign_data<T: Serializable + CryptoHashable>(&self, data: &T,
                                                   hash_type: HashType) -> Result<Signature, CryptoError>{
        let key_option = self.get_secret_key();
        if key_option.is_none(){
            return Err(CryptoError::ArgumentError("The certificate does not have private key"));
        }
        let m_type = Self::get_type();
        if m_type == CertificateType::EnciphermentCertificate{
            return Err(CryptoError::ArgumentError("Certificate is for encipherment, not signing"));
        }
        let key = key_option.unwrap();
        return key.sign(data, hash_type);
    }

    ///
    /// Verifies signature of data
    ///
    /// # Arguments
    ///
    /// * `data`: Data which signature must be verified
    /// * `signature`: Signature itself
    ///
    /// returns: bool
    ///
    fn verify_signature<T: Serializable + CryptoHashable>(&self, data: &T,
                                                          signature: &Signature) -> bool{
        let m_type = Self::get_type();
        if m_type == CertificateType::EnciphermentCertificate{
            panic!("Trying to use encipherment certificate for signature verification");
        }
        let key = self.get_public_key();
        key.verify_signature(data, signature)
    }

    ///
    /// Encrypts data with certificate public key
    /// # Arguments
    ///
    /// * `data`: Data to encrypt
    ///
    /// returns: Result<Vec<u8, Global>, CryptoError>
    ///
    fn encrypt<T: Serializable>(&self, data: &T) -> Result<Serialized, CryptoError>{
        let m_type = Self::get_type();
        if m_type != CertificateType::EnciphermentCertificate{
            return Err(CryptoError::ArgumentError("Using non-encipherment certificate for encryption"));
        }
        let key = self.get_public_key();
        key.encrypt(data)
    }

    ///
    /// Decrypts data with secret key of certificate
    /// # Arguments
    ///
    /// * `data`: Data to decrypt
    ///
    /// returns: Result<T, SerializationError>
    ///
    fn decrypt<T: Deserializable>(&self, data: &Serialized) -> Result<T, SerializationError>{
        let key_option = self.get_secret_key();
        //TODO: Use either CryptoError or proper SerializationError
        if key_option.is_none(){
            return Err(SerializationError::InvalidDataError(""));
        }
        let m_type = Self::get_type();
        if m_type != CertificateType::EnciphermentCertificate{
            return Err(SerializationError::InvalidDataError(""));
        }
        let key = key_option.unwrap();
        key.decrypt::<T>(data)
    }
    
    ///
    /// Gets name of certificate
    /// 
    /// returns: String: name of certificate
    ///
    fn get_name(&self) -> String;
    
    
    ///
    /// Gets flags of certificate
    /// 
    /// returns: u128: certificate flags
    ///
    fn get_flags(&self) -> u128;
    
    ///
    /// Sets flags to certificate
    /// 
    /// # Arguments
    /// * flags: u128: flags to set
    /// 
    fn set_flags(&mut self, flags: u128);
    
    ///
    /// Checks that certificate has certain flag
    /// 
    /// # Arguments
    /// * mask: u128: flag mask to check
    /// 
    /// returns: bool: if flags from mask are set
    /// 
    #[inline]
    fn check_flag(&self, mask: u128) -> bool{
        self.get_flags() & mask != 0
    }
    
    
    ///
    /// Sets certain flags mask
    /// 
    /// # Arguments
    /// * mask: u128: flag mask to set
    /// 
    fn set_flag(&mut self, mask: u128){
        let current_flags = self.get_flags();
        self.set_flags(current_flags | mask);
    }
    
    ///
    /// Unset certain flags mask
    /// 
    /// # Arguments
    /// * mask: u128: flag mask to unset
    /// 
    fn unset_flag(&mut self, mask: u128){
        let current_flags = self.get_flags();
        self.set_flags(current_flags & (!mask));
    }
}

///
/// Flag that certificate is root
/// 
pub const FLAG_ROOT_CERT: u128 = 1;

///
/// Flag that certificate is used by user
/// 
pub const FLAG_USER_CERT: u128 = 1<<1;

///
/// Flag that certificate is used by server/broker
/// 
pub const FLAG_SERVER_CERT: u128 = 1<<2;

///
/// Flag that certificate is used by client machine
/// 
pub const FLAG_CLIENT_CERT: u128 = 1<<3;

///
/// Flag that certificate can sign other certificates
/// 
pub const FLAG_SIGN_CERTS: u128 = 1<<4;

///
/// Flag that certificate can sign data, messages
/// 
pub const FLAG_SIGN_MESSAGES: u128 = 1<<5;

///
/// Flag that the commands signed by this certificate can not write anything
/// 
pub const FLAG_NO_WRITE: u128 = 1<<6;

///
/// Flag that the command signed by this certificate can not read state
/// 
pub const FLAG_NO_READ: u128 = 1<<7;

