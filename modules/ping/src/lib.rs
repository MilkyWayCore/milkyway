mod responder;

use colored::Colorize;
use libmilkyway::message::common::Message;
use libmilkyway::module::{CLIStatus, HostType, MilkywayModule, ModuleDataBus};
use libmilkyway::services::transport::MessageFilter;
use crate::responder::PingResponder;

///
/// The module for pinging peers
///
pub struct PingModule {
    filter_id: Option<u128>,
}

impl PingModule {
    pub fn new() -> PingModule {
        PingModule {
            filter_id: None,
        }
    }
}

impl MilkywayModule for PingModule {
    fn get_id(&self) -> u64 {
        2
    }

    fn get_commands(&self) -> Vec<String> {
        vec!["ping".to_string()]
    }

    fn on_load(&mut self, data_bus: Box<dyn ModuleDataBus>) {
        let mut service = data_bus.get_transport_service();
        let my_id = data_bus.get_host_id();
        if my_id.is_none(){
            log::error!("Can not properly load ping module: not in a network");
            return;
        }
        let my_id = my_id.unwrap();
        let transport = service.get_transport();
        let responder = Box::new(PingResponder::new(my_id, self.get_id(), 
                                                    transport));
        self.filter_id = Some(service.subscribe_to_messages(MessageFilter::new()
                                                                .filter_module(self.get_id()), 
                                                            responder));
    }

    fn on_cli_command(&mut self, command: Vec<String>, arguments: Vec<String>) -> CLIStatus {
        todo!();
    }

    fn on_server_receive(&self, _packet: &Message) { /* stub */ }

    fn on_client_receive(&self, _packet: &Message) { /* stub */ }

    fn on_cli_receive(&self, _packet: &Message) { /* stub */ }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn create() -> *mut dyn MilkywayModule{
    let object = PingModule::new();
    let boxed: Box<dyn MilkywayModule> = Box::new(object);
    Box::into_raw(boxed)
}
