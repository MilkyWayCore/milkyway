use std::collections::HashMap;
use tokio::select;
use tokio::sync::mpsc::{Sender, Receiver};
use libmilkyway::actor::binder::{AsyncBinderChannelImpl, BinderChannel, BinderMessage};
use libmilkyway::controllers::authorization::AuthorizationController;
use libmilkyway::message::common::Message;
use libmilkyway::services::transport::{MessageFilter, TransportService};
use libmilkyway::tokio::tokio_spawn;
use libmilkyway::transport::{TransportListener, TransportSender};
use libmilkyway::unwrap_variant;
use crate::listeners::TokioAsyncListener;
use crate::services::transport::ServiceSubscriptionResponse::{NotSubscribed, OkId};

const CHANNEL_BUFFER_SIZE: usize = 65536;


/** A channel to communicate with binder **/
type WorkerBinder = dyn BinderChannel<BinderMessage<ServiceSubscriptionRequest, ServiceSubscriptionResponse>>;

///
/// A tokio-base transport service
///
pub struct TokioTransportServiceImpl{
    /** Sender to message receiver **/
    message_sender: Sender<Message>,
    /** Binder to communicate with worker **/
    worker_binder: Box<WorkerBinder>,
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


#[derive(Debug, Clone, PartialEq)]
enum ServiceSubscriptionResponse{
    /** Generic Ok response **/
    Ok,
    /** Ok response with ID of subscription **/
    OkId(u128),
    /** Request to unsubscribe from unexisten filter ID **/
    NotSubscribed,
}

struct TokioTransportServiceWorker{
    /** ID of last subscription **/
    last_id: u128,
    /** Map of listener ID to functions **/
    listeners: HashMap<u128, Box<dyn TransportListener>>,
    /** Map of listener ID to filters **/
    filters: HashMap<u128, MessageFilter>,
    binder: AsyncBinderChannelImpl<BinderMessage<ServiceSubscriptionRequest, ServiceSubscriptionResponse>>,
    /** Receives messages from modules **/
    message_receiver: Receiver<Message>,
}

impl TokioTransportServiceWorker {
    pub(crate) fn new(binder: AsyncBinderChannelImpl<BinderMessage<ServiceSubscriptionRequest, ServiceSubscriptionResponse>>, receiver: Receiver<Message>) -> Self{
        TokioTransportServiceWorker{
            last_id: 0,
            listeners: HashMap::new(),
            filters: HashMap::new(),
            message_receiver: receiver,
            binder,
        }
    }
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
                Some(message) = self.binder.rx.recv() => {
                    let message = unwrap_variant!(message, BinderMessage::Query);
                    let result = self.handle_subscription(message).clone();
                    self.binder.tx.send(BinderMessage::Response(result)).await.expect("Can not respond to service");
                }
                Some(message) = listener_rx.recv() => {
                    self.handle_remote_message(message);
                }
                Some(_) = peer_id_rx.recv() => {
                    /* stub: we need to tell nameservice about this */
                }
                else => {
                    log::error!("No opened receivers!");
                }
            }
        }
    }
}

impl TokioTransportServiceImpl{
    pub fn run<T>(listener: T) -> TokioTransportServiceImpl where T: TokioAsyncListener + 'static{
        let (tx, rx) = tokio::sync::mpsc::channel(CHANNEL_BUFFER_SIZE);
        let (local_binder, remote_binder) =
            AsyncBinderChannelImpl::<BinderMessage<ServiceSubscriptionRequest, ServiceSubscriptionResponse>>::duplex(CHANNEL_BUFFER_SIZE);
        let mut worker = TokioTransportServiceWorker::new(remote_binder, rx);
        tokio_spawn(async move {
            worker.run(listener).await;
        });
        TokioTransportServiceImpl{
            message_sender: tx,
            worker_binder: Box::new(local_binder),
        }
    }
}

impl TransportService for TokioTransportServiceImpl {
    fn subscribe_to_messages(&mut self, filter: &MessageFilter, listener: Box<dyn TransportListener>) -> u128 {
        self.worker_binder.send_message(
            BinderMessage::Query(ServiceSubscriptionRequest::Subscribe((filter.clone(), listener))));
        let response =
            unwrap_variant!(self.worker_binder.receive_message(), BinderMessage::Response);
        unwrap_variant!(response, ServiceSubscriptionResponse::OkId)
    }

    fn unsubscribe(&mut self, filter_id: u128) {
        self.worker_binder.send_message(BinderMessage::Query(ServiceSubscriptionRequest::Unsubscribe(filter_id)));
        let inner_variant = unwrap_variant!(self.worker_binder.receive_message(),
            BinderMessage::Response);
        if inner_variant != ServiceSubscriptionResponse::Ok{
            log::error!("Unexpected response for unsubscribe request: {:?}", inner_variant);
        }
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