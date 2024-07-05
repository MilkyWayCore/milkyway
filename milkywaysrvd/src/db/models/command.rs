use diesel::{Queryable, Selectable};
use diesel::sql_types::Timestamp;

///
/// A command which is executed by certain module on remote machine
///
/// # Fields
/// * id: ID of command
/// * timestamp: time when command was issued
/// * module_id: ID of module which should handle command, 0 is reserved for milkywayd itself
/// * bytecode: Code which should be executed by module
/// * hash: Hash of timestamp, moduled_id and bytecode concatenated
/// * signature: Signed hash
/// * key_id: ID of key with which command was signed
#[derive(Queryable)]
pub struct Command{
    pub id: u64,
    pub timestamp: Timestamp,
    pub module_id: u64,
    pub bytecode: Vec<u8>,
    pub hash: Vec<u8>,
    pub signature: Vec<u8>,
}
