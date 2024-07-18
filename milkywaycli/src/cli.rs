use std::io::{BufRead, stdin, stdout, Write};
use colored::Colorize;
use libmilkyway::module::loader::DynamicModule;

///
/// Stores state of CLI and handles commands
///
pub(crate) struct CLIController{
    known_commands: Vec<String>,
    modules: Vec<DynamicModule>,
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
        let mut string_namespaces = Vec::<String>::new();
        for s in &namespaces{
            string_namespaces.push(s.to_string());
        }
        let toplevel_command = namespaces[0];
        if !self.known_commands.contains(&toplevel_command.to_string()){
             println!("{}: {}{}", "error".red().bold().underline(), "unknown command: ".clear(),
                      toplevel_command);
            return false;
        }
        for module in &mut self.modules{
            module.instance.on_cli_command(string_namespaces.clone(), arguments.clone());
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

    ///
    /// Runs a CLI
    ///
    pub fn run(&mut self){
        loop {
            print!("{}{}", "mway>".bold().underline(), " ".clear());
            stdout().flush().expect("Flushing failed");
            let cmdline = stdin().lock().lines().next().unwrap();
            let (command, arguments) = Self::parse_command(cmdline.unwrap());
            if command == "quit" || command == "exit"{
                break;
            }
            self.handle_command(command, arguments);
        }
    }
}