pub mod loader;

use crate::message::common::Message;
use crate::services::certificate::CertificateServiceBinder;
use crate::services::name::NameService;
use crate::transport::TransportService;

///
/// A data bus for modules
/// Allows exchanging data between modules and MilkyWay on a local machine
///
pub trait ModuleDataBus: Send + Sync{
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

    ///
    /// Gets a certificate service
    ///
    /// returns: Box<dyn CertificateService>: a boxed trait object of a CertificateService
    ///
    fn get_certificate_service(&self) -> Box<CertificateServiceBinder>;
}

///
/// A dynamically loadable module
///
pub trait MilkywayModule: Send + Sync{
    ///
    /// Gets a unique ID of module
    ///
    fn get_id(&self) -> u64;
    
    ///
    /// Gets a supported CLI commands by a module
    /// 
    fn get_commands(&self) -> Vec<String>;

    ///
    /// Called when module is loaded
    ///
    /// # Arguments
    /// * data_bus: Arc<Box<dyn ModuleDataBus>>: an implementation of a data bus
    ///
    fn on_load(&mut self, data_bus: Box<dyn ModuleDataBus>);

    ///
    /// Called when some CLI command is received.
    ///
    /// # Arguments
    /// * command: String: a command received from CLI
    /// * arguments Vec<String>: arguments passed from CLI
    /// 
    /// # Command examples
    /// Level 2 command
    /// ```sh
    /// mway certman/list
    /// ```
    /// Level 3 command 
    /// ```sh
    /// mway certman/encryption/generate name="my_encryption_cert"
    ///```
    fn on_cli_command(&self, command: Vec<String>, arguments: Vec<String>);

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