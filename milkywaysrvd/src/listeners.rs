mod tokio_tcp;

use tokio::sync::mpsc::{Sender, Receiver};
use async_trait::async_trait;
use libmilkyway::message::common::Message;

#[async_trait]
pub(crate) trait TokioAsyncListener: Send + Sync{
    ///
    /// Starts a listener with given tx for sending received messages and
    /// rx to receive messages to send.
    ///
    /// # Arguments
    /// * tx: transmitter to put received messages
    /// * rx: receiver of messages to be sent
    ///
    async fn run(&mut self, tx: Sender<Message>, rx: Receiver<Message>);
}