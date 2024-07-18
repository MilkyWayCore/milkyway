mod namespaces;
mod utils;

use std::sync::{Arc, Mutex};
use colored::Colorize;
use libmilkyway::cli::router::{CommandNamespace, CommandRouter};
use libmilkyway::message::common::Message;
use libmilkyway::module::{CLIStatus, MilkywayModule, ModuleDataBus};
use libmilkyway::module::CLIStatus::{Done, NamespaceChange};
use libmilkyway::services::certificate::CertificateServiceBinder;
use crate::namespaces::root::RootNamespace;

///
/// The module for managing certificates
/// 
pub struct CertmanModule{
    certificate_service: Option<Arc<Mutex<Box<CertificateServiceBinder>>>>,
    router: CommandRouter,
}

impl CertmanModule {
    pub fn new() -> CertmanModule{
        CertmanModule{
            certificate_service: None,
            router: CommandRouter::new(),
        }
    }
}

impl MilkywayModule for CertmanModule {
    fn get_id(&self) -> u64 {
        1
    }

    fn get_commands(&self) -> Vec<String> {
        vec!["certman".to_string()]
    }

    fn on_load(&mut self, data_bus: Box<dyn ModuleDataBus>) {
        let mut binder = Arc::new(Mutex::new(data_bus.get_certificate_service()));
        self.certificate_service = Some(binder.clone());
        self.router.register_namespace(vec!["certman".to_string(), "root".to_string()], 
                                       Box::new(RootNamespace::new(binder.clone())))
    }

    fn on_cli_command(&mut self, command: Vec<String>, arguments: Vec<String>) -> CLIStatus {
        if self.router.is_namespace(&command){
            return NamespaceChange(command);
        }
        if !self.router.on_command(command, arguments){
            println!("{} {}", "error:".red().bold().underline(), "No such command");
        }
        Done
    }

    fn on_server_receive(&self, _packet: &Message) { /* stub */ }

    fn on_client_receive(&self, _packet: &Message) { /* stub */ }

    fn on_cli_receive(&self, _packet: &Message) { /* stub */ }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create() -> *mut dyn MilkywayModule{
    let object = CertmanModule::new();
    let boxed: Box<dyn MilkywayModule> = Box::new(object);
    Box::into_raw(boxed)
}
