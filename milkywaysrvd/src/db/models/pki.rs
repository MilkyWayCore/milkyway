use diesel::Queryable;

///
/// A key for PKI infrastructure
///
/// # Fields
/// * id: Unique id number of key
/// * key: Byte encoded key
/// * hash: Hash of the key
/// * signature: signature of the key
/// * parent_key: a key with which this one is signed, null in case of a root key
///
/// # Notes
/// Key validity should be specified in key byte array itself
#[derive(Queryable)]
pub struct Key {
    pub id: u64,
    pub key: Vec<u8>,
    pub hash: Vec<u8>,
    pub signature: Vec<u8>,
    pub parent_key: Option<u64>,
}