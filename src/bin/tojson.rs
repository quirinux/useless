use std::{fs, io};
use std::io::Read;
use std::process::ExitCode;
use serde_yaml;
use serde_json;
use serde_json::Value;
use toml;

fn grab_input() -> String {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        let arg = args[1].clone();
        match fs::read_to_string(&arg) {
            Ok(o) => return o.clone(),
            Err(_) => { },
        }
    }

    let mut input = Vec::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_end(&mut input).expect("failed to readh from stdin");
    String::from_utf8(input).expect("invalid utf-8")
}

fn to_value(input: String) -> Option<Value> {
    let mut value: Option<Value>;
    // trying out YAML
    value = match serde_yaml::from_str(&input) {
        Ok(o) => Some(o),
        Err(_) => None,
    };
    if value.is_some() { return value; }

    // trying out TOML
    value = match toml::from_str(&input) {
        Ok(o) => Some(o),
        Err(_) => None,
    };
    if value.is_some() { return value; }

    value
}

fn main() -> ExitCode {
    let input = grab_input();
    let value: Option<Value> = to_value(input);


    if value.is_some() {
        match serde_json::to_string(&value.unwrap()) {
            Ok(o) => {
                print!("{}", o);
                return ExitCode::SUCCESS;
            },
            Err(e) => {
                eprint!("{}", e);
                return ExitCode::FAILURE;
            }
        }
    }

    println!("format not supported");
    return ExitCode::FAILURE;
}
