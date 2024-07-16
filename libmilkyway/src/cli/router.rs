use std::collections::HashMap;

///
/// CommandNamespace is a trait which implements on namespace of commands
/// E.g. it implements everything in `certman/encryption`
///
pub trait CommandNamespace{
    fn on_command(&mut self, command: String, args: Vec<String>);
}


///
/// CommandRouter allows quickly implementing namespaces by just adding path
/// and namespace.
///
pub struct CommandRouter{
    namespaces: HashMap<Vec<String>, Box<dyn CommandNamespace>>,
}

impl CommandRouter {
    ///
    /// Creates empty command router
    /// 
    #[inline]
    pub fn new() -> CommandRouter{
        CommandRouter{
            namespaces: HashMap::new(),
        }
    }
    
    ///
    /// Adds new namespace to router
    /// 
    /// # Arguments
    /// * namespace_path: Vec<String>: path to a namespace
    /// * namespace: Box<dyn CommandNamespace>: A boxed trait object with handler 
    ///                                         of particular namespace
    /// 
    /// # Panics
    /// * If the namespace is already registered
    /// 
    #[inline]
    pub fn register_namespace(&mut self, namespace_path: Vec<String>, 
                              namespace: Box<dyn CommandNamespace>){
        if self.namespaces.contains_key(&namespace_path){
            panic!("Namespace is already registered");
        }
        self.namespaces.insert(namespace_path, namespace);
    }
    
    ///
    /// Handles command
    /// 
    /// # Arguments
    /// * command: Vec<String>: a full path to command(including command itself)
    /// * arguments: Vec<String>: all arguments to command
    /// 
    /// # Panics
    /// * If command vector is empty
    /// 
    /// # Returns
    /// true if command was found, false otherwise
    pub fn on_command(&mut self, command: Vec<String>, arguments: Vec<String>) -> bool{
        if command.len() == 0{
            panic!("Empty command vector");
        } 
        let command_name = command.last().unwrap();
        let namespace = command[0..command.len()-1].to_vec();
        if !self.namespaces.contains_key(&namespace){
            return false;
        }
        self.namespaces.get_mut(&namespace).unwrap().on_command(command_name.clone(), arguments);
        true
    }
}