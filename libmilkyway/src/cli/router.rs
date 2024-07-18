use std::collections::HashMap;

///
/// CommandNamespace is a trait which implements on namespace of commands
/// E.g. it implements everything in `certman/encryption`
///
pub trait CommandNamespace: Send + Sync{
    fn on_command(&mut self, command: String, args: Vec<String>);
}


///
/// CommandRouter allows quickly implementing namespaces by just adding path
/// and namespace.
///
pub struct CommandRouter{
    namespaces: HashMap<Vec<String>, Box<dyn CommandNamespace>>,
    subnamespaces: Vec<Vec<String>>,
}

impl CommandRouter {
    ///
    /// Creates empty command router
    /// 
    #[inline]
    pub fn new() -> CommandRouter{
        CommandRouter{
            namespaces: HashMap::new(),
            subnamespaces: vec![],
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
        for i in 1..namespace_path.len(){
            let subpath = namespace_path[0..i].to_vec();
            if !self.subnamespaces.contains(&subpath){
                self.subnamespaces.push(subpath);
            }
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
    /// 
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
    
    
    ///
    /// Checks that given path is known namespace
    /// 
    /// # Arguments
    /// * path: Vec<String>: Path to check
    /// 
    /// returns: true if path is a namespace, false otherwise
    /// 
    #[inline]
    pub fn is_namespace(&self, path: &Vec<String>) -> bool{
        self.namespaces.contains_key(path) || self.subnamespaces.contains(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    // Mock CommandNamespace implementation for testing
    struct MockNamespace {
        received_commands: Rc<RefCell<Vec<(String, Vec<String>)>>>,
    }

    impl MockNamespace {
        fn new() -> Self {
            Self {
                received_commands: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn get_received_commands(&self) -> Rc<RefCell<Vec<(String, Vec<String>)>>> {
            self.received_commands.clone()
        }
    }

    impl CommandNamespace for MockNamespace {
        fn on_command(&mut self, command: String, args: Vec<String>) {
            self.received_commands.borrow_mut().push((command, args));
        }
    }

    #[test]
    fn test_register_namespace() {
        let mut router = CommandRouter::new();
        let namespace_path = vec!["certman".to_string(), "encryption".to_string()];
        let namespace = Box::new(MockNamespace::new());

        router.register_namespace(namespace_path.clone(), namespace);
        assert!(router.namespaces.contains_key(&namespace_path));
    }

    #[test]
    #[should_panic(expected = "Namespace is already registered")]
    fn test_register_namespace_already_registered() {
        let mut router = CommandRouter::new();
        let namespace_path = vec!["certman".to_string(), "encryption".to_string()];
        let namespace = Box::new(MockNamespace::new());

        router.register_namespace(namespace_path.clone(), namespace);
        // Registering the same namespace again should panic
        router.register_namespace(namespace_path, Box::new(MockNamespace::new()));
    }

    #[test]
    fn test_on_command() {
        let mut router = CommandRouter::new();
        let namespace_path = vec!["certman".to_string(), "encryption".to_string()];
        let namespace = Box::new(MockNamespace::new());
        let received_commands = namespace.get_received_commands();

        router.register_namespace(namespace_path.clone(), namespace);

        let command_path = vec!["certman".to_string(), "encryption".to_string(), "add".to_string()];
        let arguments = vec!["arg1".to_string(), "arg2".to_string()];
        let result = router.on_command(command_path.clone(), arguments.clone());

        assert!(result);
        let received_commands = received_commands.borrow();
        assert_eq!(received_commands.len(), 1);
        assert_eq!(received_commands[0], ("add".to_string(), arguments));
    }

    #[test]
    fn test_on_command_namespace_not_found() {
        let mut router = CommandRouter::new();
        let namespace_path = vec!["certman".to_string(), "encryption".to_string()];
        let namespace = Box::new(MockNamespace::new());
        router.register_namespace(namespace_path.clone(), namespace);

        let command_path = vec!["certman".to_string(), "decryption".to_string(), "add".to_string()];
        let arguments = vec!["arg1".to_string(), "arg2".to_string()];
        let result = router.on_command(command_path, arguments);

        assert!(!result);
    }

    #[test]
    #[should_panic(expected = "Empty command vector")]
    fn test_on_command_empty_command() {
        let mut router = CommandRouter::new();
        let arguments = vec!["arg1".to_string(), "arg2".to_string()];
        router.on_command(Vec::new(), arguments);
    }
}

