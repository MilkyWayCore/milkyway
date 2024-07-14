use std::mem::size_of;
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
            data = transformer.detransform(&data);
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
        if timeout.is_some(){
            panic!("set timeout: operation not supported");
        }
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
        let result = self.stream.read(&mut data_buf).await;
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