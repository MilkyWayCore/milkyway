//Have you ever wondered how typical C++ code looks like?
//Code below is almost like C++ :)
//It needs refactoring (very much)
use std::collections::HashMap;

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::actor::binder::{AsyncBinderChannelImpl, BinderChannel, BinderMessage, BinderServiceHandler};
use crate::actor::binder::coroutine::BinderAsyncServiceMessage::{BindRequest, BindResponse, ControlTx, SignalTx};
use crate::services::certificate::{CertificateServiceBinderRequest, CertificateServiceBinderResponse};
use crate::tokio::{tokio_block_on, tokio_spawn};
use crate::unwrap_variant;

///
/// A message for internal Async Binder communication
///
pub enum BinderAsyncServiceMessage<Q, R> where Q: Send + Sync, R: Send + Sync{
    BindRequest(Sender<BinderMessage<Q, R>>),
    StopRequest,
    BindResponse(Sender<BinderMessage<Q, R>>),
    ControlTx(Sender<Self>),
    SignalTx(Sender<bool>),
}


///
/// Async service wrapping handler to do RPC calls
///
pub struct BinderAsyncService<Q, R> where Q: Send + Sync, R: Send + Sync{
    signal_tx: Sender<bool>,
    control_tx: Option<Sender<BinderAsyncServiceMessage<Q, R>>>,
    control_rx: Option<Receiver<BinderAsyncServiceMessage<Q, R>>>
}

const ASYNC_BINDER_SERVICE_CHANNEL_BUFSIZE: usize = 128;

impl<Q, R> BinderAsyncService<Q, R> where Q: Send + Sync + 'static, R: Send + Sync + 'static{
    ///
    /// Creates a service with given handler and starts it.
    ///
    /// # Arguments
    /// * handler: A handler to handle queries
    ///
    pub fn run(mut handler: Box<dyn BinderServiceHandler<Q, R>>) -> Self{
        let (service_tx, mut control_rx) = channel::<BinderAsyncServiceMessage<Q, R>>(ASYNC_BINDER_SERVICE_CHANNEL_BUFSIZE);
        tokio_spawn(async move {
            let (control_tx, mut service_rx) = channel::<BinderAsyncServiceMessage<Q, R>>(ASYNC_BINDER_SERVICE_CHANNEL_BUFSIZE);
            let (signal_tx, mut signal_rx) = channel::<bool>(ASYNC_BINDER_SERVICE_CHANNEL_BUFSIZE);
            service_tx.send(ControlTx(control_tx)).await.expect("Can not send control transmitter");
            service_tx.send(SignalTx(signal_tx.clone())).await.expect("Can not send signal transmitter");
            let mut last_bind_id: usize = 0;
            let mut binder_channels  = HashMap::<usize, AsyncBinderChannelImpl::<BinderMessage<Q, R>>>::new();
            loop {
                signal_rx.recv().await.expect("Signal communication failure");
                println!("New message");
                let message = service_rx.try_recv();
                if message.is_ok() {
                    let message = message.unwrap();
                    match message {
                        BinderAsyncServiceMessage::BindRequest(local_tx) => {
                            let (remote_tx, local_rx) = channel::<BinderMessage<Q, R>>(ASYNC_BINDER_SERVICE_CHANNEL_BUFSIZE);
                            let channel = AsyncBinderChannelImpl::new(None,
                                                                      local_tx, local_rx);
                            binder_channels.insert(last_bind_id, channel);
                            last_bind_id += 1;
                            service_tx.send(BindResponse(remote_tx)).await.unwrap();
                        }
                        BinderAsyncServiceMessage::StopRequest => {
                            break;
                        }
                        BinderAsyncServiceMessage::BindResponse(_) => {
                            panic!("Invalid message: BindResponse");
                        }
                        ControlTx(_) => {
                            panic!("Invalid message: ControlTx");
                        }
                        BinderAsyncServiceMessage::SignalTx(_) => {
                            panic!("Invalid message: SignalTx");
                        }
                    }
                }
                let mut unbinded: Vec<usize> = Vec::new();
                for (key, channel) in binder_channels.iter_mut(){
                    let message = channel.rx.try_recv();
                    if message.is_err(){
                        continue;
                    }
                    let message = message.unwrap();
                    match message {
                        BinderMessage::Query(query) => {
                            channel.tx.send(
                                BinderMessage::Response(handler.handle_message(query))
                            ).await.unwrap();
                        }
                        BinderMessage::Response(_) => {}
                        BinderMessage::Unbind => {
                            println!("Unbind message");
                            let key_clone = key;
                            unbinded.push(*key_clone)
                        }
                    }
                }
                println!("unbinded={:?}", unbinded);
                for key in unbinded.iter(){
                    println!("Removing {:?}", key);
                    let mut channel = binder_channels.get_mut(&key).unwrap();
                    channel.tx.closed();
                    channel.rx.close();
                    binder_channels.remove(&key);
                }
                println!("iter");
            }
            panic!("Service died!");
        });
        let (msg_ctl, msg_sig) = tokio_block_on(async {
            (control_rx.recv().await.unwrap(), control_rx.recv().await.unwrap())
        });
        let control_tx = unwrap_variant!(msg_ctl, ControlTx);
        let signal_tx = unwrap_variant!(msg_sig, SignalTx);
        Self{
            control_tx: Some(control_tx),
            control_rx: Some(control_rx),
            signal_tx,
        }

    }

