use crate::wallet::key::Address;
use sha2::{Sha256, Digest};

pub struct Transaction {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
}

impl Transaction {
    pub fn new(from: Address, to: Address, amount: u64, sk: &[u8]) -> Self {
        let timestamp = chrono::Utc::now().timestamp() as u64;
        let mut tx = Self {
            from,
            to,
            amount,
            timestamp,
            signature: vec![],
        };
        tx.sign(sk);
        tx
    }
    
    fn sign(&mut self, sk: &[u8]) {
        let msg = format!("{:?}{:?}{}{}", self.from, self.to, self.amount, self.timestamp);
        let mut hasher = Sha256::new();
        hasher.update(msg);
        hasher.update(sk);
        self.signature = hasher.finalize().to_vec();
    }
    
    pub fn verify(&self) -> bool {
        // 简化验证：检查签名长度和非零
        !self.signature.is_empty() && self.amount > 0
    }
}
