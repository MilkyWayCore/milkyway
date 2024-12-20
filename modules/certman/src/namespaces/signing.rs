use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use colored::Colorize;
use libmilkyway::cli::arguments::parse_arguments;
use libmilkyway::cli::router::CommandNamespace;
use libmilkyway::cli::table::Table;
use libmilkyway::pki::certificate::{Certificate, FLAG_CLIENT_CERT, FLAG_NO_READ, FLAG_NO_WRITE, FLAG_SERVER_CERT, FLAG_SIGN_CERTS, FLAG_SIGN_MESSAGES, FLAG_USER_CERT};
use libmilkyway::pki::hash::HashType;
use libmilkyway::pki::impls::certificates::falcon1024::Falcon1024Certificate;
use libmilkyway::pki::impls::keys::falcon1024::generate_falcon1024_keypair;
use libmilkyway::serialization::deserializable::Deserializable;
use libmilkyway::serialization::serializable::Serializable;
use libmilkyway::services::certificate::{CertificateService, CertificateServiceBinder, ROOT_CERTIFICATE_SERIAL};
use crate::utils::{certificates_flags_to_string, optional_serial_to_string};


const SIGNING_CHUNK_SIZE: usize = 65536;

pub struct SigningNamespace{
    cert_binder: Arc<Mutex<Box<CertificateServiceBinder>>>,
}

impl SigningNamespace {
    pub fn new(binder: Arc<Mutex<Box<CertificateServiceBinder>>>) -> Self{
        SigningNamespace{
            cert_binder: binder
        }
    }

