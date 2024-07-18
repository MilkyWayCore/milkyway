use std::path::Path;
use std::sync::{Arc, Mutex};
use libmilkyway::actor::binder::coroutine::BinderAsyncService;
use libmilkyway::module::ModuleDataBus;
use libmilkyway::services::certificate::{CertificateAsyncService, CertificateServiceBinder};
use libmilkyway::services::name::NameService;
use libmilkyway::transport::TransportService;
use crate::services::certificate::CertificateServiceImpl;

///
/// A DataBus for CLI program
/// 
#[derive(Clone)]
pub struct CLIDataBus{
    certificate_service: Arc<Mutex<CertificateAsyncService>>,
}

impl CLIDataBus{
    pub fn new(certificate_storage: &str) -> CLIDataBus{
        let fpath = Path::new(certificate_storage);
        let service_impl = if fpath.exists(){
            CertificateServiceImpl::load_from_file(certificate_storage)
        } else {
            CertificateServiceImpl::new(certificate_storage)
        };
        let service = Box::new(service_impl);
        let service = BinderAsyncService::run(service);
        CLIDataBus{
            certificate_service: Arc::new(Mutex::new(service)),
        }
    }
}

impl ModuleDataBus for CLIDataBus{
    fn get_transport_service(&self) -> Box<dyn TransportService> {
        todo!()
    }

    fn get_name_service(&self) -> Box<dyn NameService> {
        todo!()
    }

    fn get_certificate_service(&self) -> Box<CertificateServiceBinder> {
        self.certificate_service.lock().unwrap().bind()
    }
}

