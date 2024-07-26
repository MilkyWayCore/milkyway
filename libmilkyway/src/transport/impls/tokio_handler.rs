use std::collections::HashMap;
use log::error;
use tokio::sync::mpsc::Receiver;
use crate::actor::binder::{AsyncBinderChannelImpl, BinderChannel, BinderMessage};
use crate::message::common::Message;
use crate::services::transport::MessageFilter;
use crate::transport::handler::{TransportHandler, TransportHandlerRequest, TransportHandlerResponse, TransportHandlerServiceBinder, TransportHandlerWorkerBinder, TransportWorkerBinderMessage};
use crate::transport::TransportListener;
use crate::unwrap_variant;

struct ListenerHandle{
    filter: MessageFilter,
    listener: Box<dyn TransportListener>,
    binder: Box<TokioTransportHandlerServiceBinder>,
}

pub struct TokioTransportHandlerImpl{
    workers: HashMap<u128, Box<TransportHandlerWorkerBinder>>,
    listeners: HashMap<u128, ListenerHandle>,
    service_binder: Box<TokioTransportHandlerServiceBinder>,
    merged_workers_stream: Option<Receiver<TransportWorkerBinderMessage>>,
    merged_listeners_stream: Option<Receiver<BinderMessage<TransportHandlerRequest, TransportHandlerResponse>>>,
}

pub type TokioTransportHandlerServiceBinder = AsyncBinderChannelImpl<BinderMessage<TransportHandlerRequest, TransportHandlerResponse>>;

impl TokioTransportHandlerImpl {
    pub fn new(binder: Box<TokioTransportHandlerServiceBinder>) -> Self{
        TokioTransportHandlerImpl{
            workers: HashMap::new(),
            listeners: HashMap::new(),
            service_binder: binder,
            merged_workers_stream: None,
            merged_listeners_stream: None,
        }
    }

    pub async fn run(&mut self){

    }

    async fn handle_message_no_merged(&mut self){
        let message = self.service_binder.rx.recv().await;
        if message.is_none(){
            log::error!("Can not read binder message");
        }
        let message = message.unwrap();
        let message = unwrap_variant!(message, BinderMessage::Query);
        match message {
            TransportHandlerRequest::NewWorker((worker_id, binder)) => {

            }
            TransportHandlerRequest::AddListener((filter, listener)) => {

            }
            TransportHandlerRequest::SendMessage(_) => {
                log::error!("Somebody is trying to send a message, but now workers listen us");
            }
        }
    }
}

