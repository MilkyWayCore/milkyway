use crate::pki::impls::CryptoError;
///
/// Errors which may occur during serialization/deserialization
///
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationError {
    ///
    /// Error in data, with string comment on error type
    ///
    InvalidDataError(&'static str),

    ///
    /// Wrong length of data supplied
    ///
    LengthError,

    ///
    /// Cryptographic error during serialization of ciphertexts,etc.
    ///
    CryptographicError(CryptoError)
}