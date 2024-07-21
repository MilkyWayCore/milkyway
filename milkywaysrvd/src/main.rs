use libmilkyway::tokio::init_tokio;

mod configuration;
mod services;

fn main() {
    init_tokio();
    env_logger::init();
}
