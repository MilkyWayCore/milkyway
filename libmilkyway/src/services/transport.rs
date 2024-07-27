use crate::message::common::Message;
use crate::transport::{TransportListener, TransportSender};

///
/// A struct for filtering messages.
/// The operator between fields is AND
///
#[derive(Clone)]
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
/// A transport service trait which allows access to communications for
/// modules
/// 
pub trait TransportService: Send + Sync{
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
                             listener: Box<dyn TransportListener>) -> u128;
    
    ///
    /// Unsubscribes from messages
    ///
    /// # Arguments
    /// * filter_id: u128: ID of filter to unsubscribe
    ///
    fn unsubscribe(&mut self, filter_id: u128);

    ///
    /// Gets a global transport sender allowing to send messages
    /// anywhere
    ///
    /// returns: Box-ed TransportSender trait object
    fn get_sender(&mut self) -> Box<dyn TransportSender>;
    
    
    ///
    /// Sends a message using built-in sender
    /// 
    /// # Arguments
    /// * message: Message: message to be sent
    /// 
    #[inline]
    fn send_message(&mut self, message: Message){
        let mut sender = self.get_sender();
        sender.send_message(message);
    }
}