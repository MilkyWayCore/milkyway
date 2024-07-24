use std::cmp::min;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use libmilkyway::controllers::authorization::{AuthorizationController, AuthorizationMessage};
use libmilkyway::get_timestamp_with_milliseconds;
use libmilkyway::message::common::Message;
use libmilkyway::pki::impls::certificates::falcon1024::Falcon1024Certificate;
use libmilkyway::pki::impls::certificates::kyber1024::Kyber1024Certificate;
use libmilkyway::serialization::serializable::Serialized;
use libmilkyway::serialization::deserializable::Deserializable;
use libmilkyway::transport::async_stream::TokioStreamTransport;
use libmilkyway::transport::crypto::CryptoTransformer;
use libmilkyway::serialization::serializable::Serializable;
use crate::listeners::TokioAsyncListener;

const CHUNK_SIZE: usize = 1498;

// handle_connection would not start more than a second
const HANDLE_START_GAP_MS: u128 = 1000;

pub(crate) struct TokioTcpListener{
    local_signing_cert: Falcon1024Certificate,
    local_encryption_cert: Kyber1024Certificate,
    authorization_signing_serial: u128,
    authorization_encryption_serial: u128,
}

impl TokioTcpListener {
    async fn handle_connection(&self, mut socket: TcpStream,
                               mut authorization_controller: AuthorizationController, 
                               remote_message_tx: Sender<Message>, 
                               mut local_message_rx: Receiver<Message>){
        let min_connection_open_timestamp = get_timestamp_with_milliseconds() - HANDLE_START_GAP_MS;
        let mut size_buffer = [0u8; std::mem::size_of::<usize>()];
        let result = socket.read_exact(&mut size_buffer).await;
        if result.is_err(){
            log::error!("Error while reading from socket");
            return;
        }
        let mut bytes_to_read = usize::from_le_bytes(size_buffer);
        let mut message_buffer = [0u8; CHUNK_SIZE];
        let mut authorization_message = Serialized::new();
        while bytes_to_read > 0{
            let size_to_receive = min(bytes_to_read, CHUNK_SIZE);
            let result = socket.read(&mut message_buffer).await;
            if result.is_err(){
                log::error!("Receive failed");
                return;
            }
            let actual_bytes_count = result.unwrap();
            if actual_bytes_count != size_to_receive{
                log::error!("Expected to receive {:?} bytes got {:?}", size_to_receive, actual_bytes_count);
                return;
            }
            authorization_message.extend(message_buffer[0..actual_bytes_count].to_vec());
        }
        let message_result = AuthorizationMessage::from_serialized(&authorization_message);
        if message_result.is_err(){
            log::error!("Can not parse message");
            return;
        }
        let (message, _) = message_result.unwrap();
        if message.timestamp < min_connection_open_timestamp{
            log::error!("Authorization message is too old!");
            return;
        }
        let result = authorization_controller.check_authorization_message(message);
        if result.is_none(){
            log::error!("Authorization failed!");
            authorization_controller.finalize();
            return;
        }
        let my_message =
            authorization_controller.generate_authorization_message(self.authorization_signing_serial,
                                                                    self.authorization_encryption_serial,
                                                                    true);
        let my_message = my_message
            .expect("Can not generate authorization message")
            .serialize();
        let my_message_size: usize = my_message.len();
        let my_result = socket.write_all(&my_message_size.to_le_bytes()).await;
        if my_result.is_err(){
            log::error!("Can not send authorization message size");
            return;
        }
        let my_result = socket.write_all(&my_message).await;
        if my_result.is_err(){
            log::error!("Can not send authorization message data");
            return;
        }
        authorization_controller.finalize();
        drop(authorization_controller);
        let (signing_cert, encryption_cert) = result.unwrap();
        let mut stream_wrapper = TokioStreamTransport::from_stream(socket);
        let crypto_transformer = CryptoTransformer::new(self.local_signing_cert.clone(),
                                                            self.local_encryption_cert.clone(),
                                                            signing_cert, encryption_cert);
        stream_wrapper.add_transformer(Box::new(crypto_transformer));
        loop {
            select! {
                Some(message) = stream_wrapper.receive_raw(None) => {
                    let result = Message::from_serialized(&message);
                    if result.is_err(){
                        log::error!("Unparsable message received");
                    } else {
                        let (message, _) = result.unwrap();
                        remote_message_tx.send(message).await
                        .expect("Can not send received message");
                    }
                }
                Some(message) = local_message_rx.recv() => {
                    let serialized = message.serialize();
                    let result = stream_wrapper.send_raw(serialized).await;
                    if result.is_err(){
                        log::error!("Can not communicate with remote");
                        break;
                    }
                }
                else => {
                    log::error!("No communication streams left");
                    break;
                }
            }
        }
        log::info!("handle_connection: finished handling connection");
    }
}

#[async_trait]
impl TokioAsyncListener for TokioTcpListener{
    async fn run(&mut self, tx: Sender<Message>, rx: Receiver<Message>) {
        todo!()
    }
}