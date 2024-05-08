use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::string::String;
use std::sync::mpsc;
use std::thread;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use block_chain::BlockChain;

const LOCAL_BLOCKCHAIN_LISTEN_ADDR: &str = "0.0.0.0:9996";
const LOCAL_BLOCKCHAIN_ADDR: &str = "127.0.0.1:9996";

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
        balance: u64,
    },
    #[command(name = "balance")]
    Balance {
        /// Name of the account holder
        name: String,
    },
    #[command(name = "transfer")]
    Transfer {
        /// Name of the sending account holder
        sender: String,
        /// Name of the receiving account holder
        receiver: String,
        /// starting balance on the account
        balance: u64,
    },
}

#[derive(Debug)]
enum Transaction {
    CreateAccount {
        /// Name of the account holder
        name: String,
        /// starting balance on the account
        balance: u64,
    },
    Transfer {
        /// Name of the sending account holder
        sender: String,
        /// Name of the receiving account holder
        receiver: String,
        /// starting balance on the account
        balance: u64,
    },
    Balance{
        /// Name of the account holder
        name: String,
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::StartNode { block_time }) => {
            start_node(block_time, LOCAL_BLOCKCHAIN_LISTEN_ADDR);
        }
        Some(command) => {
            println!("{}", ask_node(command, LOCAL_BLOCKCHAIN_ADDR));
        }
        _ => { unreachable!() }
    }
}

fn ask_node(command: &Commands, addr: &str) -> String {
    if let Ok(mut stream) = TcpStream::connect(addr) {
        // if stream.set_read_timeout(Some(Duration::from_secs(2))).is_err(){eprintln!("Could set read timeout")};
        // if stream.set_write_timeout(Some(Duration::from_secs(2))).is_err() { eprintln!("Could set write timeout") };
        // serde::json : Not as small over-the-wire as binary representation, but easier to debug
        if let Ok(val) = stream.write_all((serde_json::to_string(command)
            .expect("The command should be well formed already") + "\n").as_bytes()) {
            let mut buf = String::new();
            if let Ok(_val) = BufReader::new(stream).read_line(&mut buf) {
                format!("{:?}: {}", command, String::from_utf8(buf.into()).expect("We should have sent utf8"))
            } else {
                "Could not read from server sending the command".to_string()
            }
        } else {
            "Could not write to server after initial connection".to_string()
        }
    } else {
        "Could not connect to server".to_string()
    }
}

fn _main() {
    let command = Commands::CreateAccount { name: "bob".to_string(), balance: 54321 };
    thread::spawn(move || ask_node(&command, "127.0.0.1:8888"));
    let command = Commands::Balance { name: "bob".to_string() };
    thread::spawn(move || ask_node(&command, "127.0.0.1:8888"));
    // thread::spawn(move ||start_node(&mut accounts, &"2".to_string(), &"127.0.0.1:8888"));
    start_node( "2", "127.0.0.1:8888");
}

fn start_node(block_time: &str, addr: &str) {
    let block_time: u64 = block_time.parse().expect("Block time should be a number of seconds");
    assert!(block_time > 0, "Block time should be a positive number of seconds");
    // NOTE: We could have reused Commands::Transfer, but that would be bad "de-duplication"
    // as these data structures don't serve the same purpose and will diverge in later development.
    let (transactions_tx, transactions_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut transactions_rx = transactions_rx;

        let mut block_chain = BlockChain::new(block_time);
        loop {
            block_chain.try_mining(&mut transactions_rx);
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
                eprintln!("{:?}", dbg!(serde_json::from_str::<Commands>(&buf)));
                if val > 1 {
                    let response = dbg!(process_remote_command(
                        transactions_tx.clone(),
                        serde_json::from_str(&buf).expect("We should have received a serialized Commands"),
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

fn process_remote_command(transactions_tx: mpsc::Sender<(mpsc::Sender<String>, Transaction)>,  command: Commands) -> String {
    let (msg_tx, msg_rx) = mpsc::channel();
    match command {
        Commands::StartNode { block_time: _ } => {
            println!("We shouldn't receive that remotely");
            unimplemented!("We don't allow restarting the node remotely.");
        }
        Commands::CreateAccount { name, balance } => {
            transactions_tx.send((msg_tx,
                                  Transaction::CreateAccount {
                                      name,
                                      balance,
                                  })).expect("It should stay open until we kill the whole executable");
            msg_rx.recv().expect("Should be an error message, in the worst case")
        }
        Commands::Balance { name } => {
            transactions_tx.send((msg_tx,
                                  Transaction::Balance {
                                      name,
                                  })).expect("It should stay open until we kill the whole executable");
            msg_rx.recv().expect("Should be an error message, in the worst case")
            
        }
        command @ Commands::Transfer { .. } => {
            return format!("Will add this transaction in the next block: {:?}", &command);
        }
        _ => { unreachable!() }
    }
}


#[cfg(test)]
mod acceptance_tests;
mod block_chain;
