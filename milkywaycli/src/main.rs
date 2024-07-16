mod services;
mod bus;

use std::path::Path;
use colored::Colorize;
use libmilkyway::module::loader::DynamicModule;


#[allow(unsafe_code)]
unsafe fn load_modules_from(dir_path: &Path) -> Vec<DynamicModule> {
    let mut result = Vec::<DynamicModule>::new();
    for file_name in dir_path.iter() {
        let module = DynamicModule::load(file_name.to_str().unwrap());
        if module.is_err() {
            println!("{} {}{}", "Failed to load module:".bold(), "".clear(),
                     file_name.to_str().unwrap());
            continue;
        }
        result.push(module.unwrap());
    }
    result
}

fn main() {
    let mut known_commands = Vec::<String>::new();
    let modules_path = Path::new(".");
    unsafe {
        let mut modules = load_modules_from(modules_path);
    }
}
