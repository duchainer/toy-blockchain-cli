use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use crate::Transaction;

#[derive(Debug)]
struct Block {
    current_block_num: usize,
    transactions: Vec<Transaction>,
}

#[derive(Debug)]
pub struct BlockChain {
    node_start_instant: Instant,
    duration_between_blocks: Duration,
    last_mining_time: Instant,
    blocks: Vec::<Block>,
    accounts: HashMap::<String, u64>,
}

impl Default for BlockChain {
    fn default() -> Self {
        Self::new(10)
    }
}

impl BlockChain {
    pub(crate) fn new(block_time: u64) -> Self {
        let node_start_instant = Instant::now();
        let mut last_mining_time = Instant::now();
        let mut blocks = Vec::new();
        let duration_between_blocks = Duration::from_secs(block_time);
        let accounts = HashMap::new();
        Self {
            node_start_instant,
            duration_between_blocks,
            last_mining_time,
            blocks,
            accounts,
        }
    }
}

impl BlockChain {
    pub(crate) fn try_mining(&mut self, transactions_rx: &mut Receiver<(mpsc::Sender<String>, Transaction)>) {
        let current_time = Instant::now();
        if current_time.duration_since(self.last_mining_time) > self.duration_between_blocks {
            let mut block = Block {
                current_block_num: self.blocks.len(),
                transactions: Vec::<Transaction>::new(),
            };
            while let Ok((msg_tx, transaction)) = transactions_rx.try_recv() {
                msg_tx.send(match transaction {
                    Transaction::Balance { name } => {
                        let balance = self.accounts.get(&name);
                        match balance {
                            Some(val) => format!("Account of {} has a balance of {}", name, val),
                            None => format!("No account found for {}", name),
                        }
                    }
                    Transaction::CreateAccount { name, balance } => {
                        match self.accounts.insert(name.clone(), balance) {
                            None => {
                                format!("Created account of {} with balance {}",
                                        name,
                                        self.accounts.get(&name).expect("We should have inserted the account now."))
                            }
                            Some(balance) => {
                                format!("Already existing account of {} with balance {}", name, balance)
                            }
                        }
                    }
                    transaction @ Transaction::Transfer { .. } => {
                        if let Err(msg) = transfer_between_accounts(&mut self.accounts, &transaction) {
                            msg
                        } else {
                            if let Transaction::Transfer { sender, receiver, balance } = &transaction {
                                {
                                    *self.accounts.get_mut(sender)
                                        .expect("We always have a number in there if the key exists") -= balance
                                }
                                {
                                    *self.accounts.get_mut(receiver)
                                        .expect("We always have a number in there if the key exists") += balance
                                }
                                block.transactions.push(transaction.clone());
                                format!("Successfully transferred {} from {} to {}", balance, sender, receiver)
                            } else {
                                format!("Wrong transaction: {:?}", transaction)
                            }
                        }
                    }
                }).expect("msg_tx should be open for one send");
            }
            self.blocks.push(block);
            println!("{:.0?}: created block {:?}",
                     current_time.duration_since(self.node_start_instant),
                     self.blocks.last().expect("Just placed it in")
            );
            self.last_mining_time = Instant::now();
        }
    }
    pub(crate) fn balance(&self, account_name: &str, base_balance: u128) -> u64 {
        todo!("");
        // let diff_balance :i128 = self.blocks.
        // base_balance +
    }
}

fn transfer_between_accounts(accounts: &mut HashMap<String, u64>, t: &Transaction) -> Result<(), String> {
    match t {
        Transaction::Transfer { sender, receiver, balance } =>
            {
                if let Some(sender_balance) = accounts.get_mut(sender) {
                    if *sender_balance >= *balance {
                        // NOTE In a real system, we would use atomic operations/transaction
                        *sender_balance -= *balance;
                        if let Some(receiver_balance) = accounts.get_mut(receiver) {
                            *receiver_balance += *balance;
                            return Ok(());
                        } else {
                            *accounts.get_mut(sender).expect("It existed a few statement ago") -= *balance;
                        }
                    }
                }
                Err(format!("Failed to transfer {} from {} to {}", balance, sender, receiver))
            }
        _ => Err("We can't transfer using anything but Transaction::Transfer".to_string()),
    }
}
