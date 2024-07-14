use std::collections::HashMap;
use std::hash::Hash;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::error::SerializationError;
use crate::serialization::error::SerializationError::{InvalidDataError, LengthError};
use crate::serialization::serializable::{Serializable, Serialized};

pub mod serializable;
pub mod deserializable;
pub mod error;


macro_rules! int_type_serializable_deserializable {
    ($($t:ty),*) => {
        $(
            impl Serializable for $t {
                fn serialize(&self) -> Serialized {
                    self.to_le_bytes().to_vec()
                }
            }

            impl Deserializable for $t {
                fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
                    let size = std::mem::size_of::<$t>();
                    if serialized.len() < size {
                        return Err(SerializationError::LengthError);
                    }
                    let bytes: [u8; std::mem::size_of::<$t>()] = serialized[..size].try_into().map_err(|_| SerializationError::InvalidDataError("Read out of bytes"))?;
                    Ok((<$t>::from_le_bytes(bytes), size))
                }
            }
        )*
    }
}

int_type_serializable_deserializable!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize);

impl<T> Serializable for Vec<T> where T: Serializable{
    fn serialize(&self) -> Serialized {
        let mut result = Serialized::new();
        result.extend(self.len().serialize());
        for s in self.iter(){
            result.extend(s.serialize());
        }
        result
    }
}

impl<T> Deserializable for Vec<T> where T: Deserializable{
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let mut result = Vec::<T>::new();
        let deserialized_size = usize::from_serialized(serialized);
        if deserialized_size.is_err() {
            return Err(deserialized_size.err().unwrap());
        }
        let (size, mut offset) = deserialized_size.unwrap();
        for _ in 0..size{
            let element_result = T::from_serialized(&serialized[offset..].to_vec());
            if element_result.is_err() {
                return Err(element_result.err().unwrap());
            }
            let (element, element_offset) = element_result.unwrap();
            result.push(element);
            offset += element_offset;
        }
        Ok((result, offset))
    }
}

impl<T> Serializable for Option<T> where T: Serializable + Clone {
    fn serialize(&self) -> Serialized {
        if self.is_none(){
            Serialized::from(&[0])
        } else {
            let mut result = Serialized::from(&[1]);
            let serialized_internal = self.clone().unwrap().serialize();
            result.extend(serialized_internal);
            result
        }
    }
}

impl<T> Deserializable for Option<T> where T: Deserializable{
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        if serialized.len() == 0{
            return Err(SerializationError::LengthError);
        }
        let option_flag = serialized[0] != 0;
        if !option_flag{
            return Ok((None, 1));
        }
        let deserialization_result = T::from_serialized(&serialized[1..].to_vec());
        if deserialization_result.is_err(){
            return Err(deserialization_result.err().unwrap());
        }
        let (deserialized, mut offset) = deserialization_result.unwrap();
        offset += 1; // We have used 1 byte for option info
        Ok((Some(deserialized), offset))
    }
}

impl Serializable for bool {
    fn serialize(&self) -> Serialized {
        if *self{
            Serialized::from(&[1])
        } else {
            Serialized::from(&[0])
        }
    }
}

impl Deserializable for bool{
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        if serialized.len() < 1{
            return Err(LengthError);
        }
        return if serialized[0] == 0 {
            Ok((false, 1))
        } else {
            Ok((true, 1))
        }
    }
}

impl<K: Serializable + Clone, V: Serializable + Clone> Serializable for HashMap<K, V> {
    fn serialize(&self) -> Serialized {
        let mut keys = Vec::<K>::new();
        let mut values = Vec::<V>::new();
        for (key, value) in self.iter(){
            keys.push(key.clone());
            values.push(value.clone());
        }
        let mut result = keys.serialize();
        result.extend(values.serialize());
        result
    }
}