    ///
    /// Creates new binder channel and returns it to caller
    ///
    /// * returns: Implementation of async binder channel
    ///
    pub fn bind(&mut self) -> Box<dyn BinderChannel<BinderMessage<Q, R>>>{
        let (service_tx, local_rx) = channel::<BinderMessage<Q,R>>(ASYNC_BINDER_SERVICE_CHANNEL_BUFSIZE);
        let ctl_tx = self.control_tx.clone().unwrap();
        tokio_block_on(async{
            ctl_tx.send(BindRequest(service_tx)).await.unwrap();
            self.signal_tx.clone().send(true).await.unwrap();
        });
        let recv_coroutine = self.control_rx.as_mut().unwrap().recv();
        let local_tx = tokio_block_on(recv_coroutine).unwrap();
        let local_tx= unwrap_variant!(local_tx, BindResponse);
        let result = AsyncBinderChannelImpl::new(Some(self.signal_tx.clone()), local_tx, local_rx);
        Box::new(result)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use super::*;
    use tokio::sync::mpsc::channel;
    use tokio::time::sleep;
    use crate::actor::binder::{AsyncBinderChannelImpl, Binder, BinderChannel, BinderMessage, BinderServiceHandler};
    use crate::actor::binder::BinderMessage::Unbind;
    use crate::actor::binder::coroutine::BinderAsyncServiceMessage::{BindRequest, BindResponse, ControlTx, SignalTx};
    use crate::tokio::{init_tokio, tokio_block_on, tokio_spawn};
    use crate::unwrap_variant;

    struct TestHandler;

    impl BinderServiceHandler<u8, u8> for TestHandler {
        fn handle_message(&mut self, request: u8) -> u8 {
            request + 1
        }
    }

    #[test]
     fn test_run_service() {
        init_tokio();
        let handler = Box::new(TestHandler);
        let service = BinderAsyncService::run(handler);

        assert!(service.control_tx.is_some());
        assert!(service.control_rx.is_some());
    }

    #[test]
     fn test_bind_to_service() {
        init_tokio();
        let handler = Box::new(TestHandler);
        let mut service = BinderAsyncService::run(handler);

        let mut binder_channel = service.bind();

        assert!(binder_channel.is_alive());

        // Test sending a request and receiving the correct response
        let request = 27;
        let expected_response = 28;

        binder_channel.send_message(BinderMessage::Query(request));
        let response = binder_channel.receive_message();

        if let BinderMessage::Response(res) = response {
            assert_eq!(res, expected_response);
        } else {
            panic!("Expected a response");
        }
    }

    #[test]
    fn test_unbind_from_service() {
        init_tokio();
        let handler = Box::new(TestHandler);
        let mut service = BinderAsyncService::run(handler);

        let mut binder_channel = service.bind();

        assert!(binder_channel.is_alive());

        binder_channel.send_message(Unbind);
        // Give some time to handle unbind
        tokio_block_on(async {
            sleep(Duration::from_millis(30)).await;
        });

        // Unbinding the channel should make it not alive anymore
        assert!(!binder_channel.is_alive());
    }

    #[test]
    fn test_handle_request() {
        init_tokio();
        let handler = Box::new(TestHandler);
        let mut service = BinderAsyncService::run(handler);

        let mut binder_channel = service.bind();

        let request = 42;
        let expected_response = 43;

        let response = binder_channel.handle_request(request);

        assert_eq!(response, expected_response);
    }
}
