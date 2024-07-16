///
/// Name service is responsible for handling known machine names
/// and certificates
/// 
pub trait NameService{
    ///
    /// Gets name of client by ID
    /// 
    /// # Arguments
    /// * id: u128: ID to lookup
    /// 
    /// returns: String: name of client
    /// 
    fn get_name_by_id(&self, id: u128) -> String;
    
    ///
    /// Gets domain of whole network
    /// 
    /// returns: String: domain name
    fn get_domain(&self) -> String;
}