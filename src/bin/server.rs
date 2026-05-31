use clap::Parser;
use std::{
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    #[arg(short, long, default_value = "7878")]
    port: u16,
}

fn handle_connection(mut stream: TcpStream) ->  Result<(), Box<dyn std::error::Error>> {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    println!("Request: {http_request:#?}");

    stream.write_all("HTTP/1.1 200 OK\r\n".as_bytes())?;
    stream.write_all("Content-Type: text/plain\r\n".as_bytes())?;
    stream.write_all("Length: 4\r\n\r\n".as_bytes())?;
    stream.write_all("PONG".as_bytes())?;

    Ok(())
}

fn main() {
    let args = Args::parse();
    let binding = "127.0.0.1:".to_string() + &args.port.to_string();
    println!("Listening on => {binding}");
    let listener = TcpListener::bind(binding).unwrap();


    for stream in listener.incoming() {
        let stream = stream.unwrap();

        match handle_connection(stream) {
            Err(e) => println!("{e}"),
            Ok(_) => {},
        }
    }
}
