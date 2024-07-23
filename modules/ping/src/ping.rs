use libmilkyway::message::common::Message;
use libmilkyway::message::types::MessageType;
use libmilkyway::services::transport::TransportService;
use libmilkyway::transport::TransportSender;

pub(crate) fn ping(service: &mut Box<dyn TransportService>, 
                   sender: &mut Box<dyn TransportSender>, 
                   target: u128, timeout: u64){
    let ping_message = Message::new()
        .set_current_timestamp()
        .set_destination(target)
        .set_type(MessageType::Ping);
    sender.send_message(ping_message.clone());
    let msg = service.blocking_recv(target, Some(timeout));
    if msg.is_some(){
        println!("Got message");
    } else {
        println!("Timeout");
    }
}