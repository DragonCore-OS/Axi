use sha2::{Sha256, Digest};
use chrono::Utc;

pub struct GenesisBlock {
    pub timestamp: u64,
    pub anchor_power: f64,    // kWh
    pub anchor_compute: f64,  // TFLOPs
    pub hash: String,
    pub constitution_hash: String,
}

impl GenesisBlock {
    pub fn new(power_kwh: f64, compute_tflops: f64) -> Self {
        let timestamp = Utc::now().timestamp() as u64;
        let constitution = include_str!("../../CONSTITUTION.md");
        
        let mut hasher = Sha256::new();
        hasher.update(constitution);
        let const_hash = format!("{:x}", hasher.finalize());
        
        let mut hasher = Sha256::new();
        hasher.update(&timestamp.to_le_bytes());
        hasher.update(&power_kwh.to_le_bytes());
        hasher.update(&compute_tflops.to_le_bytes());
        let block_hash = format!("{:x}", hasher.finalize());
        
        Self {
            timestamp,
            anchor_power: power_kwh,
            anchor_compute: compute_tflops,
            hash: block_hash,
            constitution_hash: const_hash,
        }
    }
    
    pub fn verify_constitution(&self) -> bool {
        let current = include_str!("../../CONSTITUTION.md");
        let mut hasher = Sha256::new();
        hasher.update(current);
        let hash = format!("{:x}", hasher.finalize());
        hash == self.constitution_hash
    }
}
