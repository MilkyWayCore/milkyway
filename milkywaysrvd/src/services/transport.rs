use std::collections::HashMap;
use tokio::sync::mpsc::{Sender, Receiver};
use libmilkyway::controllers::authorization::AuthorizationController;
use libmilkyway::message::common::Message;
use libmilkyway::services::transport::{MessageFilter, TransportService};
use libmilkyway::tokio::{tokio_block_on, tokio_spawn};
use libmilkyway::transport::{TransportListener, TransportSender};


const CHANNEL_BUFFER_SIZE: usize = 65536;

///
/// A tokio-base transport service
///
pub struct TokioTransportServiceImpl{
    /** A controller of authorization procedure **/
    authorization_controller: AuthorizationController,
    /** Receiver of all messages **/
    message_receiver: Receiver<Message>,
    /** Sender to message receiver **/
    message_sender: Sender<Message>,
    /** Sender to subscripton control **/
    subscription_ctl_sender: Sender<ServiceSubscriptionRequest>,
}

///
/// An implementation sender for tokio transport service
///
pub struct TokioTransportSenderImpl{
    tx: Sender<Message>,
}

impl TransportSender for TokioTransportSenderImpl{
    fn send_message(&mut self, message: Message) {
        let tx = self.tx.clone();
        tokio_spawn(async move {
            tx.send(message).await.expect("Can not contact TransportService");
        });
    }
}

enum ServiceSubscriptionRequest{
    /** Request to subscribe **/
    Subscribe((MessageFilter, Box<dyn TransportListener>)),
    /** Request to unsubscribe **/
    Unsubscribe(u128),
}

enum ServiceSubscriptionResponse{
    /** Generic Ok response **/
    Ok,
    /** Ok response with ID of subscription **/
    OkId(u128),
}

struct TokioTransportServiceWorker{
    listeners: HashMap<u128, Box<dyn TransportListener>>,
    subscription_ctl_rx: Receiver<ServiceSubscriptionRequest>,
    subscription_ctl_tx: Sender<ServiceSubscriptionResponse>,
    /** Receives messages from modules **/
    message_receiver: Receiver<Message>,
}

impl TokioTransportServiceImpl{
    pub fn run(authorization_controller: AuthorizationController) -> TokioTransportServiceImpl{
        let (tx, rx) = tokio::sync::mpsc::channel(CHANNEL_BUFFER_SIZE);
        let (subscription_tx, subscription_rx) = tokio::sync::mpsc::channel(CHANNEL_BUFFER_SIZE);
        TokioTransportServiceImpl{
            authorization_controller,
            message_receiver: rx,
            message_sender: tx,
            subscription_ctl_sender: subscription_tx,
        }
    }
}

impl TransportService for TokioTransportServiceImpl {
    fn subscribe_to_messages(&mut self, filter: &MessageFilter, listener: Box<dyn TransportListener>) -> u128 {
        tokio_block_on(async move {
            self.subscription_ctl_sender.send(ServiceSubscriptionRequest::Subscribe((filter.clone(), listener)))
                .await.expect("Can not send subscribe request");
        });
        todo!()
    }

    fn unsubscribe(&mut self, filter_id: u128) {
        tokio_block_on(async move { 
            self.subscription_ctl_sender.send(ServiceSubscriptionRequest::Unsubscribe(filter_id))
                .await.expect("Can not send unsubscribe request")
        });
    }

    fn get_sender(&mut self) -> Box<dyn TransportSender> {
        Box::new( TokioTransportSenderImpl{
            tx: self.message_sender.clone()
        })
    }

    fn blocking_recv(&mut self, source: u128, timeout: Option<u64>) -> Option<Message> {
        todo!()
    }
}