use crate::pki::hash::{CryptoHashable, Hash, HashType};
use crate::serialization::serializable::Serializable;

impl<T> CryptoHashable for T where T: Serializable{
    fn crypto_hash(&self, hash_type: HashType) -> Hash {
        match hash_type {
            HashType::None => {
                Hash {
                    algorithm: HashType::None,
                    hash: vec![0],
                }
            }
            HashType::SHA512 => { todo!() }
        }
    }
}