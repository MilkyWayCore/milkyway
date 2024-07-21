use std::sync::{Arc, Mutex};
use colored::Colorize;
use libmilkyway::cli::arguments::parse_arguments;
use libmilkyway::cli::router::CommandNamespace;
use libmilkyway::cli::table::Table;
use libmilkyway::pki::certificate::{Certificate, FLAG_CLIENT_CERT, FLAG_SIGN_CERTS};
use libmilkyway::pki::hash::HashType;
use libmilkyway::pki::impls::certificates::falcon1024::Falcon1024Certificate;
use libmilkyway::pki::impls::keys::falcon1024::generate_falcon1024_keypair;
use libmilkyway::serialization::serializable::Serializable;
use libmilkyway::services::certificate::{CertificateService, CertificateServiceBinder, ROOT_CERTIFICATE_SERIAL};
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

    fn generate_signed_certificate(&mut self, binder: &mut Box<CertificateServiceBinder>, serial_number: u128,
                                   parent_serial_number: u128, /* Serial number of certificate to sign with */
                                   name: String, flags: u128) -> Result<Falcon1024Certificate, &'static str>{
        if parent_serial_number==ROOT_CERTIFICATE_SERIAL{
            let root_certificate = binder.get_root_certificate();
            if root_certificate.is_none(){
                return Err("No root certificate");
            }
            let root_certificate = root_certificate.unwrap();
            let (public_key, secret_key) =generate_falcon1024_keypair();
            let mut certificate = Falcon1024Certificate{
                serial_number: serial_number,
                parent_serial_number: parent_serial_number,
                secret_key: Some(secret_key),
                public_key: public_key,
                signature: None,
                name: name,
                flags: flags,
            };
            let result = root_certificate.sign_data(&certificate.clone_without_signature_and_sk(),
                                                    HashType::None);
            if result.is_err(){
                return Err("Can not sign certificate");
            }
            certificate.signature = Some(result.unwrap());
            return Ok(certificate);
        } else {
            let parent_certificate = binder.get_signing_certificate(parent_serial_number);
            if parent_certificate.is_none(){
                return Err("Can not find parent certificate");
            }
            let parent_certificate = parent_certificate.unwrap();
            let can_sign = parent_certificate.check_flag(FLAG_SIGN_CERTS);
            if !can_sign{
                return Err("This certificate can not sign");
            }
            let (public_key, secret_key) =generate_falcon1024_keypair();
            let mut certificate = Falcon1024Certificate{
                serial_number: serial_number,
                parent_serial_number: parent_serial_number,
                secret_key: Some(secret_key),
                public_key: public_key,
                signature: None,
                name: name,
                flags: flags,
            };
            let result = parent_certificate.sign_data(&certificate.clone_without_signature_and_sk(), HashType::None);
            if result.is_err(){
                return Err("Can not sign a new certificate");
            }
            certificate.signature = Some(result.unwrap());
            return Ok(certificate);
        }
    }


    pub fn generate(&mut self, arguments: Vec<String>){
        // 1) Get certificate and sign in separate function
    }

    pub fn remove(&mut self, arguments: Vec<String>){
        let argmap = parse_arguments(arguments);
        if !argmap.contains_key("serial"){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'serial' is required");
            return;
        }
        let serial = argmap.get("serial").unwrap();
        if serial.is_none(){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'serial' must have a value");
            return;
        }
        let serial = serial.clone().unwrap();
        let serial = serial.parse::<u128>();
        if serial.is_err(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument serial must be a positive number");
            return;
        }
        let serial = serial.unwrap();
        let mut binder = self.cert_binder.lock().unwrap();
        let result = binder.remove_signing_certificate(serial);
        if !result {
            println!("{} {}", "error:".red().bold().underline(), "Can not remove certificate");
            return;
        }
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