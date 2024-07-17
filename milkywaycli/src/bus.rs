use libmilkyway::module::ModuleDataBus;
use libmilkyway::services::certificate::CertificateService;
use libmilkyway::services::name::NameService;
use libmilkyway::transport::TransportService;

pub struct CLIDataBus{
    certificate_service: Box<dyn CertificateService>
}

impl CLIDataBus{
    pub fn new(certificate_service: Box<dyn CertificateService>) -> CLIDataBus{
        CLIDataBus{
            certificate_service,
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

    fn get_certificate_service(&self) -> Box<dyn CertificateService> {
        todo!()
    }
}

