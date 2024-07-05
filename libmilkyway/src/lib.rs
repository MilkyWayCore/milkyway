pub mod serialization;
pub mod pki;
mod macros;


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

