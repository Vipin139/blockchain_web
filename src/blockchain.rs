use chrono::prelude::*;
use std::collections::{HashMap, HashSet};
use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block<T> {
    pub index: u64,
    pub timestamp: i64,
    pub data: T,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub merkle_root: String,
}

trait Hashable {
    fn calculate_hash(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block<Vec<Transaction>>>,
    pub balances: HashMap<String, u64>,
    pub seen_transactions: HashSet<String>,
    pub mempool: Arc<Mutex<Vec<Transaction>>>
}

impl Blockchain {
    pub fn new() -> Self {
        let mut chain = Vec::new();

        let genesis = Blockchain::create_genesis_block();

        chain.push(genesis);

        let mut balances = HashMap::new();

        balances.insert("Alice".to_string(), 100);
        balances.insert("Bob".to_string(), 50);

        Blockchain {
            chain,
            balances,
            seen_transactions: HashSet::new(),
            mempool: Arc::new(Mutex::new(Vec::new()))
        }

    }

    pub fn create_genesis_block() -> Block<Vec<Transaction>> {

        let data = Vec::new();

        let timestamp = Utc::now().timestamp();

        let hash_input = format!("{}{}{}{}", 0, timestamp, "0", 0);

        let hash = calculate_hash(&hash_input);

        let merkle_root = hash.clone();

        Block {
            index: 0,
            timestamp,
            data,
            previous_hash: "0".to_string(),
            hash,
            nonce: 0,
            merkle_root: merkle_root.to_string()
        }

    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {

        if self.seen_transactions.contains(&tx.id) {
            return Err("Duplicate transaction".to_string());
        }

        if tx.amount == 0 {
            return Err("Transaction amount must be greater than 0".to_string());
        }

        let sender_balance =  self.balances.get(&tx.sender).cloned().unwrap_or(0);

        if sender_balance < tx.amount {
            return Err("Insufficient balance".to_string());
        }

        self.seen_transactions.insert(tx.id.clone());

        let mut pool = self.mempool.lock().unwrap();
        pool.push(tx);

        Ok(())

    } 

    pub fn mine_pending_transaction(&mut self) {

        let mut pool = self.mempool.lock().unwrap();

        if pool.is_empty() {
            println!("No transactions to mine");
            return;
        }

        let transactions = pool.clone();

        let merkle_root = compute_merkle_root(&transactions);

        let prev_block = self.chain.last().unwrap();

        let index = prev_block.index + 1;
        let timestamp = Utc::now().timestamp();
        let previous_hash = prev_block.hash.clone();

        let mut nonce = 0;

        let hash;

        loop {

            let input = format!("{}{}{}{}{}{}", index, timestamp, merkle_root, previous_hash, nonce, transactions.len());

            let candidate = calculate_hash(&input);

            if candidate.starts_with("0000") {
                hash = candidate;
                break;
            }

            nonce += 1;

        }

        let block = Block {
                index,
                timestamp,
                data: transactions.clone(),
                previous_hash,
                hash,
                nonce,
                merkle_root
            };

        for tx in &transactions {

            let sender_balance = self.balances.entry(tx.sender.clone()).or_insert(0);
            *sender_balance -= tx.amount;

            let receiver_balance = self.balances.entry(tx.receiver.clone()).or_insert(0);
            *receiver_balance += tx.amount;
        }

        self.chain.push(block);

        pool.clear();

        println!("Block mined successfully!");

    }

    pub fn is_chain_valid(&self) -> bool {

        for i in 1..self.chain.len() {
            
            let current = &self.chain[i];
            let previous = &self.chain[i-1];

            if current.previous_hash != previous.hash {
                return false;
            }

            let input = format!("{}{}{:?}{}{}", current.index, current.timestamp, current.data, current.previous_hash, current.nonce);

            let recalculated = calculate_hash(&input);

            if current.hash != recalculated {
                return false;
            }

            if current.timestamp < previous.timestamp {
                return false;
            }

            for tx in &current.data {

                if tx.amount == 0 {
                    return false;
                }
            }
        }

        true
    }

    pub fn transfer(&mut self, sender: &str, receiver: &str, amount: u64) -> Result<(), String> {

        if sender == receiver {
            return Err("Sender and receiver cannot be the same".to_string());
        }

        if amount == 0 {
            return Err("Amount must be greater than 0".to_string());
        }

        let balance = self.balances.get(sender).cloned().unwrap_or(0);

        if balance < amount {
            return  Err("Insufficient balance".to_string());
        }

        let id = format!("{}-{}-{}-{}", sender, receiver, amount, Utc::now().timestamp());

        let tx = Transaction {
            id,
            sender: sender.to_string(),
            receiver: receiver.to_string(),
            amount,
        };

        self.add_transaction(tx)
    }

    pub fn detect_fork(chain1: &Vec<Block<Vec<Transaction>>>, chain2: &Vec<Block<Vec<Transaction>>>) -> Option<usize> {

        let min_len = std::cmp::min(chain1.len(), chain2.len());

        for i in 0..min_len {

            if chain1[i].hash != chain2[i].hash {
                return Some(i);
            }
        }

        None
    }

    pub fn resolve_work(chain1: &Vec<Block<Vec<Transaction>>>, chain2: &Vec<Block<Vec<Transaction>>>) -> Vec<Block<Vec<Transaction>>> {

        if chain1.len() > chain2.len() {
            chain1.clone()
        } else {
            chain2.clone()
        }
    }

}

pub fn compute_merkle_root(txs: &Vec<Transaction>) -> String {

        if txs.is_empty() {
            return calculate_hash("empty");
        }

        let mut hashes: Vec<String> = txs.iter().map(|tx| calculate_hash(&format!("{:?}", tx))).collect();

        while hashes.len() > 1 {

            let mut next_level = Vec::new();

            for i in (0..hashes.len()).step_by(2) {

                let left = &hashes[i];

                let right = if i + 1 < hashes.len() {
                    &hashes[i+1]
                } else {
                    left
                };

                let combined = format!("{}{}", left, right);

                next_level.push(calculate_hash(&combined));
            }

            hashes = next_level;
        }

        hashes[0].clone()
    }

pub fn calculate_hash(input: &str) -> String {

    let mut hasher = Sha256::new();

    hasher.update(input);

    let result = hasher.finalize();

    hex::encode(result)
}