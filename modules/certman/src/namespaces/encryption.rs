use std::sync::{Arc, Mutex};
use libmilkyway::services::certificate::CertificateServiceBinder;

pub struct EncryptionNamespace{
    cert_binder: Arc<Mutex<Box<CertificateServiceBinder>>>,
}

impl EncryptionNamespace {
    pub fn new(binder: Arc<Mutex<Box<CertificateServiceBinder>>>) -> EncryptionNamespace{
        EncryptionNamespace{
            cert_binder: binder,
        }
    }
}