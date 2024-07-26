use libmilkyway::tokio::init_tokio;
use libmilkyway::transport::async_stream::TokioStreamTransport;

mod configuration;
mod services;
mod listeners;

fn main() {
    init_tokio();
    env_logger::init();
    let listener = 
}
