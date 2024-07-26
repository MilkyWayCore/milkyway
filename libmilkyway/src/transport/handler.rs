use crate::actor::binder::{BinderChannel, BinderMessage};
use crate::message::common::Message;
use crate::services::transport::MessageFilter;
use crate::transport::TransportListener;
use crate::transport::worker::TransportWorker;
use crate::unwrap_variant;


///
/// A request to transport handler from service
///
pub enum TransportHandlerRequest{
    /** New worker added **/
    NewWorker((u128, Box<TransportHandlerWorkerBinder>)),

    /** Adds new listener with given filter **/
    AddListener((MessageFilter, Box<dyn TransportListener>)),

    /** Sends a message **/
    SendMessage(Message)
}

///
/// A response from transport handler to service
///
#[derive(PartialEq, Debug)]
pub enum TransportHandlerResponse{
    /** Operation completed, the ID is the result **/
    OkId(u128),

    /** Operation is completed, no usable results **/
    Ok,
}

///
/// A binder of worker to handler and handler to worker which allows
/// to communicate in a duplex way.
///
pub type TransportHandlerWorkerBinder = dyn BinderChannel<TransportWorkerBinderMessage>;

impl TransportWorker for TransportHandlerWorkerBinder {
    fn on_bind_to_handler(&mut self, binder: Box<TransportHandlerWorkerBinder>) {
        self.send_message(TransportWorkerBinderMessage::BindedToHandler(binder));
    }
}

///
/// A message between handler and worker
///
pub enum TransportWorkerBinderMessage{
    /** Tells worker that now it is binded to handler **/
    BindedToHandler(Box<TransportHandlerWorkerBinder>),

    /** Sends message in duplex sides **/
    Msg(Message),
}

pub type TransportHandlerServiceBinder = dyn BinderChannel<BinderMessage<TransportHandlerRequest, TransportHandlerResponse>>;

///
/// A transport handler manages all communications with the workers
///
pub trait TransportHandler: Send + Sync{
    ///
    /// The function SHOULD be called whenever new worker is created and bind for it is called
    ///
    /// # Arguments
    /// * remote_id: u128: the ID of remote peer we are communicating with
    /// * worker_channel: A binder channel which allows to handler-worker communication
    ///
    fn on_new_worker_binded(&mut self, remote_id: u128, 
                            worker_channel: Box<TransportHandlerWorkerBinder>);

    ///
    /// The function adds new message listener with given filter
    ///
    /// # Arguments
    /// * filter: MessageFilter: a filter for messages
    /// * listener: Box<dyn TransportListener>: a Box-ed listener for messages
    ///
    fn add_listener(&mut self, filter: MessageFilter, listener: Box<dyn TransportListener>) -> u128;

    ///
    /// This function sends a message.
    /// The TranportHandler MUST use message destination field to find a proper worker for sending it.
    ///
    /// # Arguments
    /// * message: Message: a message to send
    ///
    fn send(&mut self, message: Message);
}

impl TransportHandler for TransportHandlerServiceBinder{
    fn on_new_worker_binded(&mut self, remote_id: u128, worker_channel: Box<TransportHandlerWorkerBinder>) {
        self.send_message(BinderMessage::Query(TransportHandlerRequest::NewWorker((remote_id, worker_channel))));
        let result = unwrap_variant!(self.receive_message(), BinderMessage::Response);
        if result != TransportHandlerResponse::Ok{
            log::error!("on_new_worker_binded: bad response: {:?}", result);
        }
    }

    fn add_listener(&mut self, filter: MessageFilter, listener: Box<dyn TransportListener>) -> u128 {
        self.send_message(BinderMessage::Query(TransportHandlerRequest::AddListener((filter, listener))));
        let result = unwrap_variant!(self.receive_message(), BinderMessage::Response);
        unwrap_variant!(result, TransportHandlerResponse::OkId)
    }

    fn send(&mut self, message: Message) {
        self.send_message(BinderMessage::Query(TransportHandlerRequest::SendMessage(message)));
        let result = unwrap_variant!(self.receive_message(), BinderMessage::Response);
        if result != TransportHandlerResponse::Ok{
            log::error!("send: result {:?} is not Ok", result);
        }
    }
}



