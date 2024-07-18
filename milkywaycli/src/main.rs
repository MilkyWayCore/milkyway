pub mod services;
mod bus;
mod configuration;
mod cli;

use std::fs;
use std::path::Path;
use std::process::exit;
use colored::Colorize;
use libmilkyway::module::loader::DynamicModule;
use libmilkyway::tokio::init_tokio;
use crate::bus::CLIDataBus;
use crate::cli::CLIController;
use crate::configuration::CLIConfiguration;


#[allow(unsafe_code)]
unsafe fn load_modules_from(dir_path: &Path) -> Vec<DynamicModule> {
    let mut result = Vec::<DynamicModule>::new();
    let paths = fs::read_dir(dir_path.iter());
    if paths.is_err(){
        println!("{}{}{}", "warning:".yellow().bold().underline(), " ".clear(),
                 "No modules directory found");
        return vec![];
    }
    for entry in paths.unwrap() {
        if entry.is_err(){
            continue;
        }
        let entry = entry.unwrap();
        let metadata = entry.metadata();
        if metadata.is_err(){
            continue;
        }
        let metadata = metadata.unwrap();
        if metadata.is_dir(){
            continue;
        }
        let fname = entry.path();
        let fname = fname.to_str().unwrap();
        let module = 
        unsafe {
            DynamicModule::load(fname)
        };
        if module.is_err() {
            println!("{}{}{} {}{}", "warning:".yellow().bold().underline(), " ".clear(),
                     "Failed to load module:".bold(), "".clear(),
                     fname);
            //println!("{:?}", module.err().unwrap());
            continue;
        }
        result.push(module.unwrap());
    }
    result
}


fn main() {
    // Initialize tokio
    init_tokio();

    // Read configuration
    let configuration = CLIConfiguration::load(Path::new("/tmp/mwayrc.yml"));
    if configuration.is_none(){
        println!("{}:{}", "error".red().bold().underline(), " can not read configuration".clear());
        exit(-1);
    }
    let configuration = configuration.unwrap();
    let storage_path_option = configuration.get_storage_path();
    if storage_path_option.is_none(){
        println!("{}:{}", "error".red().bold().underline(), " no storage_path in configuration".clear());
    }
    let storage_path = storage_path_option.unwrap();
    let binding = storage_path.join(Path::new("certs.dat"));
    let certificate_store_path = binding.as_path();
    let modules_path_option = configuration.get_modules_path();
    let modules_path = if modules_path_option.is_none(){
        Path::new("/opt/mway/lib/modules")
    } else {
        modules_path_option.unwrap()
    };

    // Load modules
    let mut modules: Vec<DynamicModule>;
    unsafe {
        modules = load_modules_from(modules_path);
    }

    // Create data bus
    // It will also start services
    let data_bus = CLIDataBus::new(certificate_store_path.to_str().unwrap());

    //Now tell all modules they are loaded
    for module in &mut modules{
        module.instance.on_load(Box::new(data_bus.clone()));
    }

    // Create a CLI controller
    let mut controller = CLIController::new(modules);

    // Check arguments
    let arguments: Vec<String> = std::env::args().collect();
    let arguments = arguments[1..].to_vec();
    if arguments.len() > 0{
        // Execute command provided
        let result = controller.handle_command(arguments[0].clone(), arguments[1..].to_vec().clone());
        if !result{
            exit(-1);
        }
        exit(0);
    }

    // No arguments were provided => start interactive shell
    controller.run();
}
