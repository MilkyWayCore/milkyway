use std::collections::HashMap;
use tokio::select;
use tokio::sync::mpsc::{Sender, Receiver};
use libmilkyway::controllers::authorization::AuthorizationController;
use libmilkyway::message::common::Message;
use libmilkyway::message::types::MessageType::Pong;
use libmilkyway::services::transport::{MessageFilter, TransportService};
use libmilkyway::tokio::{tokio_block_on, tokio_spawn};
use libmilkyway::transport::{TransportListener, TransportSender};
use crate::listeners::TokioAsyncListener;
use crate::services::transport::ServiceSubscriptionResponse::{NotSubscribed, OkId};

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
pub(crate) struct TokioTransportSenderImpl{
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


#[derive(Debug, Clone)]
enum ServiceSubscriptionResponse{
    /** Generic Ok response **/
    Ok,
    /** Ok response with ID of subscription **/
    OkId(u128),
    /** Request to unsubscribe from unexisten filter ID **/
    NotSubscribed,
}

struct TokioTransportServiceWorker{
    last_id: u128,
    listeners: HashMap<u128, Box<dyn TransportListener>>,
    filters: HashMap<u128, MessageFilter>,
    subscription_ctl_rx: Receiver<ServiceSubscriptionRequest>,
    subscription_ctl_tx: Sender<ServiceSubscriptionResponse>,
    /** Receives messages from modules **/
    message_receiver: Receiver<Message>,
}

impl TokioTransportServiceWorker {
    fn handle_subscription(&mut self, request: ServiceSubscriptionRequest) -> ServiceSubscriptionResponse{
        match request {
            ServiceSubscriptionRequest::Subscribe(listener) => {
                self.last_id += 1;
                let (filter, receiver) = listener;
                self.listeners.insert(self.last_id, receiver);
                self.filters.insert(self.last_id, filter);
                OkId(self.last_id)
            }
            ServiceSubscriptionRequest::Unsubscribe(id) => {
                if !self.listeners.contains_key(&id){
                    NotSubscribed
                } else {
                    self.listeners.remove(&id);
                    self.filters.remove(&id);
                    ServiceSubscriptionResponse::Ok
                }
            }
        }
    }

    fn handle_remote_message(&mut self, message: Message){
        for (key, listener) in self.listeners.iter_mut(){
            let filter = self.filters.get(key).expect("What a terrible failure: no filter for listener");
            if filter.module_id.is_some(){
                if message.module_id != filter.module_id.unwrap(){
                    continue;
                }
            }
            if filter.from_id.is_some(){
                if message.source != filter.from_id.unwrap(){
                    continue;
                }
            }
            listener.on_message(message.clone());
        }
    }


    pub async fn run<T: TokioAsyncListener + 'static>(&mut self, mut listener: T){
        let (listener_tx, mut listener_rx) = tokio::sync::mpsc::channel::<Message>(CHANNEL_BUFFER_SIZE);
        let (messages_tx, messages_rx) = tokio::sync::mpsc::channel::<Message>(CHANNEL_BUFFER_SIZE);
        let (peer_id_tx, mut peer_id_rx) = tokio::sync::mpsc::channel::<u128>(CHANNEL_BUFFER_SIZE);
        tokio::spawn(async move {
            listener.run(listener_tx, messages_rx, peer_id_tx).await;
        });
        loop {
            select! {
                Some(message) = self.message_receiver.recv() => {
                    messages_tx.send(message).await.expect("Can not communicate with listener");
                }
                Some(message) = self.subscription_ctl_rx.recv() => {
                    let result = self.handle_subscription(message).clone();
                    self.subscription_ctl_tx.send(result).await.expect("Can not respond to service");
                }
                Some(message) = listener_rx.recv() => {
                    self.handle_remote_message(message);
                }
                else => {
                    log::error!("No opened receivers!");
                }
            }
        }
    }
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