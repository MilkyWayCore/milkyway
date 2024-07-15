use std::mem::size_of;
use std::time::Duration;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::serialization::deserializable::Deserializable;
use crate::serialization::serializable::{Serializable, Serialized};
use crate::transport::{Transport, TransportTransformer};

///
/// A transport over a tokio stream.
///
pub struct StreamTransport<T: AsyncReadExt + AsyncWriteExt + Send + Unpin>{
    stream: T,
    transformers: Vec<Box<dyn TransportTransformer>>,
}

impl<T: AsyncReadExt + AsyncWriteExt + Send + Unpin> StreamTransport<T> {
    pub fn from_stream(stream: T) -> StreamTransport<T>{
        StreamTransport{
            stream,
            transformers: vec![],
        }
    }
    
    fn apply_transform(&self, mut data: Serialized) -> Serialized{
        for transformer in &self.transformers{
            data = transformer.transform(&data);
        }
        data
    }
    
    fn apply_detransform(&self, mut data: Serialized) -> Serialized{
        for transformer in self.transformers.iter().rev(){
            data = transformer.detransform(&data).expect("REASON");
        }
        data
    }
}

#[async_trait]
impl<T: AsyncReadExt + AsyncWriteExt + Send + Unpin> Transport for StreamTransport<T> {
    #[inline]
    async fn send_raw(&mut self, data: Serialized) -> Result<usize, tokio::io::Error> {
        let data = self.apply_transform(data);
        let size = data.len();
        let mut data_with_size = size.serialize();
        data_with_size.extend(data.serialize());
        self.stream.write(&data_with_size).await
    }

    async fn receive_raw(&mut self, timeout: Option<u64>) -> Option<Serialized> {
        let mut data_size_buf: Serialized = Serialized::with_capacity(size_of::<usize>());
        data_size_buf.fill(0);
        let result = self.stream.read(&mut data_size_buf).await;
        if result.is_err(){
            return None;
        }
        let data_size = usize::from_serialized(&data_size_buf);
        if data_size.is_err(){
            return None;
        }
        let (data_size_unwrapped, _) = data_size.unwrap();
        let mut data_buf = Serialized::with_capacity(data_size_unwrapped);
        data_buf.fill(0);
        if timeout.is_some() {
            let result = tokio::time::timeout(Duration::from_millis(timeout.unwrap()),
                                              self.stream.read(&mut data_buf)).await;
            if result.is_err(){
                return None;
            }
        } else {
            let result = self.stream.read(&mut data_buf).await;
        }
        if result.is_err(){
            return None;
        }
        if result.unwrap() < data_size_unwrapped{
            return None;
        }
        Some(self.apply_detransform(data_buf))
    }

    #[inline]
    fn add_transformer<'a>(&'a mut self, transformer: Box<dyn TransportTransformer>) -> &'a Self {
        self.transformers.push(transformer);
        self
    }
}

/* Tests begin here */
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt, duplex};
    use tokio::time::{timeout, Duration};
    use std::sync::Arc;
    use std::sync::Mutex;
    use crate::serialization::serializable::{Serializable, Serialized};
    use crate::serialization::deserializable::Deserializable;
    use crate::serialization::error::SerializationError;
    use crate::transport::{Transport, TransportTransformer};
    use async_trait::async_trait;

    #[derive(Clone)]
    struct TestTransformer {
        pub pre_transform: Arc<Mutex<Serialized>>,
        pub post_transform: Arc<Mutex<Serialized>>,
    }

    impl TransportTransformer for TestTransformer {
        fn detransform(&self, data: &Serialized) -> Result<Serialized, SerializationError> {
            let mut pre_transform = self.pre_transform.lock().unwrap();
            *pre_transform = data.clone();
            Ok(data.clone())
        }

        fn transform(&self, data: &Serialized) -> Serialized {
            let mut post_transform = self.post_transform.lock().unwrap();
            *post_transform = data.clone();
            data.clone()
        }
    }

    #[tokio::test]
    async fn test_send_raw() {
        let (mut client, mut server) = duplex(64);
        let mut transport = StreamTransport::from_stream(client);

        let data: Serialized = vec![1, 2, 3, 4, 5];
        let size = transport.send_raw(data.clone()).await.unwrap();

        assert_eq!(size, 5 + size_of::<usize>());

        let mut data_size_buf = vec![0u8; size_of::<usize>()];
        server.read_exact(&mut data_size_buf).await.unwrap();

        let (data_size, _) = usize::from_serialized(&data_size_buf).unwrap();
        assert_eq!(data_size, 5);

        let mut data_buf = vec![0u8; data_size];
        server.read_exact(&mut data_buf).await.unwrap();

        assert_eq!(data_buf, data);
    }

    #[tokio::test]
    async fn test_receive_raw() {
        let (mut client, mut server) = duplex(64);
        let mut transport = StreamTransport::from_stream(client);

        let data: Serialized = vec![1, 2, 3, 4, 5];
        let data_size = data.len();
        let mut data_with_size = data_size.serialize();
        data_with_size.extend(data.clone());

        server.write_all(&data_with_size).await.unwrap();

        let received_data = transport.receive_raw(None).await.unwrap();
        assert_eq!(received_data, data);
    }

    #[tokio::test]
    async fn test_transformers() {
        let (mut client, mut server) = duplex(64);
        let mut transport = StreamTransport::from_stream(client);

        let pre_transform = Arc::new(Mutex::new(Vec::new()));
        let post_transform = Arc::new(Mutex::new(Vec::new()));

        let transformer = TestTransformer {
            pre_transform: pre_transform.clone(),
            post_transform: post_transform.clone(),
        };

        transport.add_transformer(Box::new(transformer));

        let data: Serialized = vec![1, 2, 3, 4, 5];
        transport.send_raw(data.clone()).await.unwrap();

        let mut data_size_buf = vec![0u8; size_of::<usize>()];
        server.read_exact(&mut data_size_buf).await.unwrap();

        let (data_size, _) = usize::from_serialized(&data_size_buf).unwrap();
        assert_eq!(data_size, 5);

        let mut data_buf = vec![0u8; data_size];
        server.read_exact(&mut data_buf).await.unwrap();

        assert_eq!(data_buf, data);

        let pre_transform_data = pre_transform.lock().unwrap().clone();
        let post_transform_data = post_transform.lock().unwrap().clone();

        assert_eq!(pre_transform_data, data);
        assert_eq!(post_transform_data, data);
    }

    #[tokio::test]
    async fn test_receive_raw_with_timeout() {
        let (mut client, mut server) = duplex(64);
        let mut transport = StreamTransport::from_stream(client);

        let result = timeout(Duration::from_millis(100), transport.receive_raw(Some(100))).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_and_receive() {
        let (mut client, mut server) = duplex(64);
        let mut transport = StreamTransport::from_stream(client);

        let data: Serialized = vec![1, 2, 3, 4, 5];
        transport.send_raw(data.clone()).await.unwrap();

        let received_data = transport.receive_raw(None).await.unwrap();
        assert_eq!(received_data, data);
    }
}

