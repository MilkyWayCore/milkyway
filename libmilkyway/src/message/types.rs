use crate::serialization::error::SerializationError;
use crate::serialization::serializable::Serialized;
use libmilkyway_derive::{EnumDeserializable, EnumSerializable};
use crate::serialization::serializable::Serializable;
use crate::serialization::deserializable::Deserializable;

///
/// Message type.
/// Defines a type of messages being sent.
/// 
#[derive(EnumSerializable, EnumDeserializable, Clone, Debug, PartialEq)]
pub enum MessageType{
    ///
    /// Ping request from other host
    /// 
    Ping,
    ///
    /// Ping response from other host
    /// 
    Pong,
    ///
    /// Request to execute command on a remote server
    /// 
    Exec,
    ///
    /// Request to apply state on a remote server
    /// 
    StateApply,
    ///
    /// Request to revert state on a remote server
    /// 
    StateRevert,
    ///
    /// Report about execution, state application or reversion
    /// 
    Report,
    ///
    /// Key Exchange, message containing key data
    /// 
    KeyEx,
    ///
    /// Log message, contains information about something happend in network
    /// 
    LogMessage,
    ///
    /// Acknowledged, tells that some message was received. MUST NOT be sent by server.
    /// 
    Ack,
    ///
    /// Set peer ID in the network
    /// 
    SetPeerID
}