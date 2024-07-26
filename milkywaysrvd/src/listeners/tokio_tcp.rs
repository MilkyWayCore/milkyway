use std::cmp::min;
use std::sync::Arc;
use std::sync::mpsc::channel;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use libmilkyway::actor::binder::BinderChannelProvider;
use libmilkyway::controllers::authorization::{AuthorizationController, AuthorizationMessage};
use libmilkyway::get_timestamp_with_milliseconds;
use libmilkyway::message::common::Message;
use libmilkyway::message::types::MessageType;
use libmilkyway::pki::impls::certificates::falcon1024::Falcon1024Certificate;
use libmilkyway::pki::impls::certificates::kyber1024::Kyber1024Certificate;
use libmilkyway::serialization::serializable::Serialized;
use libmilkyway::serialization::deserializable::Deserializable;
use libmilkyway::transport::async_stream::TokioStreamTransport;
use libmilkyway::transport::crypto::CryptoTransformer;
use libmilkyway::serialization::serializable::Serializable;
use libmilkyway::services::certificate::{CertificateService, CertificateServiceBinderProvider};
use crate::listeners::TokioAsyncListener;

const CHUNK_SIZE: usize = 1498;

/** How many milliseconds connection may take. This constant SHOULD be module-specific as 
    depending on protocol the value of connection time may differ **/
const HANDLE_START_GAP_MS: u128 = 1000;

pub(crate) struct TokioTcpListener {
    local_signing_cert: Falcon1024Certificate,
    local_encryption_cert: Kyber1024Certificate,
    authorization_signing_serial: u128,
    authorization_encryption_serial: u128,
    bind_address: String,
    certificate_service_binder_provider: Arc<Mutex<Box<CertificateServiceBinderProvider>>>,
}

struct TokioTcpWorker{
    local_signing_cert: Falcon1024Certificate,
    local_encryption_cert: Kyber1024Certificate,
    authorization_signing_serial: u128,
    authorization_encryption_serial: u128,
    certificate_service_binder_provider: Arc<Mutex<Box<CertificateServiceBinderProvider>>>,
}

impl TokioTcpWorker {
    async fn handle_connection(
        &self,
        mut socket: TcpStream,
        mut authorization_controller: AuthorizationController,
        remote_message_tx: Sender<Message>,
        mut local_message_rx: Receiver<Message>,
        peer_id: u128,
    ) {
        let min_connection_open_timestamp = get_timestamp_with_milliseconds() - HANDLE_START_GAP_MS;
        
        let auth_result = self.read_authorization_message(&mut socket, &mut authorization_controller, min_connection_open_timestamp).await;

        if auth_result.is_err() {
            log::error!("read_authorization_message: {}", auth_result.err().unwrap());
            return;
        }

        if let Err(e) = self.send_authorization_response(&mut socket, &mut authorization_controller).await {
            log::error!("send_authorization_response: {}", e);
            return;
        }

        self.setup_stream_transport(socket, remote_message_tx,
                                    local_message_rx, authorization_controller, auth_result.unwrap(), 
                                    peer_id).await;
    }

    async fn read_authorization_message(
        &self,
        socket: &mut TcpStream,
        authorization_controller: &mut AuthorizationController,
        min_timestamp: u128,
    ) -> Result<(Falcon1024Certificate, Kyber1024Certificate), &'static str> {
        let mut size_buffer = [0u8; std::mem::size_of::<usize>()];
        socket.read_exact(&mut size_buffer).await.map_err(|_| "Error while reading from socket")?;

        let mut bytes_to_read = usize::from_le_bytes(size_buffer);
        let mut message_buffer = [0u8; CHUNK_SIZE];
        let mut authorization_message = Serialized::new();

        while bytes_to_read > 0 {
            let size_to_receive = min(bytes_to_read, CHUNK_SIZE);
            let actual_bytes_count = socket.read(&mut message_buffer).await.map_err(|_| "Receive failed")?;

            if actual_bytes_count != size_to_receive {
                return Err("Expected bytes mismatch");
            }
            authorization_message.extend(message_buffer[0..actual_bytes_count].to_vec());
            bytes_to_read -= actual_bytes_count;
        }

