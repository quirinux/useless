use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process::ExitCode;
use core::time;
use lexopt::prelude::*;
use subst;
use reqwest;

#[derive(Debug, Default)]
struct Args {
    help: bool,
    verbose: usize,
    url: String,
    method: String,
    username: String,
    password: String,
    timeout: Option<time::Duration>,
    secure: bool,
    headers: reqwest::header::HeaderMap,
    body: HashMap<String, String>,
    client: reqwest::blocking::Client,
}

impl Args {
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.parse_args(std::env::args().collect())
    }

    fn parse_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // println!("{}", file_path);
        let mut file = File::open(file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        // println!("{}", contents);
        // println!("{:?}", subst::Env);
        let raw = subst::substitute(&contents.to_owned(), &subst::Env)?;
        // println!("{}", raw);

        let mut args = vec![];
        let mut token = String::new();
        let mut rune: char;
        let mut i: usize = 0;
        args.push("foobar".to_string()); // fake program name
        while i < raw.len() {
            rune = raw.chars().nth(i).ok_or("could not index raw char")?;
            // print!("{}", rune);

            if rune == ' ' || rune == '\n' || rune == '\r' {
                if token.len() > 0 {
                    args.push(token);
                    token = String::new();
                }
                i += 1;
                continue;
            } else if rune == '\\' {
                i += 2;
                continue;
            } else if rune == '"' {
                    // println!("{}", rune);
                    i += 1;
                    rune = raw.chars().nth(i).ok_or("could not index raw char")?;
                while rune != '"' && i < raw.len() {
                    token.push(rune);
                    i += 1;
                    rune = raw.chars().nth(i).ok_or("could not index raw char")?;
                    // println!("{}", rune);
                }
            } else if rune == '#' {
                if token.len() > 0 {
                    args.push(token);
                    token = String::new();
                }
                i += 1;
                rune = raw.chars().nth(i).ok_or("could not index raw char")?;
                while rune != '\n' && rune != '\r' && i < raw.len() {
                    // print!("{}.{}", i, rune);
                    i += 1;
                    rune = raw.chars().nth(i).ok_or("could not index raw char")?;
                }
                continue;

            } else {
                token.push(rune);
            }
            i += 1;
        }

        if token.len() > 0 {
            args.push(token);
        }

        // println!("{:?}", args);
        self.parse_args(args)
    }

    fn parse_args(&mut self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        let mut parser = lexopt::Parser::from_iter(&args);
        while let Some(arg) = parser.next()? {
            // println!("{:?}", arg);
            match arg {
                Short('h') | Long("help")  => self.help = true,
                Short('v') | Long("verbose")  => self.verbose += 1,
                Short('m') | Long("method")  => self.method = parser.value()?.parse()?,
                Short('u') | Long("username")  => self.username = parser.value()?.parse()?,
                Short('p') | Long("password")  => self.password = parser.value()?.parse()?,
                Short('t') | Long("timeout")  => self.timeout = Some(time::Duration::from_secs(parser.value()?.parse()?)),
                Short('s') | Long("secure")  => self.secure = true,
                Short(_) | Long(_) => continue,
                Value(val) => {
                    let v = val.to_str().ok_or("unreachable")?;
                    if File::open(v).is_ok() {
                        self.parse_from_file(v)?;
                    } else if v.starts_with(":443") {
                        self.url = "https://localhost".to_owned() + v;
                    } else if v.starts_with(":") {
                        self.url = "http://localhost".to_owned() + v;
                    } else if v.contains(":") {
                        let kv: Vec<&str> = v.split(":").collect();
                        let k = kv[0];
                        let v = kv[1..].join("");
                        self.headers.insert(
                            reqwest::header::HeaderName::from_bytes(k.as_bytes())?,
                            reqwest::header::HeaderValue::from_str(&v)?);
                    } else if v.contains("=") {
                        let kv: Vec<&str> = v.split("=").collect();
                        let k = kv[0];
                        let v = kv[1..].join("");
                        self.body.insert(k.to_string(), v.to_string());
                    }

                },
            }
        }

        Ok(())
    }

    fn prepare(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cb = reqwest::blocking::ClientBuilder::new();
        cb = cb.default_headers(self.headers.clone());

        if let Some(t) = self.timeout {
            cb = cb.timeout(t);
        }

        self.client = cb.build()?;
        Ok(())
    }

    fn run(&self) -> Result<reqwest::blocking::Response, Box<dyn std::error::Error>> {

        let mut method = reqwest::Method::from_bytes("GET".as_bytes())?;
        if self.method.len() > 0 {
            method = reqwest::Method::from_bytes(self.method.as_bytes())?;
        }

        let mut rb = self.client.request(method, self.url.clone());

        if self.username.len() > 0 || self.password.len() > 0 {
            rb = rb.basic_auth(self.username.clone(), Some(self.password.clone()));
        }

        if self.body.len() > 0 {
            rb = rb.header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .json(&self.body);
        }

        Ok(rb.send()?)

    }
}

fn main() -> ExitCode {
    let mut args: Args = Default::default();
    if let Err(e) = args.init() {
        eprintln!("{:?}", e);
        return ExitCode::FAILURE;
    }

    // println!("{:?}", args);

    if let Err(e) = args.prepare() {
        eprintln!("{:?}", e);
        return ExitCode::FAILURE;
    }

    match args.run() {
        Ok(r) => {
            if let Ok(o) = r.text() {
                println!("{}", o);
            }
        },
        Err(e) => {
            eprintln!("{:?}", e);
            return ExitCode::FAILURE;
        }
    }

    return ExitCode::SUCCESS;
}
