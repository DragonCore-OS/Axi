use std::collections::HashMap;
use crate::wallet::key::Address;

pub struct Ledger {
    balances: HashMap<String, u64>,
    last_movement: HashMap<String, u64>, // 记录上次转账时间（用于燃烧）
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            last_movement: HashMap::new(),
        }
    }
    
    pub fn balance(&self, addr: &Address) -> u64 {
        *self.balances.get(&addr.to_string()).unwrap_or(&0)
    }
    
    pub fn mint(&mut self, addr: &Address, amount: u64, timestamp: u64) {
        let key = addr.to_string();
        *self.balances.entry(key.clone()).or_insert(0) += amount;
        self.last_movement.insert(key, timestamp);
    }
    
    pub fn transfer(&mut self, from: &Address, to: &Address, amount: u64, timestamp: u64) -> Result<(), &'static str> {
        let from_key = from.to_string();
        let to_key = to.to_string();
        
        let balance = self.balances.get(&from_key).copied().unwrap_or(0);
        if balance < amount {
            return Err("Insufficient balance");
        }
        
        *self.balances.entry(from_key.clone()).or_insert(0) -= amount;
        *self.balances.entry(to_key.clone()).or_insert(0) += amount;
        
        self.last_movement.insert(from_key, timestamp);
        self.last_movement.insert(to_key, timestamp);
        
        Ok(())
    }
    
    pub fn get_last_movement(&self, addr: &Address) -> u64 {
        *self.last_movement.get(&addr.to_string()).unwrap_or(&0)
    }
}
