use std::process::{ExitCode, Command, Stdio};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Args {
    /// Amunt of jobs to run
    #[arg(short, long, default_value = "100")]
    jobs: usize,

    /// Parallel jobs to run
    #[arg(short, long, default_value = "10")]
    concurrency: usize,

    // Command to run
    #[arg(last = true)]
    command: Vec<String>,

}

fn main_loop(mut args: Args) -> Result<(), Box<dyn std::error::Error>> {

    let mut children: Vec<Option<std::process::Child>> = Vec::with_capacity(args.concurrency);

    for _ in 0..children.capacity() {
        children.push(None);
    }


    let mut counter = 0;
    let pb = ProgressBar::new(args.jobs.try_into()?);
    pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] {wide_bar} {pos}/{len} - {percent}%")?);

    loop {
        // exiting if done
        if counter >= args.jobs {
            break;
        }

        // removing done from queue
        for idx in 0..args.concurrency {
            if let Some(c) = &mut children[idx]{
                match c.try_wait() {
                    Ok(Some(status)) => children[idx] = None,
                    Ok(None) => {},
                    Err(_) => {},
                }
            } else {
                let mut child = Command::new(args.command[0].clone())
                    .args(&mut args.command[1..])
                    .spawn()?;
                children[idx] = Some(child);
                counter += 1;
                pb.inc(1);
            }
        }
    }
    pb.finish();

    Ok(())
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.command.len() < 1 {
        eprintln!("missing command, check --help for usage");
        return ExitCode::FAILURE
    }

    if let Err(e) = main_loop(args) {
        eprintln!("{:?}", e);
        return ExitCode::FAILURE;
    }

    return ExitCode::SUCCESS;
}
