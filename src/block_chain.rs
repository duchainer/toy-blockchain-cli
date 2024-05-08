use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use crate::{Transaction, TransactionTransfer};

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
    pub(crate) fn try_mining(
        &mut self,
        transactions_rx: &mut Receiver<(mpsc::Sender<String>, Transaction)>,
        transfers: &mut Vec<TransactionTransfer>,
    ) {
        let current_time = Instant::now();
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
                Transaction::Transfer(transaction @ TransactionTransfer { .. }) => {
                    match can_transfer(&mut self.accounts, &transaction) {
                        Ok(()) => {
                            transfers.push(transaction.clone());
                            format!("Will add this transaction in the next block: {:?}", &transaction)
                        }
                        Err(msg) => {
                            msg
                        }
                    }
                }
            }).expect("msg_tx should be open for one send");
        }
        if current_time.duration_since(self.last_mining_time) > self.duration_between_blocks {
            transfers.iter().for_each(|transaction| {
                self.transfer(&mut block, &transaction);
            });
            self.blocks.push(block);
            println!("{:.0?}: created block {:?}",
                     current_time.duration_since(self.node_start_instant),
                     self.blocks.last().expect("Just placed it in")
            );
            self.last_mining_time = Instant::now();
        }
    }

    fn transfer(&mut self, block: &mut Block, transaction: &TransactionTransfer) -> String {
        if let Err(msg) = can_transfer(&self.accounts, &transaction) {
            return msg;
        }
        if let Err(msg) = transfer_between_accounts(&mut self.accounts, &transaction) {
            msg
        } else {
            block.transactions.push(Transaction::Transfer(transaction.clone()));
            format!("Successfully transferred {} from {} to {}", transaction.balance, transaction.sender, transaction.receiver)
        }
    }
}

fn can_transfer(accounts: &HashMap<String, u64>, transfer: &TransactionTransfer) -> Result<(), String> {
    if let Some(sender_balance) = accounts.get(&transfer.sender) {
        if *sender_balance >= transfer.balance {
            if accounts.contains_key(&transfer.receiver) {
                Ok(())
            } else {
                Err(format!("Missing receiver's account: {}: cannot send {} to {}",
                            &transfer.receiver, &transfer.sender, &transfer.balance))
            }
        } else {
            Err(format!("Insufficient funds in {}'s account: cannot send {} to {}",
                        &transfer.sender, &transfer.balance, &transfer.receiver))
        }
    } else {
        Err(format!("Missing sender's account: {}: cannot send {} to {}",
                    &transfer.sender, &transfer.balance, &transfer.receiver))
    }
}

fn transfer_between_accounts(accounts: &mut HashMap<String, u64>, t: &TransactionTransfer) -> Result<(), String> {
    if let Some(sender_balance) = accounts.get_mut(&t.sender) {
        if *sender_balance >= t.balance {
            // NOTE In a real system, we would use atomic operations/transaction
            *sender_balance -= t.balance;
            if let Some(receiver_balance) = accounts.get_mut(&t.receiver) {
                *receiver_balance += t.balance;
                return Ok(());
            } else {
                *accounts.get_mut(&t.sender).expect("It existed a few statement ago") -= t.balance;
            }
        }
    }
    Err(format!("Failed to transfer {} from {} to {}", t.balance, t.sender, t.receiver))
}
