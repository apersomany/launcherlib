pub mod context;
pub mod manifest;
pub mod version;

use std::collections::HashMap;

pub fn format(str: &str, vars: &HashMap<&str, String>) -> String {
    let split: Vec<&str> = str.split("${").collect();
    if split.len() == 2 {
        let first = split[0];
        let split: Vec<&str> = split[1].split("}").collect();
        let var = match vars.get(split[0]) {
            Some(var) => &var,
            None => "null"
        };
        vec![first, var, split[1]].join("")
    } else { str.to_string() }
}