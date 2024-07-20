use std::sync::{Arc, Mutex};
use colored::Colorize;
use libmilkyway::cli::router::CommandNamespace;
use libmilkyway::services::certificate::CertificateServiceBinder;
use crate::namespaces::root::RootNamespace;

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

    }

    pub fn import(&mut self, arguments: Vec<String>){

    }

    pub fn sign_file(&mut self, argument: Vec<String>){

    }

    pub fn verify_file_signature(&mut self, argument: Vec<String>){

    }

    pub fn show(&mut self){

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