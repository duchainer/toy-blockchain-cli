use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::string::String;
use std::sync::mpsc;
use std::thread;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use block_chain::BlockChain;

mod block_chain;

#[cfg(test)]
mod acceptance_tests;

const LOCAL_BLOCKCHAIN_LISTEN_ADDR: &str = "0.0.0.0:9966";
const LOCAL_BLOCKCHAIN_ADDR: &str = "127.0.0.1:9966";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Serialize, Deserialize, Debug)]
enum Commands {
    #[command(name = "start_node")]
    /// Starts a new local blockchain, that mines a block every `block_time` (default 10s).
    /// -- NOTE: For now, we use a single hardcoded address for the start_node.
    ///       - So you'll need to un only a single one, or you'll see a "Address already in use" error
    StartNode {
        #[clap(long, default_value = "10")]
        /// Seconds between each block
        block_time: String,
    },
    #[command(name = "create_account")]
    /// Creates a new account with an initial balance
    /// Is a no-op if the account already exists, you'll just get an error message
    CreateAccount {
        /// Name of the account holder
        name: String,
        /// starting balance on the account
        balance: u64,
    },
    #[command(name = "balance")]
    /// Returns the balance of the account, if it exists
    Balance {
        /// Name of the account holder
        name: String,
    },
    #[command(name = "transfer")]
    /// Ask for a token transfer stored in the next mined block
    /// It will check twice if the transaction is valid, since balance can change
    Transfer {
        /// Name of the sending account holder
        sender: String,
        /// Name of the receiving account holder
        receiver: String,
        /// starting balance on the account
        balance: u64,
    },
}

#[derive(Debug, Clone)]
struct TransactionTransfer {
    /// Name of the sending account holder
    pub sender: String,
    /// Name of the receiving account holder
    pub receiver: String,
    /// starting balance on the account
    pub balance: u64,
}

#[derive(Debug, Clone)]
enum Transaction {
    CreateAccount {
        /// Name of the account holder
        name: String,
        /// starting balance on the account
        balance: u64,
    },
    Transfer(TransactionTransfer),
    Balance {
        /// Name of the account holder
        name: String,
    },
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

fn start_node(block_time: &str, addr: &str) {
    let block_time: u64 = block_time.parse().expect("Block time should be a number of seconds");
    assert!(block_time > 0, "Block time should be a positive number of seconds");
    // NOTE: We could have reused Commands::Transfer, but that could be bad "de-duplication"
    // as these data structures don't serve the same purpose and could diverge in later development.
    let (transactions_tx, transactions_rx) = mpsc::channel();

    thread::spawn(move || {
        let mut transactions_rx = transactions_rx;

        let mut block_chain = BlockChain::new(block_time);
        let mut transfers = Vec::new();
        loop {
            block_chain.try_mining(&mut transactions_rx, &mut transfers);
        }
    });

    let listener = TcpListener::bind(addr).unwrap();
    loop {
        if let Ok((stream, _addr)) = listener.accept() {
            let mut stream = stream;
            let mut buf = String::new();
            let ret = BufReader::new(&stream).read_line(&mut buf);
            if let Ok(val) = ret {
                if val > 1 {
                    let response = process_remote_command(
                        transactions_tx.clone(),
                        serde_json::from_str(&buf).expect("We should have received a serialized Commands"),
                    ) + "\n";
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


fn process_remote_command(transactions_tx: mpsc::Sender<(mpsc::Sender<String>, Transaction)>, command: Commands) -> String {
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
        Commands::Transfer { sender, receiver, balance } => {
            transactions_tx.send((msg_tx,
                                  Transaction::Transfer(TransactionTransfer {
                                      sender,
                                      receiver,
                                      balance,
                                  }))).expect("It should stay open until we kill the whole executable");
            msg_rx.recv().expect("Should be an error message, in the worst case")
        }
    }
}

