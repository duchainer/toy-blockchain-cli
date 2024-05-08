use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::string::String;
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

const LOCAL_BLOCKCHAIN_LISTEN_ADDR: &str = "0.0.0.0:9998";
const LOCAL_BLOCKCHAIN_ADDR: &str = "127.0.0.1:9998";

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
        /// Name of the account holder
        name: String,
        /// starting balance on the account
        balance: u128,
    },
    #[command(name = "balance")]
    Balance {
        /// Name of the account holder
        name: String,
    },
}


fn main() {
    let cli = Cli::parse();

    let mut accounts = HashMap::<String, u128>::new();
    match &cli.command {
        Some(Commands::StartNode { block_time }) => {
            start_node(&mut accounts, block_time, LOCAL_BLOCKCHAIN_LISTEN_ADDR);
        }
        Some(command) => {
            ask_node(command, LOCAL_BLOCKCHAIN_ADDR);
        }
        _ => { unreachable!() }
    }
}

fn ask_node(command: &Commands, addr: &str) {
    // println!("{:?} sleeping", command);
    // sleep(Duration::from_secs(2));
    // println!("{:?} awoke", command);
    if let Ok(mut stream) = TcpStream::connect(addr) {
        // if stream.set_read_timeout(Some(Duration::from_secs(2))).is_err(){eprintln!("Could set read timeout")};
        // if stream.set_write_timeout(Some(Duration::from_secs(2))).is_err() { eprintln!("Could set write timeout") };
        // serde::json : Not as small over-the-wire as binary representation, but easier to debug
        if let Ok(val) = stream.write_all((serde_json::to_string(command)
            .expect("The command should be well formed already") + "\n").as_bytes()) {
            let mut buf = String::new();
            if let Ok(_val) = BufReader::new(stream).read_line(&mut buf) {
                println!("{:?}: {}", command, String::from_utf8(buf.into()).expect("We should have sent utf8"));
            }
        }
    }
}

fn _main() {
    let command = Commands::CreateAccount { name: "bob".to_string(), balance: 54321 };
    thread::spawn(move || ask_node(&command, "127.0.0.1:8888"));
    let command = Commands::Balance { name: "bob".to_string() };
    thread::spawn(move || ask_node(&command, "127.0.0.1:8888"));
    let mut accounts = HashMap::<String, u128>::new();
    // thread::spawn(move ||start_node(&mut accounts, &"2".to_string(), &"127.0.0.1:8888"));
    start_node(&mut accounts, "2", "127.0.0.1:8888");
}

fn start_node(accounts: &mut HashMap<String, u128>, block_time: &str, addr: &str) {
    let block_time: u64 = block_time.parse().expect("Block time should be a number of seconds");
    assert!(block_time > 0, "Block time should be a positive number of seconds");
    thread::spawn(move || {
        let duration_between_blocks = Duration::from_secs(block_time);

        let node_start_instant = Instant::now();
        let mut last_mining_time = Instant::now();
        let mut current_block_num = 0u128;
        loop {
            try_mining(duration_between_blocks, &node_start_instant, &mut last_mining_time, &mut current_block_num);
        }
    });

    let listener = TcpListener::bind(addr).unwrap();
    // listener.set_nonblocking(true).expect("We should be on a OS where TCP can be non-blocking");
    loop {
        if let Ok((stream, _addr)) = listener.accept() {
            let mut stream = stream;/*.expect("The client should stay connected for at least one tick");*/
            let mut buf = String::new();
            let ret = BufReader::new(&stream/*.try_clone().expect("TcpStream should be cloneable")*/).read_line(&mut buf);
            if let Ok(val) = ret {
                println!("{:?}", dbg!(serde_json::from_str::<Commands>(&buf)));
                if val > 1 {
                    let response = dbg!(process_remote_command(
                        accounts,
                        // (serde_json::from_str(&buf)).expect("We should have received a serialized Commands"),
                        // TODO Figure out why comm only reads empty string
                        Commands::CreateAccount { name: "bob".to_string(), balance: 1000 },
                    ) + "\n");
                    match stream.write_all(response.as_bytes()) {
                        Err(v) => {
                            println!("Couldn't respond: {} because {}", response, v);
                        }
                        a => {
                            println!("Tried to respond: {} , and sent {:?} bytes", response, a);
                        }
                    }
                }
            }
        }
    }
}

fn try_mining(duration_between_blocks: Duration, node_start_instant: &Instant, last_mining_time: &mut Instant, current_block_num: &mut u128) {
    let current_time = Instant::now();
    if current_time.duration_since(*last_mining_time) > duration_between_blocks {
        println!("{:.0?}: created block {} with content: {:#?}",
                 current_time.duration_since(*node_start_instant),
                 current_block_num,
                 ""
        );
        *current_block_num += 1;
        *last_mining_time = Instant::now();
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
