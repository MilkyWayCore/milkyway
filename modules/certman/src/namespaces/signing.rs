use std::sync::{Arc, Mutex};
use colored::Colorize;
use libmilkyway::cli::arguments::parse_arguments;
use libmilkyway::cli::router::CommandNamespace;
use libmilkyway::cli::table::Table;
use libmilkyway::pki::certificate::Certificate;
use libmilkyway::serialization::serializable::Serializable;
use libmilkyway::services::certificate::{CertificateService, CertificateServiceBinder};
use crate::namespaces::root::RootNamespace;
use crate::utils::certificates_flags_to_string;

pub struct SigningNamespace{
    cert_binder: Arc<Mutex<Box<CertificateServiceBinder>>>,
}

impl SigningNamespace {
    pub fn new(binder: Arc<Mutex<Box<CertificateServiceBinder>>>) -> Self{
        SigningNamespace{
            cert_binder: binder
        }
    }

    pub fn generate(&mut self, arguments: Vec<String>){

    }

    pub fn remove(&mut self, arguments: Vec<String>){

    }

    pub fn export(&mut self, arguments: Vec<String>){
        println!("{:?}", arguments);
        println!("{:?}", parse_arguments(arguments.clone()));
        let argmap = parse_arguments(arguments);
        if !argmap.contains_key("file"){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'file' is required");
            return;
        }
        let file = argmap.get("file").unwrap();
        if file.is_none(){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'file' requires a value");
            return;
        }
        if !argmap.contains_key("serial") {
            println!("{} {}", "error:".red().bold().underline(), "Argument 'serial' is required");
            return;
        }
        let serial = argmap.get("serial").unwrap();
        if serial.is_none(){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'serial' requires a value");
            return;
        }
        let mut binder = self.cert_binder.lock().unwrap();
        let serial = serial.clone().unwrap().parse::<u128>();
        if serial.is_err(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'serial' must be a positive integer");
            return;
        }
        let serial = serial.unwrap();
        if serial==0{
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not export root certificate");
            return;
        }
        let certificate = binder.get_signing_certificate(serial);
        if certificate.is_none(){
            println!("{} {}", "error:".red().bold().underline(),
                     "No certificate with such serial number");
            return;
        }
        let certificate = certificate.unwrap();
        certificate.dump(&file.clone().unwrap());
    }

    pub fn import(&mut self, arguments: Vec<String>){

    }

    pub fn sign_file(&mut self, argument: Vec<String>){

    }

    pub fn verify_file_signature(&mut self, argument: Vec<String>){

    }

    pub fn show(&mut self){
        let result =self.cert_binder.lock().unwrap().get_signing_certificates();
        let mut table = Table::new(vec!["SERIAL", "NAME", "FLAGS"]);
        for certificate in result{
            table.add_row(vec![&certificate.get_serial().to_string(),
            &certificate.get_name(), &certificates_flags_to_string(certificate.get_flags())]);
        }
        table.display();
    }
}

impl CommandNamespace for SigningNamespace {
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
            "sign-file" => {
                self.sign_file(args);
            }
            "verify-file-signature" => {
                self.verify_file_signature(args);
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