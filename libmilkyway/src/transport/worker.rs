use crate::transport::handler::TransportHandlerWorkerBinder;

///
/// A transport worker manages one or few connections with peers
/// 
pub trait TransportWorker: Send + Sync{
    ///
    /// A function to be called when worker is binded to handler
    /// 
    /// # Arguments
    /// * binder: A binder to handler
    /// 
    fn on_bind_to_handler(&mut self, binder: Box<TransportHandlerWorkerBinder>);
}