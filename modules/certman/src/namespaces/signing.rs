use std::sync::{Arc, Mutex};
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