use crate::message::common::Message;
use crate::transport::{Transport, TransportChannel};

///
/// A struct for filtering messages.
/// The operator between fields is AND
///
pub struct MessageFilter{
    pub from_id: Option<u128>,
    pub module_id: Option<u64>,
}

impl MessageFilter {
    ///
    /// Creates new empty message filter
    ///
    pub fn new() -> MessageFilter{
        MessageFilter{
            from_id: None,
            module_id: None,
        }
    }

    ///
    /// Add filter on source id
    ///
    /// # Arguments
    /// * id: u128: source to wait messages for
    ///
    /// returns: reference to self
    ///
    pub fn filter_from(&mut self, id: u128) -> &Self {
        self.from_id = Some(id);
        self
    }

    ///
    /// Add filter on module id
    ///
    /// # Arguments
    /// * id: u128: ID of module to filter by
    ///
    /// returns: reference to self
    ///
    pub fn filter_module(&mut self, id: u64) -> &Self {
        self.module_id = Some(id);
        self
    }
}

///
/// A trait which is capable of receiving messages and handling them
///
pub trait TransportServiceListener: Send + Sync{
    ///
    /// Called when message is received 
    /// 
    /// # Arguments
    /// * message: Message: the message which was received
    fn on_receive_message(&mut self, message: Message);
}

///
/// A transport service trait which allows access to communications for
/// modules
/// 
pub trait TransportService: Send + Sync{
    ///
    /// Gets direct transport to peer with given ID
    /// 
    /// # Arguments
    /// * id: u128: ID to get transport to
    /// 
    /// returns: Boxed transport or None if peer does not exist
    /// 
    fn get_transport_channel(&mut self, id: u128) -> Option<Box<dyn TransportChannel>>;

    ///
    /// Gets a global transport controller which allows to send messages anywhere
    ///
    /// returns: Boxed transport
    ///
    fn get_transport(&mut self) -> Box<dyn Transport>;
    
    ///
    /// Subscribes to messages with given message filter and listener
    /// 
    /// # Arguments
    /// * filter: MessageFilter: a filter for messages
    /// * listener: A listener used for getting
    /// 
    /// returns: u128: an ID of filter
    /// 
    fn subscribe_to_messages(&mut self,
                             filter: &MessageFilter,
                             listener: Box<dyn TransportServiceListener>) -> u128;
    
    ///
    /// Unsubscribes from messages
    ///
    /// # Arguments
    /// * filter_id: u128: ID of filter to unsubscribe
    ///
    fn unsubscribe(&mut self, filter_id: u128);
}