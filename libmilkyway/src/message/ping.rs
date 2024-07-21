use crate::message::common::{AsMessage, Message};
use crate::message::types::MessageType;
use crate::serialization::serializable::Serializable;


///
/// A dummy struct for ping message all needed data may be determined from
/// message headers.
/// 
pub struct PingMessage;

impl PingMessage{
    ///
    /// Creates new ping message
    /// 
    pub fn new() -> PingMessage{
        PingMessage{}
    }
}

impl AsMessage for PingMessage{
    fn as_message(&self) -> Message {
        Message{
            id: 0,
            timestamp: 0,
            message_type: MessageType::Ping,
            data: None,
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        }
    }
}

///
/// A response to ping. Has an ID from message headers on which ping exactly we reply
/// 
pub struct PongMessage{
    pub ping_message_id: u128,
}

impl PongMessage{
    ///
    /// Creates a response to a ping message
    /// 
    pub fn new(message_id: u128) -> PongMessage{
        PongMessage{
            ping_message_id: message_id,
        }
    }
    
    ///
    /// Creates a response to a ping message from a ping message itself
    /// 
    #[inline]
    pub fn from_ping_message(msg: &Message) -> PongMessage{
        PongMessage::new(msg.id)
    }
}

impl AsMessage for PongMessage{
    fn as_message(&self) -> Message {
        Message{
            id: 0,
            timestamp: 0,
            message_type: MessageType::Ping,
            data: Some(self.ping_message_id.serialize()),
            signature: None,
            source: 0,
            destination: 0,
            module_id: 0,
        }
    }
}
