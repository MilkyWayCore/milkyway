use std::io::{BufRead, stdin, stdout, Write};
use colored::Colorize;

///
/// Asks user for confirmation
///
/// # Arguments
/// * prompt: &str: Prompt to show user
///
/// returns: bool: true if user entered "y" or "Y", false otherwise
///
pub fn confirm(prompt: &str) -> bool{
    loop {
        print!("{}{}", prompt.bold(), ". Proceed?[y/N]: ");
        stdout().lock().flush().expect("Can not flush");
        let result = stdin().lock().lines().next().expect("Can not read line").unwrap();
        if result.len() != 0{
            let c= result.chars().nth(0).unwrap();
            return c == 'y' || c == 'Y';
        }
    }
}