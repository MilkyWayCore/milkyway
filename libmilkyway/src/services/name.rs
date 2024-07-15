///
/// Name service is responsible for handling known machine names
/// and certificates
/// 
pub trait NameService{
    fn get_name_by_id(&self, id: u128) -> String;
    fn add_name(&mut self) -> bool;
}