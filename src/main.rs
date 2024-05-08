use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::string::String;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const LOCAL_BLOCKCHAIN_ADDR: &str = "127.0.0.1:9999";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
enum Commands {
    #[command(name = "start_node")]
    StartNode {
        #[clap(long, default_value = "10")]
        /// Seconds between each block
        block_time: String,
    },
    #[command(name = "create_account")]
    CreateAccount {
        // #[clap()]
        /// Name of the account holder
        name: String,
        // #[clap()]
        /// starting balance on the account
        balance: u128,
    },
    #[command(name = "balance")]
    Balance {
        // #[clap()]
        /// Name of the account holder
        name: String,
    },
}


fn main() {
    let cli = Cli::parse();

    let mut accounts = HashMap::<String, u128>::new();
    match &cli.command {
        Some(Commands::StartNode { block_time }) => {
            start_node(&mut accounts, block_time, &LOCAL_BLOCKCHAIN_ADDR.to_string());
        }
        Some(command) => {
            if let Ok(mut stream) = TcpStream::connect(LOCAL_BLOCKCHAIN_ADDR) {
                // Not as performant as binary representation, but easier to debug
                if let Ok(val) = stream.write(serde_json::to_string(command)
                    .expect("The command should be well formed already").as_bytes()) {
                    let mut buf = Vec::new();
                    if let Ok(_val) = stream.read(&mut buf) {
                        println!("{}", String::from_utf8(buf.into()).expect("We should have sent utf8"));
                    }
                }
            }
        }
        _ => { unreachable!() }
    }
}

fn _not_main() {
    let mut accounts = HashMap::<String, u128>::new();
    start_node(&mut accounts, &"2".to_string(), &"127.0.0.1:8888".to_string())
}

fn start_node(mut accounts: &mut HashMap<String, u128>, block_time: &String, addr: &String) {
    let block_time: u64 = block_time.parse().expect("Block time should be a number of seconds");
    assert!(block_time > 0, "Block time should be a positive number of seconds");
    let duration_between_blocks = Duration::from_secs(block_time);

    let mut last_mining_time = SystemTime::UNIX_EPOCH;
    let mut current_block_num = 0u128;

    let listener = TcpListener::bind(addr).unwrap();
    listener.set_nonblocking(true).expect("We should be on a OS where TCP can be non-blocking");
    loop {
        try_mining(duration_between_blocks, &mut last_mining_time, &mut current_block_num);
        if let Ok((stream, addr)) = listener.accept() {
            let mut stream = stream;/*.expect("The client should stay connected for at least one tick");*/
            let mut buf = String::new();
            let ret = stream.read_to_string(&mut buf);
            if let Ok(val) = ret {
                if val > 1 {
                    let response = process_remote_command(
                        &mut accounts,
                        serde_json::from_str(&buf).expect("We should have received a serialized Commands"),
                    );
                    if let Err(v) = stream.write(response.as_bytes()) {
                        println!("Couldn't respond: {}", response);
                    }
                }
            }
        }
    }
}

fn try_mining(duration_between_blocks: Duration, last_mining_time: &mut SystemTime, current_block_num: &mut u128) {
    let current_time = SystemTime::now();
    if current_time.duration_since(*last_mining_time).expect("Time should be monotonic") > duration_between_blocks {
        println!("{:.0?}: created block {} with content: {:#?}",
                 current_time.duration_since(UNIX_EPOCH).expect("Time should be monotonic"),
                 current_block_num,
                 ""
        );
        *current_block_num += 1;
        *last_mining_time = SystemTime::now();
    }
}

fn process_remote_command(accounts: &mut HashMap<String, u128>, command: Commands) -> String {
    match command {
        Commands::StartNode { block_time: _ } => {
            println!("We shouldn't receive that remotely");
            unimplemented!("We don't allow restarting the node remotely.");
        }
        Commands::CreateAccount { name, balance } => {
            accounts.insert(name.clone(), balance);
            format!("Created account of {} with balance {}", name, accounts.get(&name).expect("We should have inserted the account now."))
        }
        Commands::Balance { name } => {
            let balance = accounts.get(&name);
            match balance {
                Some(val) => format!("Account of {} has a balance of {}", name, val),
                None => format!("No account found for {}", name),
            }
        }
        _ => { unreachable!() }
    }
}

#[cfg(test)]
mod acceptance_tests;
