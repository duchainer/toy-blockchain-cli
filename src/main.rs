use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "start_node")]
    StartNode{
        #[clap(long, default_value = "10")]
        /// Seconds between each block
        block_time: String
    },
}


fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::StartNode{block_time}) => {
            let block_time: u64 = block_time.parse().expect("Block time should be a number of seconds");
            assert!(block_time > 0, "Block time should be a positive number of seconds");
            let mut current_block_num = 0u128;
            loop {
                sleep(Duration::from_secs(block_time));
                println!("{:.0?}: created block {} with content: {:#?}",
                         SystemTime::now().duration_since(UNIX_EPOCH).expect("Time should be monotonic"),
                         current_block_num,
                         ""
                );
                current_block_num += 1;
            }
        }
        _ => { unreachable!() }
    }
}

#[cfg(test)]
mod acceptance_tests;