impl<K: Deserializable + Eq + Hash + Clone, 
     V: Deserializable + Clone> Deserializable for HashMap<K, V> {
    fn from_serialized(serialized: &Serialized) -> Result<(Self, usize), SerializationError> {
        let keys_result = Vec::<K>::from_serialized(serialized);
        if keys_result.is_err(){
            return Err(keys_result.err().unwrap());
        }
        let (keys, mut offset) = keys_result.unwrap();
        let values_result = Vec::<V>::from_serialized(&serialized[offset..].to_vec());
        if values_result.is_err(){
            return Err(values_result.err().unwrap());
        }
        let (values, values_offset) = values_result.unwrap();
        if values.len() != keys.len(){
            return Err(InvalidDataError("Different sizes of values and keys. Not a HashMap?"));
        }
        offset += values_offset;
        let mut result = Self::new();
        for i in 0..keys.len(){
            result.insert(keys[i].clone(), values[i].clone());
        }
        drop(keys);
        drop(values);
        Ok((result, offset))
    }
}


/* Tests begin here */
mod tests {
    use libmilkyway_derive::{Deserializable, Serializable};
    use super::*;

    macro_rules! test_serialization {
        ($($name:ident: $t:ty),*) => {
            $(
                #[test]
                fn $name() {
                    let value: $t = 42 as $t;
                    let serialized = value.serialize();
                    let (deserialized, size) = <$t>::from_serialized(&serialized).unwrap();
                    assert_eq!(value, deserialized);
                    assert_eq!(size, std::mem::size_of::<$t>());
                }
            )*
        }
    }

    test_serialization!(
        test_u16: u16,
        test_u32: u32,
        test_u64: u64,
        test_u128: u128,
        test_i8: i8,
        test_i16: i16,
        test_i32: i32,
        test_i64: i64,
        test_i128: i128,
        test_usize: usize
    );