        let (message, _) = AuthorizationMessage::from_serialized(&authorization_message).map_err(|_| "Cannot parse message")?;

        if message.timestamp < min_timestamp {
            return Err("Authorization message is too old!");
        }
        
        let certificates = authorization_controller.check_authorization_message(message);

        if certificates.is_none() {
            return Err("Authorization failed!");
        }

        Ok(certificates.unwrap())
    }

    async fn send_authorization_response(
        &self,
        socket: &mut TcpStream,
        authorization_controller: &mut AuthorizationController,
    ) -> Result<(), &'static str> {
        let my_message = authorization_controller
            .generate_authorization_message(
                self.authorization_signing_serial,
                self.authorization_encryption_serial,
                true,
            )
            .expect("Cannot generate authorization message")
            .serialize();

        let my_message_size = my_message.len();

        socket.write_all(&my_message_size.to_le_bytes()).await.map_err(|_| "Cannot send authorization message size")?;
        socket.write_all(&my_message).await.map_err(|_| "Cannot send authorization message data")?;

        authorization_controller.finalize();

        Ok(())
    }

    async fn setup_stream_transport(
        &self,
        mut socket: TcpStream,
        remote_message_tx: Sender<Message>,
        mut local_message_rx: Receiver<Message>,
        authorization_controller: AuthorizationController,
        cert_pair: (Falcon1024Certificate, Kyber1024Certificate),
        peer_id: u128,
    ) {
        let (signing_cert, encryption_cert) = cert_pair;

        let mut stream_wrapper = TokioStreamTransport::from_stream(socket);
        let crypto_transformer = CryptoTransformer::new(
            self.local_signing_cert.clone(),
            self.local_encryption_cert.clone(),
            signing_cert,
            encryption_cert,
        );
        stream_wrapper.add_transformer(Box::new(crypto_transformer));
        
        let result = stream_wrapper.send_raw(Message::new()
            .set_type(MessageType::SetPeerID)
            .set_destination(peer_id)
            .set_id(0)
            .set_current_timestamp()
            .set_source(0)
            .serialize()).await;
        
        if result.is_err(){
            log::error!("send_raw: Can not send peer id set message");
            return;
        }
        drop(result);

        loop {
            select! {
                Some(message) = stream_wrapper.receive_raw(None) => {
                    if let Ok((message, _)) = Message::from_serialized(&message) {
                        if remote_message_tx.send(message).await.is_err() {
                            log::error!("Cannot send received message");
                        }
                    } else {
                        log::error!("Unparsable message received");
                    }
                }
                Some(message) = local_message_rx.recv() => {
                    let serialized = message.serialize();
                    if stream_wrapper.send_raw(serialized).await.is_err() {
                        log::error!("Cannot communicate with remote");
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
    async fn run(&mut self, tx: Sender<Message>, rx: Receiver<Message>, peer_id_tx: Sender<u128>) {
        log::info!("Starting up a tokio async listener");
        let mut last_peer_id: u128 = 1<<128; //FUTURE: FLAG_REMOTE_IS_PEER
        let listener = TcpListener::bind(self.bind_address.as_str())
            .await
            .expect("Can not bind listener");
        loop {
            let (socket, _) = listener.accept().await
                .expect("Can not accept connection");
            let binder = self.certificate_service_binder_provider.lock().await.bind();
            let authorization_controller = AuthorizationController::new(binder);
            let (_peer_tx, mut peer_rx) = tokio::sync::mpsc::channel::<Message>(65536);
            //FIXME: CRTITICAL: DoS: limit number of coroutines spawned
            let worker = TokioTcpWorker{
                local_signing_cert: self.local_signing_cert.clone(),
                local_encryption_cert: self.local_encryption_cert.clone(),
                authorization_encryption_serial: self.authorization_encryption_serial,
                authorization_signing_serial: self.authorization_signing_serial,
                certificate_service_binder_provider: self.certificate_service_binder_provider.clone(),
            };
            let c_tx = tx.clone();
            tokio::spawn(async move {
                worker.handle_connection(socket, authorization_controller, c_tx, peer_rx, last_peer_id).await;
            });
            peer_id_tx.send(last_peer_id).await.expect("Can not notify about new peer");
            last_peer_id += 1;
        }
    }
}