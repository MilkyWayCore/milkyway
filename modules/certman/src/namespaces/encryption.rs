use std::sync::{Arc, Mutex};
use libmilkyway::cli::router::CommandNamespace;
use libmilkyway::cli::table::Table;
use libmilkyway::pki::certificate::Certificate;
use libmilkyway::services::certificate::{CertificateService, CertificateServiceBinder};
use crate::utils::{certificates_flags_to_string, optional_serial_to_string};
use colored::Colorize;

pub struct EncryptionNamespace{
    cert_binder: Arc<Mutex<Box<CertificateServiceBinder>>>,
}

impl EncryptionNamespace {
    pub fn new(binder: Arc<Mutex<Box<CertificateServiceBinder>>>) -> EncryptionNamespace{
        EncryptionNamespace{
            cert_binder: binder,
        }
    }
    pub fn generate(&mut self, args:Vec<String>){
        todo!()
    }
    pub fn remove(&mut self, args:Vec<String>){
        todo!()
    }
    pub fn export(&mut self, args:Vec<String>){
        todo!()
    }
    pub fn import(&mut self, args:Vec<String>){
        todo!()
    }
    pub fn show(&mut self){
        let result =self.cert_binder.lock().unwrap().get_encryption_certificates();
        let mut table = Table::new(vec!["SERIAL", "NAME", "FLAGS", "PARENT SERIAL"]);
        for certificate in result{
            table.add_row(vec![&certificate.get_serial().to_string(),
                               &certificate.get_name(), &certificates_flags_to_string(certificate.get_flags()),
                               &*optional_serial_to_string(certificate.get_parent_serial())]);
        }
        table.display();
    }
}
impl CommandNamespace for EncryptionNamespace{
    fn on_command(&mut self, command: String, args: Vec<String>) {
        match command.as_str() {
            "generate" => {
                self.generate(args);
            }
            "remove" => {
                self.remove(args);
            }
            "export" => {
                self.export(args);
            }
            "import" => {
                self.import(args);
            }
            "show" => {
                self.show();
            }
            &_ => {
                println!("{} {}", "error:".red().bold().underline(), "No such command");
            }
        }
    }
}