    fn generate_signed_certificate(&self, binder: &mut Box<CertificateServiceBinder>, serial_number: u128,
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
    fn parse_flags(value: String) -> Option<u128> {
        let flags = value.split(",");
        let mut result = 0;
        for flag in flags{
            if flag == "no-read"{
                result = result | FLAG_NO_READ;
                continue;
            }
            if flag == "no-write" {
                result = result | FLAG_NO_WRITE;
                continue;
            }
            if flag == "sign-messages" {
                result = result | FLAG_SIGN_MESSAGES;
                continue;
            }
            if flag == "sign-certs" {
                result = result | FLAG_SIGN_CERTS;
                continue;
            }
            if flag == "client-cert" {
                result = result | FLAG_CLIENT_CERT;
                continue;
            }
            if flag == "server-cert" {
                result = result | FLAG_SERVER_CERT;
                continue;
            }
            if flag == "user-cert" {
                result = result | FLAG_USER_CERT;
                continue;
            }
            return None;
        }
        return Some(result);
    }


    // Arguments of comma.nd(those ones in argmap)
    // * serial -- a serial number for new certificate
    // * parent -- a serial number of parent certificate
    // * name -- a name of certificate
    // * flags -- flags list, optional(use parse_flags), if not provided default 0
    pub fn generate(&mut self, arguments: Vec<String>){
        let argmap = parse_arguments(arguments);
        /* Check serial */
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
        if !argmap.contains_key("parent"){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'parent' is required");
            return;
        }
        let parent = argmap.get("parent").unwrap();
        if parent.is_none(){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'parent' must have a value");
            return;
        }
        let parent = parent.clone().unwrap();
        let parent = parent.parse::<u128>();
        if parent.is_err(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'parent' must be a positive number");
            return;
        }
        let parent = parent.unwrap();
        if !argmap.contains_key("name"){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'name' is required");
            return;
        }
        let name = argmap.get("name").unwrap();
        if name.is_none(){
            println!("{} {}", "error:".red().bold().underline(), "Argument 'name' requires a value");
            return;
        }
        let name = name.clone().unwrap();
        let mut flags = 0;
        if argmap.contains_key("flags"){
            let flags_argument =  argmap.get("flags").unwrap();
            if flags_argument.is_none(){
                println!("{} {}", "error:".red().bold().underline(), "Argument 'flags' requires a value");
                return;
            }
            let flags_result = Self::parse_flags(flags_argument.clone().unwrap());
            if flags_result.is_none(){
                println!("{} {}", "error:".red().bold().underline(), "Argument 'flags' is invalid");
                return;
            }
            flags = flags_result.unwrap();
        }
        let mut binder = self.cert_binder.lock().unwrap();
        let signed_certificate = self.generate_signed_certificate(&mut binder,
                                                                  serial, parent, name, flags);
        if signed_certificate.is_err(){
            println!("{} {}", "error:".red().bold().underline(),signed_certificate.err().unwrap());
            return;
        }
        let signed_certificate = signed_certificate.unwrap();
        let result = binder.add_signing_certificate(signed_certificate);
        if !result{
            println!("{} {}", "error:".red().bold().underline(), "Can not add certificate to servise");
            return;
        }
        binder.commit();
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
        let argmap = parse_arguments(arguments);
        if !argmap.contains_key("file"){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'file' is required");
            return;
        }
        //None
        //Some(_)
        let argument = argmap.get("file").unwrap();
        if argument.is_none(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'file' requires a value");
            return;
        }
        let file_name = argument.clone().unwrap();
        let certificate = Falcon1024Certificate::from_file(Path::new(&file_name));
        if certificate.is_err(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not read a certificate");
            return;
        }
        let certificate = certificate.unwrap();
        let mut binder = self.cert_binder.lock().unwrap();
        let result = binder.add_signing_certificate(certificate);
        if !result{
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not add certificate to service");
            return;
        }
    }

    // sign-file file=/tmp/satanic_kitten_orgy
    pub fn sign_file(&mut self, arguments: Vec<String>) {
        let argmap = parse_arguments(arguments);
        if !argmap.contains_key("signature-file") {
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'signature-file' is required");
            return;
        }
        let signature_file = argmap.get("signature-file").unwrap();
        if signature_file.is_none() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'signature-file' requires a value");
            return;
        }
        // use std::fs::File;
        // use std::io::Write;
        //
        // fn main() -> std::io::Result<()> {
        //     // Create a file named "example.txt"
        //     let mut file = File::create("example.txt")?;
        //
        //     // Write some data to the file
        //     file.write_all(b"Hello, world!")?;
        //
        //     Ok(())
        // }
        let mut signature_file = File::create(signature_file);
        if signature_file.is_err() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not create signature file");
            return;
        }
        let mut signature_file = signature_file.unwrap();
        if !argmap.contains_key("file") {
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'file' is required");
            return;
        }
        let file = argmap.get("file").unwrap();
        if file.is_none() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'file' requires a value");
            return;
        }
        let file = File::open(file.clone().unwrap());
        if file.is_err() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not open file");
            return;
        }
        let argument = argmap.get("serial");
        if argument.is_none() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'serial' is required");
            return;
        }
        let argument = argument.unwrap();
        if argument.is_none() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument requires a value");
            return;
        }
        let argument = argument.parse::<u128>();
        if argument.is_err() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'serial' must be a positive integer");
            return;
        }
        let argument = argument.unwrap();
        let mut binder = self.cert_binder.lock().unwrap();
        let certificate = binder.get_signing_certificate(argument);
        if certificate.is_none() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not find certificate");
            return;
        }
        let certificate = certificate.unwrap();
        //Result<Type, ErrorType>
        // * Some(value): Result<Type, ErrorType>
        // * Err(error_value): Result<Type, ErrorType>
        // .unwrap() ->
        // * value: Type
        // * PANIC
        //Option<Type>
        // * Some(value): Option<Type>
        // * None: Option<Type>
        //.unwrap() ->
        // * value: Type
        // * PANIC
        let file = file.unwrap();
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0u8; SIGNING_CHUNK_SIZE];
        loop {
            let bytes_read = reader.read(&mut buffer).unwrap();
            if bytes_read == 0 {
                break;
            }
            let data = &buffer[..bytes_read];
            let signature = certificate.sign_data(data, HashType::None);
            if signature.is_err() {
                println!("{} {}", "error:".red().bold().underline(),
                         "Can not sign chunk");
                return;
            }
            let signature = signature.unwrap();
            let serialized_signature = signature.serialize();
            let serialized_signature_size = serialized_signature.len();
            if signature_file.write_all(serialized_signature_size.serialize()).is_err() {
                if signature_file.write_all(signature.serialize()).is_err() {
                    println!("{} {}", "error:".red().bold().underline(),
                             "Can not write signature file");
                    return;
                }
                if signature_file.write_all(signature.serialize()).is_err() {
                    println!("{} {}", "error:".red().bold().underline(),
                             "Can not write signature file");
                    return;
                }
            }

            //chunks
            //reading chunk by chunk
            //use std::fs::File;
            // use std::io::{self, Read, BufReader};
            //
            // fn main() -> io::Result<()> {
            //     // Open the file in read-only mode
            //     let file = File::open("path/to/your/file.txt").unwrap();
            //     let mut reader = BufReader::new(file);
            //
            //     // Define the size of each chunk
            //     let chunk_size = 1024;
            //     let mut buffer = vec![0; chunk_size];
            //
            //     loop {
            //         // Read a chunk of the file
            //         let bytes_read = reader.read(&mut buffer).unwrap();
            //
            //         // If no more bytes are read, we've reached the end of the file
            //         if bytes_read == 0 {
            //             break;
            //         }
            //
            //         // Process the chunk (here we simply print it as a string)
            //         let data = &buffer[..bytes_read];
            //     }
            //
            //     Ok(())
            // }

            /*
        File: Kuzya, Watson, Murczyk, Pusheen, Fintus
        Buffer: [_, _]
        read(File) -> 2
        Buffer: [Kuzya, Watson] -> szpital
        read(File) -> 2
        Buffer [Murczyk, Pusheen] -> szpital
        read(File) -> 1
        [Fintus, _] -> szpital
        read(File) -> 0
         */
        }
    }

    pub fn verify_file_signature(&mut self, argument: Vec<String>){
        let argmap = parse_arguments(argument);
        if !argmap.contains_key("file"){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'file' is required");
            return;
        }
        let file_name = argmap.get("file").unwrap();
        if file_name.is_none(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'file' requires a value");
            return;
        }
        let file_name = file_name.clone().unwrap();
        if !argmap.contains_key("signature-file"){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'signature-file' is required");
            return;
        }
        let signature_file = argmap.get("signature-file").unwrap();
        if signature_file.is_none(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Argument 'signature-file' requires a value");
            return;
        }
        let mut signature_file = File::open(signature_file.clone().unwrap());
        let mut file = File::open(file_name);
        if signature_file.is_err(){
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not open signature-file");
            return;
        }
        let signature_file = signature_file.unwrap();
        if file.is_err() {
            println!("{} {}", "error:".red().bold().underline(),
                     "Can not open file");
            return;
        }
        let file = file.unwrap();
        
    }


    pub fn show(&mut self){
        let result =self.cert_binder.lock().unwrap().get_signing_certificates();
        let mut table = Table::new(vec!["SERIAL", "NAME", "FLAGS", "PARENT SERIAL"]);
        for certificate in result{
            table.add_row(vec![&certificate.get_serial().to_string(),
                               &certificate.get_name(), &certificates_flags_to_string(certificate.get_flags()),
                               &*optional_serial_to_string(certificate.get_parent_serial())]);
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