use crate::serialization::error::SerializationError;
use crate::serialization::deserializable::Deserializable;
use crate::serialization::serializable::Serializable;
use libmilkyway_derive::{Deserializable, Serializable};
use crate::message::common::{AsMessage, Message};
use crate::message::types::MessageType;
use crate::serialization::serializable::Serialized;

///
/// Command for executing something
/// 
#[derive(Serializable, Deserializable)]
pub struct ExecData{
    ///
    /// ID of module to which command is sent
    /// 
    pub module_id: u64,
    /// 
    /// Data provided by module about command
    /// 
    pub cmd_data: Serialized
}

impl ExecData{
    ///
    /// Creates new ExecData message data from provided id and data
    /// 
    /// # Arguments
    /// * module_id: u64: ID of module to which request is sent
    /// * data: &T: A serializable data
    pub fn new<T: Serializable>(module_id: u64, data: &T) -> Self{
        ExecData{
            module_id,
            cmd_data: data.serialize(),
        }
    }
}

impl AsMessage for ExecData{
    fn as_message(&self) -> Message {
        Message{
            id: 0,
            timestamp: 0,
            message_type: MessageType::Exec,
            data: Some(self.serialize()),
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,        }
    }
}