    #[test]
    fn test_length_error() {
        let serialized = vec![0u8; 1];
        let result = u32::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::LengthError)));
    }

    #[test]
    fn test_serialize_deserialize_vec_usize() {
        let vec: Vec<usize> = vec![1, 2, 3, 4, 5];
        let serialized = vec.serialize();
        let (deserialized, size) = Vec::<usize>::from_serialized(&serialized).unwrap();
        assert_eq!(vec, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_vec_u8() {
        let vec: Vec<u8> = vec![10, 20, 30, 40, 50];
        let serialized = vec.serialize();
        let (deserialized, size) = Vec::<u8>::from_serialized(&serialized).unwrap();
        assert_eq!(vec, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_vec_i32() {
        let vec: Vec<i32> = vec![-1, -2, 3, 4, -5];
        let serialized = vec.serialize();
        let (deserialized, size) = Vec::<i32>::from_serialized(&serialized).unwrap();
        assert_eq!(vec, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_empty_vec() {
        let vec: Vec<usize> = vec![];
        let serialized = vec.serialize();
        let (deserialized, size) = Vec::<usize>::from_serialized(&serialized).unwrap();
        assert_eq!(vec, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_length_error_vec() {
        let serialized = vec![0u8; 1];
        let result = Vec::<usize>::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::LengthError)));
    }

        #[test]
    fn test_serialize_deserialize_option_none() {
        let value: Option<u32> = None;
        let serialized = value.serialize();
        let (deserialized, size) = Option::<u32>::from_serialized(&serialized).unwrap();
        assert_eq!(value, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_option_some() {
        let value: Option<u32> = Some(42);
        let serialized = value.serialize();
        let (deserialized, size) = Option::<u32>::from_serialized(&serialized).unwrap();
        assert_eq!(value, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_option_some_complex() {
        let value: Option<Vec<u8>> = Some(vec![1, 2, 3, 4, 5]);
        let serialized = value.serialize();
        let (deserialized, size) = Option::<Vec<u8>>::from_serialized(&serialized).unwrap();
        assert_eq!(value, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_option_none_complex() {
        let value: Option<Vec<u8>> = None;
        let serialized = value.serialize();
        let (deserialized, size) = Option::<Vec<u8>>::from_serialized(&serialized).unwrap();
        assert_eq!(value, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_option_length_error() {
        let serialized: Serialized = vec![]; // Empty vector
        let result = Option::<u32>::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::LengthError)));
    }

    #[test]
    fn test_option_invalid_data_error() {
        let serialized: Serialized = vec![1, 0, 0, 0]; // Incomplete data for u32
        let result = Option::<u32>::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::LengthError)));
    }
    
    #[test]
    fn test_serialize_true() {
        let value = true;
        let serialized = value.serialize();
        assert_eq!(serialized, vec![1]);
    }

    #[test]
    fn test_serialize_false() {
        let value = false;
        let serialized = value.serialize();
        assert_eq!(serialized, vec![0]);
    }

    #[test]
    fn test_deserialize_true() {
        let serialized: Serialized = vec![1];
        let (deserialized, size) = bool::from_serialized(&serialized).unwrap();
        assert_eq!(deserialized, true);
        assert_eq!(size, 1);
    }

    #[test]
    fn test_deserialize_false() {
        let serialized: Serialized = vec![0];
        let (deserialized, size) = bool::from_serialized(&serialized).unwrap();
        assert_eq!(deserialized, false);
        assert_eq!(size, 1);
    }

    #[test]
    fn test_deserialize_length_error() {
        let serialized: Serialized = vec![];
        let result = bool::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::LengthError)));
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Serializable, Deserializable)]
    struct TestKey {
        id: u32,
    }

    #[derive(Debug, PartialEq, Clone, Serializable, Deserializable)]
    struct TestValue {
        value: Vec<u8>,
    }

    #[test]
    fn test_serialize_deserialize_empty_hashmap() {
        let hashmap: HashMap<TestKey, TestValue> = HashMap::new();
        let serialized = hashmap.serialize();
        let (deserialized, size) = HashMap::<TestKey, TestValue>::from_serialized(&serialized).unwrap();
        assert_eq!(hashmap, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_serialize_deserialize_non_empty_hashmap() {
        let mut hashmap: HashMap<TestKey, TestValue> = HashMap::new();
        hashmap.insert(
            TestKey { id: 1 },
            TestValue {
                value: "value1".to_string().as_bytes().to_vec(),
            },
        );
        hashmap.insert(
            TestKey { id: 2 },
            TestValue {
                value: "value2".to_string().as_bytes().to_vec(),
            },
        );

        let serialized = hashmap.serialize();
        let (deserialized, size) = HashMap::<TestKey, TestValue>::from_serialized(&serialized).unwrap();
        assert_eq!(hashmap, deserialized);
        assert_eq!(size, serialized.len());
    }

    #[test]
    fn test_deserialize_invalid_data() {
        let serialized: Serialized = vec![1, 2, 3]; // Invalid data for HashMap
        let result = HashMap::<TestKey, TestValue>::from_serialized(&serialized);
        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn test_deserialize_mismatched_keys_values() {
        let keys = vec![
            TestKey { id: 1 },
            TestKey { id: 2 },
        ].serialize();

        let values = vec![
            TestValue {
                value: "value1".to_string().as_bytes().to_vec(),
            },
        ].serialize();

        let mut serialized = keys.clone();
        serialized.extend(values);

        let result = HashMap::<TestKey, TestValue>::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::InvalidDataError(_))));
    }

    #[test]
    fn test_deserialize_hashmap_length_error() {
        let serialized: Serialized = vec![]; // Empty vector, should result in length error
        let result = HashMap::<TestKey, TestValue>::from_serialized(&serialized);
        assert!(matches!(result, Err(SerializationError::LengthError)));
    }

    #[test]
    fn test_serialize_deserialize_large_hashmap() {
        let mut hashmap: HashMap<TestKey, TestValue> = HashMap::new();
        for i in 0..1000 {
            hashmap.insert(
                TestKey { id: i },
                TestValue {
                    value: format!("value{}", i).as_bytes().to_vec(),
                },
            );
        }

        let serialized = hashmap.serialize();
        let (deserialized, size) = HashMap::<TestKey, TestValue>::from_serialized(&serialized).unwrap();
        assert_eq!(hashmap, deserialized);
        assert_eq!(size, serialized.len());
    }
}