///
/// Implementation of BinderService with tokio coroutines
///
pub mod coroutine;

use tokio::sync::mpsc::{Sender, Receiver};
use crate::tokio::tokio_block_on;

///
/// Binder channel allows communication between service binders from actor side
/// and client code side
///
pub trait BinderChannel<T>: Send + Sync where T: Send + Sync{

    ///
    /// Sends message to remote binder in blocking manner
    ///
    /// # Arguments
    /// * message: T: a message to send
    ///
    /// # Panics
    /// * If sending error occurs
    ///
    fn send_message(&mut self, message: T);

    ///
    /// Receives message in blocking manner and returns it
    ///
    /// # Panics
    /// * If receiving error occurs
    ///
    /// * returns: T: message received
    fn receive_message(&mut self) -> T;

    ///
    /// Checks whether binder channel is alive
    ///
    /// * returns: bool: true if channel alive, false otherwise
    fn is_alive(&self) -> bool;
}

///
/// A standard message with querying and responding to Binder requests
///
pub enum BinderMessage<Q: Send + Sync, R: Send + Sync>{
    ///
    /// Custom query message
    ///
    Query(Q),

    ///
    /// Custom response message
    ///
    Response(R),

    ///
    /// Message which is used to unbind Binder
    ///
    Unbind
}

///
/// The Binder itself. Allows doing RPC calls to a remote service.
///
/// # Template arguments
/// * Q: request message type
/// * R: response message type
///
pub trait Binder<Q: Send + Sync, R: Send + Sync>: Send + Sync{
    ///
    /// Executes RPC call for request Q and waits for result
    ///
    /// # Arguments
    /// * request: Q: request message
    ///
    /// returns: R: response message
    ///
    fn handle_request(&mut self, request: Q) -> R;

    ///
    /// Unbinds this binder from service
    ///
    fn unbind(&mut self);

}


///
/// A handler that used to receive messages and execute RPC commands
///
pub trait BinderServiceHandler<Q, R>: Send + Sync where Q: Send + Sync, R: Send + Sync{
    fn handle_message(&mut self, request: Q) -> R;
}

impl<Q, R> Binder<Q, R> for dyn BinderChannel<BinderMessage<Q, R>>
    where Q: Sync + Send, R: Sync + Send
{
    #[allow(unused_assignments)]
    fn handle_request(&mut self, request: Q) -> R {
        self.send_message(BinderMessage::Query(request));
        let result = self.receive_message();
        let mut function_result = Option::<R>::None;
        match result {
            BinderMessage::Unbind => {
                panic!("Service-side unbind is not supported");
            }
            BinderMessage::Query(_) => {
                panic!("Received query from service");
            }
            BinderMessage::Response(response) => {
                function_result = Some(response)
            }
        }
        function_result.unwrap()
    }

    #[inline]
    fn unbind(&mut self) {
        self.send_message(BinderMessage::Unbind);
    }
}

///
/// Asynchronous binder channel
///
/// # Template arguments
/// * T: Message type used inside channel
///
pub struct AsyncBinderChannelImpl<T: Send + Sync>{
    // signal_tx allows to tell remote service that message has came.
    signal_tx: Option<Sender<bool>>,
    pub tx: Sender<T>,
    pub rx: Receiver<T>,
}

impl<T> AsyncBinderChannelImpl<T> where T: Send + Sync {
    pub fn new(signal_tx: Option<Sender<bool>>, tx: Sender<T>, rx: Receiver<T>) -> Self {
        Self { signal_tx, tx, rx }
    }
}

impl<T> BinderChannel<T> for AsyncBinderChannelImpl<T> where T: Send + Sync{
    #[inline]
    fn send_message(&mut self, message: T) {
        tokio_block_on(async move {
           self.tx.send(message).await.unwrap();
           if self.signal_tx.is_some() {
               self.signal_tx.as_mut().unwrap().send(true).await.unwrap();
           }
        });
    }

    #[inline]
    fn receive_message(&mut self) -> T {
        tokio_block_on(async move {
            self.rx.recv().await
        }).unwrap()
    }

    #[inline]
    fn is_alive(&self) -> bool {
        !self.tx.is_closed() && !self.rx.is_closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc::channel;
    use crate::tokio::{init_tokio, tokio_spawn};

    #[test]
    fn test_send_receive_message() {
        init_tokio();
        let (dummy_tx, _) = channel::<bool>(1);
        let (_service_tx, client_rx) = channel::<String>(10);
        let (client_tx, mut service_rx) = channel::<String>(10);
        let mut binder_channel = AsyncBinderChannelImpl::new(None, client_tx, client_rx);

        let send_message = "Hello, World!".to_string();
        binder_channel.send_message(send_message.clone());

        let received_message: String = tokio_block_on(async move {
            service_rx.recv().await.unwrap()
        });
        assert_eq!(send_message, received_message);
    }

    #[tokio::test]
    async fn test_is_alive() {
        let (dummy_tx, _) = channel::<bool>(1);
        let (tx, rx) = channel::<String>(10);
        let binder_channel = AsyncBinderChannelImpl::new(None, tx, rx);

        assert!(binder_channel.is_alive());
    }

    type TestMessage = BinderMessage<u8, u8>;

    #[test]
    fn test_handle_request() {
        init_tokio();
        let (dummy_tx, _) = channel::<bool>(1);
        let (service_tx, client_rx) = channel::<TestMessage>(10);
        let (client_tx, mut service_rx) = channel::<TestMessage>(10);
        let binder_channel: &mut dyn BinderChannel<TestMessage> = &mut AsyncBinderChannelImpl::<TestMessage>::new(None, client_tx, client_rx) as &mut dyn BinderChannel<TestMessage>;

        let request = 27;
        let response = 42;

        tokio_spawn(async move {
            let received_message = service_rx.recv().await.unwrap();
            if let BinderMessage::Query(req) = received_message {
                assert_eq!(req, request);
                service_tx.send(BinderMessage::Response(response.clone())).await.unwrap();
            }
        });


        let result = binder_channel.handle_request(request);
        assert_eq!(result, response);
    }
}

