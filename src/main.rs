use std::thread::sleep;
use std::time::Duration;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name="start_node")]
    StartNode,
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::StartNode) => {
            loop { sleep(Duration::from_secs(1))}
        }
        _ => {unreachable!()}
    }
}

#[cfg(test)]
mod acceptance_tests;
