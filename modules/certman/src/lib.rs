use libmilkyway::message::common::Message;
use libmilkyway::module::{MilkywayModule, ModuleDataBus};
use libmilkyway::services::certificate::CertificateServiceBinder;

pub struct CertmanModule{
    certificate_service: Option<Box<CertificateServiceBinder>>
}

impl CertmanModule {
    pub fn new() -> CertmanModule{
        CertmanModule{
            certificate_service: None,
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
        self.certificate_service = Some(data_bus.get_certificate_service());
    }

    fn on_cli_command(&self, command: Vec<String>, arguments: Vec<String>) {
        println!("{:?}:", command);
        println!("{:?}:", arguments);
    }

    fn on_server_receive(&self, _packet: &Message) { /* stub */ }

    fn on_client_receive(&self, _packet: &Message) { /* stub */ }

    fn on_cli_receive(&self, _packet: &Message) { /* stub */ }
}

#[no_mangle]
pub extern "C" fn create() -> *mut dyn MilkywayModule{
    let object = CertmanModule::new();
    let boxed: Box<dyn MilkywayModule> = Box::new(object);
    Box::into_raw(boxed)
}
