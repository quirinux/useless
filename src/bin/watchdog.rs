use std::process::{ExitCode, Command};
use std::{path::Path, sync::mpsc};

use clap::Parser;
use notify::{Event, RecursiveMode, Result, Watcher};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    // Path to watch
    path: Box<Path>,

    // Command to run
    #[arg(last = true)]
    command: Vec<String>,

}

fn main_loop(mut args: Args) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;

    let mut child = Command::new(args.command[0].clone())
        .args(&mut args.command[1..])
        .spawn()?;

    watcher.watch(&args.path, RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Err(e) => eprintln!("{:?}", e),
            Ok(_) => {
                let _ = child.kill();
                let _ = child.wait();
                child = Command::new(args.command[0].clone())
                    .args(&mut args.command[1..])
                    .spawn()?;
            },
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.command.len() < 1 {
        eprintln!("missing command, check --help for usage");
        return ExitCode::FAILURE;
    }

    if let Err(e) = main_loop(args) {
        eprintln!("{:?}", e);
        return ExitCode::FAILURE;
    }

    return ExitCode::SUCCESS;
}
