///
/// Serialization and deserialization to byte arrays implementation
/// 
pub mod serialization;

///
/// Postquantum PKI implementation
/// 
pub mod pki;

///
/// Standard macros for simplifying writing code
/// 
pub mod macros;

///
/// Common messaging protocol
/// 
pub mod message;

///
/// Communication implementations
/// 
pub mod transport;

///
/// tokio utilities
/// 
pub mod tokio;

///
/// A module for loading dynamic modules
/// 
pub mod module;

///
/// Common protocol for sharing core features with modules
/// 
pub mod services;

/// 
/// CLI utilites
/// 
pub mod cli;


///
/// Actor-model architecture utilities
/// 
pub mod actor;

///
/// Common controllers
/// 
pub mod controllers;
mod utils;

use std::time::{SystemTime, UNIX_EPOCH};

///
/// Get exact timestamp with milliseconds
/// 
pub fn get_timestamp_with_milliseconds() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

/* Library-wide tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use libmilkyway_derive::Serializable;
    use libmilkyway_derive::Deserializable;
    use crate::serialization::serializable::Serializable;
    use crate::serialization::deserializable::Deserializable;
    use crate::serialization::serializable::Serialized;
    use serialization::error::SerializationError;

    #[derive(Serializable, Deserializable, Debug, PartialEq)]
    struct MyStruct {
        a: u32,
        b: Vec<u8>,
    }

    /** Test to check Serializable/Deserializable derive proc macros **/
    #[test]
    fn test_serialize_deserialize_my_struct() {
        let my_struct = MyStruct {
            a: 42,
            b: vec![0, 1, 42],
        };

        let serialized = my_struct.serialize();
        let (deserialized, _) = MyStruct::from_serialized(&serialized).unwrap();

        assert_eq!(my_struct, deserialized);
    }
}

