pub trait ChannelTx{
    fn send(&mut self);
}

pub trait ChennelRx{
    fn recv(&mut self, timeout: Option<u64>)
}