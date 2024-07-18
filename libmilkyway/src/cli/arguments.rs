use std::collections::HashMap;

///
/// Parses arguments to a HashMap
///
pub fn parse_arguments(args: Vec<String>) -> HashMap<String, Option<String>>{
    let mut argmap = HashMap::<String, Option<String>>::new();
    for entry in args{
        let result: Vec<&str> = entry.split("=").collect();
        if result.len() == 0{
            continue;
        }
        if result.len() == 1{
            argmap.insert(result[0].to_string(), None);
        }
        if result.len() == 2{
            argmap.insert(result[0].to_string(), Some(result[1].to_string()));
        }
    }
    argmap
}