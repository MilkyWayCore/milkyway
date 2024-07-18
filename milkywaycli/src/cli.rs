use std::io::{BufRead, stdin, stdout, Write};
use colored::Colorize;
use libmilkyway::module::CLIStatus;
use libmilkyway::module::loader::DynamicModule;

///
/// Stores state of CLI and handles commands
///
pub(crate) struct CLIController{
    known_commands: Vec<String>,
    modules: Vec<DynamicModule>,
    current_namespace: Vec<String>,
}

impl CLIController {
    ///
    /// Creates a CLIController with given modules
    ///
    /// # Arguments
    /// * modules: Vec<DynamicModule>: a vector of modules
    ///
    /// returns: CLIController: new CLI controller
    ///
    pub fn new(mut modules: Vec<DynamicModule>) -> Self{
        let mut known_commands = Vec::<String>::new();
        for module in &mut modules{
            known_commands.extend(module.instance.get_commands());
        }
        CLIController{
            known_commands,
            modules,
            current_namespace: Vec::<String>::new(),
        }
    }

    ///
    /// Handles exactly one command from CLI
    ///
    /// # Arguments
    /// * command_path: String: a path to command in format of "module/namespace/subnamespace/command"
    /// * arguments: Vec<String>: vector of arguments to command
    ///
    pub fn handle_command(&mut self, command_path: String, arguments: Vec<String>) -> bool{
        let namespaces: Vec<&str> = command_path.split("/").collect();
        if namespaces.len() == 0{
            return false;
        }
        let mut string_namespaces = self.current_namespace.clone();
        for s in &namespaces{
            string_namespaces.push(s.to_string());
        }
        //println!("{:?}", string_namespaces);
        let toplevel_command = string_namespaces[0].clone();
        if !self.known_commands.contains(&toplevel_command.to_string()){
             println!("{}: {}{}", "error".red().bold().underline(), "unknown command: ".clear(),
                      toplevel_command);
            return false;
        }
        for module in &mut self.modules{
            match module.instance.on_cli_command(string_namespaces.clone(), arguments.clone()){
                CLIStatus::NamespaceChange(path) => {
                    self.current_namespace = path;
                }
                CLIStatus::Done => {}
            }
        }
        true
    }

    ///
    /// Parses command to path and arguments
    ///
    /// # Arguments
    /// * cmdline: A commandline String
    ///
    /// returns: (String, Vec<String>): command and its arguments
    ///
    fn parse_command(cmdline: String) -> (String, Vec<String>){
        let command_line: Vec<&str> = cmdline.split(" ").collect();
        if command_line.len() == 0{
            return ("".to_string(), vec![]);
        }
        let command = command_line[0].to_string();
        let mut arguments = Vec::<String>::new();
        for i in 1..command_line.len(){
            arguments.push(command_line[i].to_string());
        }
        (command, arguments)
    }
    
    fn get_namespace_str(&self) -> String{
        if self.current_namespace.len() == 0{
            return "".to_string();
        }
        let mut result = " ".to_string();
        for part in self.current_namespace.iter(){
            result = result + "/" + part;
        }
        result
    }

    ///
    /// Runs a CLI
    ///
    pub fn run(&mut self){
        loop {
            print!("{}{}>{}", "mway".bold().underline(), self.get_namespace_str().blue(), " ".clear());
            stdout().flush().expect("Flushing failed");
            let cmdline = stdin().lock().lines().next().unwrap();
            let (command, arguments) = Self::parse_command(cmdline.unwrap());
            if command == "quit" || command == "exit"{
                break;
            }
            if command == ".." && self.current_namespace.len() > 0{
                self.current_namespace.pop();
                continue;
            }
            if command == "/"{
                self.current_namespace = vec![];
                continue;
            }
            self.handle_command(command, arguments);
        }
    }
}