use std::sync::Arc;
use crate::message::common::Message;
use crate::services::name::NameService;
use crate::transport::TransportService;

///
/// A data bus for modules
/// Allows exchanging data between modules and MilkyWay on a local machine
///
pub trait ModuleDataBus{
    ///
    /// Gets a transport service
    ///
    /// returns: Box<dyn TransportService>: a boxed trait object of a TransportService
    ///
    fn get_transport_service(&self) -> Box<dyn TransportService>;


    ///
    /// Gets a name service
    ///
    /// returns: Box<dyn NameService>: a boxed trait object of a NameService
    ///
    fn get_name_service(&self) -> Box<dyn NameService>;
}

///
/// A dynamically loadable module
///
pub trait MilkywayModule{
    ///
    /// Gets a unique ID of module
    ///
    fn get_id(&self) -> u64;

    ///
    /// Called when module is loaded
    ///
    /// # Arguments
    /// * data_bus: Arc<Box<dyn ModuleDataBus>>: an implementation of a data bus
    ///
    fn on_load(&mut self, data_bus: Arc<Box<dyn ModuleDataBus>>);

    ///
    /// Called when some CLI command is received.
    ///
    /// # Arguments
    /// * command: String: a command received from CLI
    /// * arguments Vec<String>: arguments passed from CLI
    ///
    fn on_cli_command(&self, command: String, arguments: Vec<String>);

    ///
    /// Handles message on milkyway server
    ///
    /// # Arguments
    /// * packet: &Message: a message received
    ///
    fn on_server_receive(&self, packet: &Message);

    ///
    /// Handles message on milkyway client
    ///
    /// # Arguments
    /// * packet: &Message: a message received
    ///
    fn on_client_receive(&self, packet: &Message);

    ///
    /// Handles messages received by CLI
    ///
    /// # Arguments
    /// * packet: &Message: a message received
    ///
    fn on_cli_receive(&self, packet: &Message);
}