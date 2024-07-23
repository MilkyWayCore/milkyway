use libmilkyway::get_timestamp_with_milliseconds;
use libmilkyway::message::common::{AsMessage, Message};
use libmilkyway::message::ping::PongMessage;
use libmilkyway::message::types::MessageType;
use libmilkyway::services::transport::TransportServiceListener;
use libmilkyway::transport::{Transport, TransportListener, TransportSender};

///
/// A struct which responds to ping requests
/// 
pub struct PingResponder{
    source_id: u128,
    module_id: u64,
    sender: Box<dyn TransportSender>
}

impl PingResponder {
    pub fn new(source_id: u128, module_id: u64, sender: Box<dyn TransportSender>) -> PingResponder{
        PingResponder{
            source_id,
            module_id,
            sender,
        }
    }
}

impl TransportListener for PingResponder{
    fn on_message(&mut self, message: Message) {
        if message.message_type != MessageType::Ping{
            log::warn!("Received message of not ping type(id={}, module_id={})",
                message.id, message.module_id);
            return;
        }
        let pong = PongMessage::from_ping_message(&message);
        // We don't actually care if this message ever reaches recepient, so no reason for blocking
        // current thread/coroutine
        self.sender.send_message(pong.as_message());
    }
}