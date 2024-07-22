/* Common services which should be available for modules */

///
/// Name service allows to resolve peers by their names
/// 
pub mod name;

///
/// Ceritifcate service manages certificates and PKI
/// 
pub mod certificate;

///
/// Transport service enables communication with other peers
/// 
pub mod transport;


///
/// An impelementations of services which may be commonly used
/// 
pub mod impls;