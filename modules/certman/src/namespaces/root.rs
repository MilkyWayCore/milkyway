use std::path::Path;
use std::sync::{Arc, Mutex};

use colored::Colorize;

use libmilkyway::cli::arguments::parse_arguments;
use libmilkyway::cli::io::confirm;
use libmilkyway::serialization::serializable::Serializable;
use libmilkyway::serialization::deserializable::Deserializable;
use libmilkyway::cli::router::CommandNamespace;
use libmilkyway::cli::table::Table;
use libmilkyway::pki::certificate::Certificate;
use libmilkyway::pki::impls::certificates::falcon1024::{Falcon1024RootCertificate, generate_falcon1024_root_certificate};
use libmilkyway::services::certificate::{CertificateService, CertificateServiceBinder};
use crate::utils::certificates_flags_to_string;

pub struct RootNamespace{
    cert_binder: Arc<Mutex<Box<CertificateServiceBinder>>>,
}

impl RootNamespace {
    pub fn new(binder: Arc<Mutex<Box<CertificateServiceBinder>>>) -> Self{
        RootNamespace{
            cert_binder: binder
        }
    }

    pub fn show(&mut self){
        let result = self.cert_binder.lock().unwrap().get_root_certificate();
        if result.is_none(){
            println!("No root certificate found");
        } else {
            let certificate = result.unwrap();
            let flags = certificate.get_flags();
            let mut table = Table::new(vec!["SERIAL", "NAME", "FLAGS"]);
            table.add_row(vec![&certificate.get_serial().to_string(),
                               &certificate.get_name(), &certificates_flags_to_string(flags)]);
            table.display();
        }
    }

    pub fn generate(&mut self, arguments: Vec<String>){
        let argmap = parse_arguments(arguments);
        if !argmap.contains_key("name"){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'name' is required");
            return;
        }
        let name = argmap.get("name").unwrap();
        if name.is_none(){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'name' requires a value");
            return;
        }
        let name = name.clone().unwrap().to_string();
        let certificate = generate_falcon1024_root_certificate(name);
        println!("Certificate generation successful");
        let mut binder = self.cert_binder.lock().unwrap();
        let old_certificate = binder.get_root_certificate();
        if old_certificate.is_some(){
            if !confirm("Root certificate is already generated"){
                return;
            }
        }
        binder.set_root_certificate(certificate);
        binder.commit();
        println!("Registered certificate in service");
    }
    
    pub fn export(&mut self, arguments: Vec<String>){
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
        let mut binder = self.cert_binder.lock().unwrap();
        let certificate = binder.get_root_certificate();
        if certificate.is_none(){
            println!("{} {}", "error:".red().bold().underline(), "No root certificate is available");
            return;
        }
        let certificate = certificate.unwrap();
        if Path::new(&file.clone().unwrap()).exists(){
            if !confirm("File already exists"){
                return;
            }
        }
        certificate.dump(&file.clone().unwrap());
        println!("Export successful");
    }
    
    pub fn import(&mut self, arguments: Vec<String>){
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
        let file = file.clone().unwrap();
        let certificate_result = Falcon1024RootCertificate::from_file(Path::new(&file));
        if certificate_result.is_err(){
            println!("{} {}", "error:".red().bold().underline(), "Can not read file. Does format is correct?");
            return;
        }
        println!("Loaded certificate successfully");
        let certificate = certificate_result.unwrap();
        let mut binder = self.cert_binder.lock().unwrap();
        let old_certificate = binder.get_root_certificate();
        if old_certificate.is_some(){
            if !confirm("Root certificate is already generated"){
                return;
            }
        }
        binder.set_root_certificate(certificate);
        binder.commit();
        println!("Registered certificate in service");
    }
}

impl CommandNamespace for RootNamespace{
    fn on_command(&mut self, command: String, args: Vec<String>) {
        match command.as_str() {
            "show" => {
                self.show();
            }
            "generate" => {
                self.generate(args);
            }
            "export" => {
                self.export(args);
            }
            "import" => {
                self.import(args)
            }
            &_ => {
                println!("{} {}", "error:".red().bold().underline(), "No such command");
            }
        }
    }
}