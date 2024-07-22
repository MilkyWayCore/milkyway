use libmilkyway::get_timestamp_with_milliseconds;
use libmilkyway::message::common::{AsMessage, Message};
use libmilkyway::message::ping::PongMessage;
use libmilkyway::message::types::MessageType;
use libmilkyway::services::transport::TransportServiceListener;
use libmilkyway::transport::Transport;

///
/// A struct which responds to ping requests
/// 
pub struct PingResponder{
    source_id: u128,
    module_id: u64,
    transport: Box<dyn Transport>,
}

impl PingResponder {
    pub fn new(source_id: u128, module_id: u64, transport: Box<dyn Transport>) -> PingResponder{
        PingResponder{
            source_id,
            module_id,
            transport,
        }
    }
}

impl TransportServiceListener for PingResponder{
    fn on_receive_message(&mut self, message: Message) {
        if message.message_type != MessageType::Ping{
            log::warn!("Received message of not ping type(id={}, module_id={})",
                message.id, message.module_id);
            return;
        }
        let pong = PongMessage::from_ping_message(&message);
        // We don't actually care if this message ever reaches recepient, so no reason for blocking
        // current thread/coroutine
        self.transport.send_non_blocking(message.source, pong.as_message());
    }
}